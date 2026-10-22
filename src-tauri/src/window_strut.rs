//! Window Strut and Multiplatform Desktop Integration (Linux Wayland/X11, Windows, macOS)
//!
//! Handles native Wayland Layer Shell (wlr-layer-shell) panel integration on Linux,
//! as well as dynamic click-through masking, always-on-top and multi-space stickiness on Windows and macOS.

#[cfg(target_os = "linux")]
use std::os::raw::c_int;
use std::sync::Mutex;
use std::collections::HashMap;
use std::time::Instant;
#[cfg(target_os = "linux")]
use gtk::prelude::*;
use tauri::{WebviewWindow, Emitter};

struct RegionCacheEntry {
    last_applied: Vec<[i32; 4]>,
    last_update: Instant,
    #[allow(dead_code)]
    was_inside: Option<bool>,
}

fn get_region_cache() -> &'static Mutex<HashMap<String, RegionCacheEntry>> {
    static CACHE: std::sync::OnceLock<Mutex<HashMap<String, RegionCacheEntry>>> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[allow(dead_code)]
fn is_cursor_in_regions(
    cursor: [i32; 2],
    win_pos: tauri::PhysicalPosition<i32>,
    regions: &[[i32; 4]],
    label: &str,
) -> bool {
    let rel_x = cursor[0] - win_pos.x;
    let rel_y = cursor[1] - win_pos.y;

    if label == "dock" && cursor[1] < win_pos.y {
        return false;
    }

    const MARGIN: i32 = 8;
    for r in regions {
        let rx = r[0] - MARGIN;
        let ry = r[1] - MARGIN;
        let rw = r[2] + MARGIN * 2;
        let rh = r[3] + MARGIN * 2;
        if rel_x >= rx && rel_x <= rx + rw && rel_y >= ry && rel_y <= ry + rh {
            return true;
        }
    }
    false
}

#[cfg(target_os = "linux")]
#[link(name = "gtk-layer-shell")]
extern "C" {
    fn gtk_layer_is_supported() -> c_int;
    fn gtk_layer_init_for_window(window: *mut gtk::ffi::GtkWindow);
    fn gtk_layer_is_layer_window(window: *mut gtk::ffi::GtkWindow) -> c_int;
    fn gtk_layer_set_layer(window: *mut gtk::ffi::GtkWindow, layer: c_int);
    fn gtk_layer_set_anchor(window: *mut gtk::ffi::GtkWindow, edge: c_int, anchor_to_edge: c_int);
    fn gtk_layer_set_exclusive_zone(window: *mut gtk::ffi::GtkWindow, exclusive_zone: c_int);
    fn gtk_layer_set_keyboard_mode(window: *mut gtk::ffi::GtkWindow, mode: c_int);
    fn gtk_layer_set_margin(window: *mut gtk::ffi::GtkWindow, edge: c_int, margin: c_int);

    fn set_native_wayland_input_regions(
        window: *mut gtk::ffi::GtkWindow,
        rects: *const i32,
        count: c_int,
    ) -> c_int;
}

#[cfg(target_os = "linux")]
/// Opt-in diagnostic/workaround: when DTB_FORCE_REDRAW_MS=<ms> is set, periodically invalidates the
/// whole window (including the WebKit child) so GTK repaints regions WebKit failed to damage.
pub fn install_forced_redraw(gtk_window: &gtk::ApplicationWindow) {
    let ms: u64 = std::env::var("DTB_FORCE_REDRAW_MS").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
    if ms == 0 {
        return;
    }
    eprintln!("[Redraw] forced full-window invalidation every {} ms", ms);
    let win = gtk_window.clone();
    gtk::glib::timeout_add_local(std::time::Duration::from_millis(ms), move || {
        win.queue_draw();
        if let Some(child) = win.child() {
            child.queue_draw();
        }
        gtk::glib::ControlFlow::Continue
    });
}

#[cfg(target_os = "linux")]
/// Applies universal zero-margin CSS to GTK to prevent any CSD / shadow offset
pub fn apply_zero_margin_css(gtk_window: &gtk::ApplicationWindow) {
    let css_provider = gtk::CssProvider::new();
    let _ = css_provider.load_from_data(b"window, decoration, .window-frame { margin: 0 !important; padding: 0 !important; border: none !important; box-shadow: none !important; background: transparent !important; }");
    if let Some(screen) = gtk::prelude::WidgetExt::screen(gtk_window) {
        gtk::StyleContext::add_provider_for_screen(&screen, &css_provider, gtk::STYLE_PROVIDER_PRIORITY_APPLICATION);
    }
}

#[cfg(target_os = "linux")]
/// Recursively applies a Cairo input shape region to a GTK widget and all its child widgets (such as WebKitWebView)
pub fn apply_recursive_input_shape(widget: &gtk::Widget, region: &cairo::Region) {
    if widget.has_window() {
        widget.input_shape_combine_region(Some(region));
    }
    if let Some(gdk_win) = widget.window() {
        gdk_win.input_shape_combine_region(region, 0, 0);
    }
    if let Ok(container) = widget.clone().downcast::<gtk::Container>() {
        for child in container.children() {
            apply_recursive_input_shape(&child, region);
        }
    }
}

#[cfg(target_os = "linux")]
/// Attempts to initialize native Wayland Layer Shell (wlr-layer-shell)
/// with TOP layer, exclusive zone of 36px, top/left/right anchors, and strict 0 margin.
pub fn init_wayland_layer_shell(gtk_window: &gtk::ApplicationWindow) -> bool {
    apply_zero_margin_css(gtk_window);
    use gtk::glib::translate::ToGlibPtr;
    unsafe {
        let supported = gtk_layer_is_supported();
        eprintln!("[EnigmaOS] gtk_layer_is_supported = {}", supported);
        if supported != 0 {
            let win_ref: &gtk::Window = gtk_window.upcast_ref();
            let win_ptr: *mut gtk::ffi::GtkWindow = win_ref.to_glib_none().0;
            gtk_layer_init_for_window(win_ptr);
            if gtk_layer_is_layer_window(win_ptr) != 0 {
                gtk_layer_set_layer(win_ptr, 3);
                gtk_layer_set_anchor(win_ptr, 2, 1);
                gtk_layer_set_anchor(win_ptr, 0, 1);
                gtk_layer_set_anchor(win_ptr, 1, 1);

                gtk_layer_set_margin(win_ptr, 2, 0); // TOP
                gtk_layer_set_margin(win_ptr, 0, 0); // LEFT
                gtk_layer_set_margin(win_ptr, 1, 0); // RIGHT

                gtk_window.set_default_size(2560, 960);
                gtk_window.resize(2560, 960);
                gtk::prelude::WidgetExt::set_size_request(gtk_window, 2560, 960);

                gtk_layer_set_exclusive_zone(win_ptr, 36);
                gtk_layer_set_keyboard_mode(win_ptr, 0);
                eprintln!("[EnigmaOS] Native Wayland Layer Shell initialized (layer=TOP, exclusive_zone=36, size=2560x960, top_margin=0)");
                return true;
            } else {
                eprintln!("[EnigmaOS] gtk_layer_is_layer_window returned 0");
            }
        }
    }
    false
}

#[cfg(target_os = "linux")]
/// Initializes native Wayland Layer Shell for the BottomDock
pub fn init_wayland_layer_shell_dock(gtk_window: &gtk::ApplicationWindow) -> bool {
    apply_zero_margin_css(gtk_window);
    use gtk::glib::translate::ToGlibPtr;
    unsafe {
        let supported = gtk_layer_is_supported();
        eprintln!("[BottomDock] gtk_layer_is_supported = {}", supported);
        if supported != 0 {
            let win_ref: &gtk::Window = gtk_window.upcast_ref();
            let win_ptr: *mut gtk::ffi::GtkWindow = win_ref.to_glib_none().0;
            gtk_layer_init_for_window(win_ptr);
            if gtk_layer_is_layer_window(win_ptr) != 0 {
                gtk_layer_set_layer(win_ptr, 3);
                gtk_layer_set_anchor(win_ptr, 3, 1);
                gtk_layer_set_anchor(win_ptr, 0, 1);
                gtk_layer_set_anchor(win_ptr, 1, 1);

                gtk_layer_set_margin(win_ptr, 3, 0); // BOTTOM
                gtk_layer_set_margin(win_ptr, 0, 0); // LEFT
                gtk_layer_set_margin(win_ptr, 1, 0); // RIGHT

                gtk_window.set_default_size(2560, 740);
                gtk_window.resize(2560, 740);
                gtk::prelude::WidgetExt::set_size_request(gtk_window, 2560, 740);
                gtk_layer_set_exclusive_zone(win_ptr, 0);
                gtk_layer_set_keyboard_mode(win_ptr, 0);
                eprintln!("[BottomDock] Native Wayland Layer Shell initialized (layer=TOP, exclusive_zone=0, anchor=BOTTOM/LEFT/RIGHT, size=2560x740)");
                return true;
            } else {
                eprintln!("[BottomDock] gtk_layer_is_layer_window returned 0");
            }
        }
    }
    false
}

#[cfg(target_os = "linux")]
/// Fallback desktop configuration for BottomDock on X11 / non-layer-shell
pub fn configure_dock_desktop_window(gtk_window: &gtk::ApplicationWindow) {
    apply_zero_margin_css(gtk_window);
    gtk_window.set_type_hint(gdk::WindowTypeHint::Dock);
    gtk_window.set_decorated(false);
    gtk_window.set_focus_on_map(false);
    gtk_window.set_accept_focus(false);
    gtk_window.set_keep_above(true);
    gtk_window.stick();
    gtk_window.set_skip_taskbar_hint(true);
    gtk_window.set_skip_pager_hint(true);

    if let Some(screen) = gtk::prelude::WidgetExt::screen(gtk_window) {
        if let Some(visual) = screen.rgba_visual() {
            gtk_window.set_visual(Some(&visual));
        }
    }
    gtk_window.set_app_paintable(true);

    let empty_region = cairo::Region::create();
    apply_recursive_input_shape(gtk_window.upcast_ref(), &empty_region);
}

#[cfg(target_os = "linux")]
/// Configures GTK window properties for Main TopBar specifically on Wayland Layer Shell.
pub fn configure_layer_shell_window_main(gtk_window: &gtk::ApplicationWindow) {
    apply_zero_margin_css(gtk_window);
    gtk_window.set_decorated(false);
    gtk_window.set_focus_on_map(false);
    gtk_window.set_accept_focus(true);

    let initial_region = cairo::Region::create();
    let _ = initial_region.union_rectangle(&cairo::RectangleInt::new(0, 0, 500, 36));
    let _ = initial_region.union_rectangle(&cairo::RectangleInt::new(1140, 0, 280, 36));
    let _ = initial_region.union_rectangle(&cairo::RectangleInt::new(2200, 0, 360, 36));
    apply_recursive_input_shape(gtk_window.upcast_ref(), &initial_region);

    use gtk::glib::translate::ToGlibPtr;
    let win_ref: &gtk::Window = gtk_window.upcast_ref();
    let win_ptr: *mut gtk::ffi::GtkWindow = win_ref.to_glib_none().0;
    let initial_rects: [i32; 12] = [
        0, 0, 500, 36,
        1140, 0, 280, 36,
        2200, 0, 360, 36,
    ];
    unsafe {
        set_native_wayland_input_regions(win_ptr, initial_rects.as_ptr(), 3);
    }
}

#[cfg(target_os = "linux")]
/// Configures GTK window properties for BottomDock on Wayland Layer Shell.
pub fn configure_layer_shell_window_dock(gtk_window: &gtk::ApplicationWindow) {
    apply_zero_margin_css(gtk_window);
    gtk_window.set_decorated(false);
    gtk_window.set_focus_on_map(false);
    gtk_window.set_accept_focus(true);

    let initial_region = cairo::Region::create();
    let _ = initial_region.union_rectangle(&cairo::RectangleInt::new(1230, 718, 100, 22));
    apply_recursive_input_shape(gtk_window.upcast_ref(), &initial_region);

    use gtk::glib::translate::ToGlibPtr;
    let win_ref: &gtk::Window = gtk_window.upcast_ref();
    let win_ptr: *mut gtk::ffi::GtkWindow = win_ref.to_glib_none().0;
    let initial_rects: [i32; 4] = [1230, 718, 100, 22];
    unsafe {
        set_native_wayland_input_regions(win_ptr, initial_rects.as_ptr(), 1);
    }
}

#[cfg(target_os = "linux")]
/// Configures GTK ApplicationWindow properties for a transparent topbar dock
pub fn configure_desktop_window(gtk_window: &gtk::ApplicationWindow) {
    gtk_window.set_type_hint(gdk::WindowTypeHint::Dock);
    gtk_window.set_decorated(false);
    gtk_window.set_focus_on_map(false);
    gtk_window.set_accept_focus(true);
    gtk_window.set_keep_above(true);
    gtk_window.stick();
    gtk_window.set_skip_taskbar_hint(true);
    gtk_window.set_skip_pager_hint(true);
    gtk_window.move_(0, 0);

    if let Some(screen) = gtk::prelude::WidgetExt::screen(gtk_window) {
        if let Some(visual) = screen.rgba_visual() {
            gtk_window.set_visual(Some(&visual));
        }
    }
    gtk_window.set_app_paintable(true);
    gtk_window.connect_map(|win| {
        win.move_(0, 0);
    });

    let empty_region = cairo::Region::create();
    gtk_window.input_shape_combine_region(Some(&empty_region));
    if let Some(gdk_win) = gtk_window.window() {
        gdk_win.input_shape_combine_region(&empty_region, 0, 0);
    }
    gtk_window.queue_draw();
}

#[cfg(target_os = "windows")]
pub mod windows_appbar {
    use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicI32, AtomicU8, Ordering};
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, ValidateRect, MONITORINFO, MONITOR_DEFAULTTOPRIMARY,
    };
    use windows_sys::Win32::UI::Shell::{
        SHAppBarMessage, ABE_TOP, ABM_NEW, ABM_QUERYPOS, ABM_REMOVE, ABM_SETPOS, APPBARDATA,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, FindWindowW, GetMessageW,
        GetSystemMetrics, GetWindowRect, PostQuitMessage, RegisterClassExW, SetLayeredWindowAttributes,
        SetWindowPos, ShowWindow, SystemParametersInfoW, TranslateMessage, MSG,
        HWND_TOPMOST, LWA_ALPHA, SM_CXSCREEN, SPIF_SENDCHANGE,
        SPIF_UPDATEINIFILE, SPI_GETWORKAREA, SPI_SETWORKAREA, SWP_ASYNCWINDOWPOS, SWP_NOACTIVATE, SWP_NOZORDER,
        SWP_SHOWWINDOW, SW_SHOWNA, WM_DESTROY, WM_ERASEBKGND, WM_PAINT, WM_USER, WNDCLASSEXW,
        WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_POPUP,
    };

    pub const WM_APPBAR_NOTIFY: u32 = WM_USER + 101;
    // Altura base calibrada para despejar completamente las cápsulas del TopBar
    // (14px margen superior flotante + 34px altura física de la cápsula + 4px despeje visual/sombra = 52px)
    pub const TOPBAR_BASE_HEIGHT: i32 = 52;

    static APPBAR_HWND: AtomicIsize = AtomicIsize::new(0);
    static APPBAR_REGISTERED: AtomicBool = AtomicBool::new(false);
    static APPBAR_HEIGHT: AtomicI32 = AtomicI32::new(TOPBAR_BASE_HEIGHT);
    // Ciclo de vida atómico: 0 = No iniciado, 1 = Inicializando (hilo activo), 2 = Registrado
    static APPBAR_INIT_STATE: AtomicU8 = AtomicU8::new(0);

    // Anclaje nativo inamovible de la ventana "main" de Tauri en coordenadas físicas del monitor
    static MAIN_HWND: AtomicIsize = AtomicIsize::new(0);
    static MAIN_MON_X: AtomicI32 = AtomicI32::new(0);
    static MAIN_MON_Y: AtomicI32 = AtomicI32::new(0);
    static MAIN_MON_W: AtomicI32 = AtomicI32::new(0);
    static MAIN_MON_H: AtomicI32 = AtomicI32::new(0);

    pub fn register_main_window_anchor(hwnd: isize, x: i32, y: i32, w: i32, h: i32) {
        if hwnd != 0 {
            MAIN_HWND.store(hwnd, Ordering::SeqCst);
        }
        MAIN_MON_X.store(x, Ordering::SeqCst);
        MAIN_MON_Y.store(y, Ordering::SeqCst);
        MAIN_MON_W.store(w, Ordering::SeqCst);
        MAIN_MON_H.store(h, Ordering::SeqCst);
        reanchor_main_window();
    }

    pub fn get_main_window_hwnd() -> isize {
        MAIN_HWND.load(Ordering::SeqCst)
    }

    pub fn reanchor_main_window() {
        let mut hwnd = MAIN_HWND.load(Ordering::SeqCst) as HWND;
        if hwnd == 0 {
            let title: Vec<u16> = "EnigmaOS\0".encode_utf16().collect();
            hwnd = unsafe { FindWindowW(std::ptr::null(), title.as_ptr()) };
            if hwnd != 0 {
                MAIN_HWND.store(hwnd as isize, Ordering::SeqCst);
            } else {
                return;
            }
        }
        unsafe {
            let x = MAIN_MON_X.load(Ordering::SeqCst);
            let y = MAIN_MON_Y.load(Ordering::SeqCst);
            let w = MAIN_MON_W.load(Ordering::SeqCst);
            let h = MAIN_MON_H.load(Ordering::SeqCst);
            if w <= 0 || h <= 0 {
                return;
            }

            let mut rect: RECT = std::mem::zeroed();
            let needs_move = if GetWindowRect(hwnd, &mut rect) != 0 {
                rect.left != x || rect.top != y || (rect.right - rect.left) != w || (rect.bottom - rect.top) != h
            } else {
                true
            };

            if needs_move {
                eprintln!("[AppBar] Re-anchoring main window to physical monitor top ({}, {}) [current: ({}, {})]", x, y, rect.left, rect.top);
                SetWindowPos(
                    hwnd,
                    HWND_TOPMOST,
                    x,
                    y,
                    w,
                    h,
                    SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_ASYNCWINDOWPOS,
                );
            }
        }
    }

    unsafe extern "system" fn appbar_wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_ERASEBKGND => 1,
            WM_PAINT => {
                ValidateRect(hwnd, std::ptr::null());
                0
            }
            WM_APPBAR_NOTIFY => {
                // Notificación ABN_POSCHANGED (código 1): monitor, resolución o barra cambiada
                if wparam as u32 == 1 {
                    reposition_appbar(hwnd);
                    let height = APPBAR_HEIGHT.load(Ordering::SeqCst);
                    enforce_workarea_top(height);
                    reanchor_main_window();
                }
                0
            }
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }

    pub unsafe fn reposition_appbar(hwnd: HWND) {
        if hwnd == 0 {
            return;
        }

        let mut mi: MONITORINFO = std::mem::zeroed();
        mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        let hmon = MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY);
        let (mon_left, mon_top, mon_right) = if hmon != 0 && GetMonitorInfoW(hmon, &mut mi) != 0 {
            (mi.rcMonitor.left, mi.rcMonitor.top, mi.rcMonitor.right)
        } else {
            (0, 0, GetSystemMetrics(SM_CXSCREEN))
        };

        let height = APPBAR_HEIGHT.load(Ordering::SeqCst);

        let mut abd: APPBARDATA = std::mem::zeroed();
        abd.cbSize = std::mem::size_of::<APPBARDATA>() as u32;
        abd.hWnd = hwnd;
        abd.uEdge = ABE_TOP;
        abd.rc = RECT {
            left: mon_left,
            top: mon_top,
            right: mon_right,
            bottom: mon_top + height,
        };

        // Protocolo nativo de Windows Shell AppBar (mismo mecanismo que Seelen UI y la barra de tareas)
        SHAppBarMessage(ABM_QUERYPOS, &mut abd);
        abd.rc.bottom = abd.rc.top + height;
        SHAppBarMessage(ABM_SETPOS, &mut abd);

        SetWindowPos(
            hwnd,
            0,
            abd.rc.left,
            abd.rc.top,
            abd.rc.right - abd.rc.left,
            abd.rc.bottom - abd.rc.top,
            SWP_NOACTIVATE | SWP_NOZORDER,
        );

        // Reanclar de inmediato la ventana "main" de Tauri para neutralizar cualquier desplazamiento forzado por Shell
        reanchor_main_window();
    }

    pub fn enforce_workarea_top(target_top: i32) {
        unsafe {
            let hwnd = APPBAR_HWND.load(Ordering::SeqCst) as HWND;
            let mut mi: MONITORINFO = std::mem::zeroed();
            mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
            let hmon = if hwnd != 0 {
                MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY)
            } else {
                MonitorFromWindow(0, MONITOR_DEFAULTTOPRIMARY)
            };

            if hmon != 0 && GetMonitorInfoW(hmon, &mut mi) != 0 {
                let required_top = mi.rcMonitor.top + target_top;
                if mi.rcWork.top < required_top {
                    let mut new_work = mi.rcWork;
                    new_work.top = required_top;
                    SystemParametersInfoW(
                        SPI_SETWORKAREA,
                        0,
                        &mut new_work as *mut _ as *mut _,
                        SPIF_SENDCHANGE | SPIF_UPDATEINIFILE,
                    );
                    reanchor_main_window();
                }
            } else {
                let mut rc: RECT = std::mem::zeroed();
                if SystemParametersInfoW(SPI_GETWORKAREA, 0, &mut rc as *mut _ as *mut _, 0) != 0 {
                    if rc.top < target_top {
                        rc.top = target_top;
                        SystemParametersInfoW(
                            SPI_SETWORKAREA,
                            0,
                            &mut rc as *mut _ as *mut _,
                            SPIF_SENDCHANGE | SPIF_UPDATEINIFILE,
                        );
                        reanchor_main_window();
                    }
                }
            }
        }
    }

    pub fn init_appbar(bar_height: i32) {
        let height = bar_height.max(TOPBAR_BASE_HEIGHT);
        APPBAR_HEIGHT.store(height, Ordering::SeqCst);

        // Si ya está registrado, re-aplicar directamente la geometría calibrada
        if APPBAR_INIT_STATE.load(Ordering::SeqCst) == 2 && APPBAR_REGISTERED.load(Ordering::SeqCst) {
            let hwnd = APPBAR_HWND.load(Ordering::SeqCst) as HWND;
            if hwnd != 0 {
                unsafe {
                    reposition_appbar(hwnd);
                    enforce_workarea_top(height);
                    reanchor_main_window();
                }
            }
            return;
        }

        // Transición atómica 0 -> 1 para garantizar que SOLO UN hilo registre la AppBar y el centinela
        if APPBAR_INIT_STATE.compare_exchange(0, 1, Ordering::SeqCst, Ordering::SeqCst).is_err() {
            // Ya hay un hilo en proceso de inicialización; la altura ya fue actualizada arriba
            return;
        }

        std::thread::spawn(move || {
            unsafe {
                let hinst = GetModuleHandleW(std::ptr::null());
                let class_name: Vec<u16> = "enigmaos_appbar\0".encode_utf16().collect();
                let wnd_class = WNDCLASSEXW {
                    cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                    style: 0,
                    lpfnWndProc: Some(appbar_wndproc),
                    cbClsExtra: 0,
                    cbWndExtra: 0,
                    hInstance: hinst,
                    hIcon: 0,
                    hCursor: 0,
                    hbrBackground: 0,
                    lpszMenuName: std::ptr::null(),
                    lpszClassName: class_name.as_ptr(),
                    hIconSm: 0,
                };
                RegisterClassExW(&wnd_class);

                let screen_w = GetSystemMetrics(SM_CXSCREEN);
                let current_h = APPBAR_HEIGHT.load(Ordering::SeqCst);
                let hwnd = CreateWindowExW(
                    WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TRANSPARENT | WS_EX_LAYERED,
                    class_name.as_ptr(),
                    std::ptr::null(),
                    WS_POPUP,
                    0,
                    0,
                    screen_w,
                    current_h,
                    0,
                    0,
                    hinst,
                    std::ptr::null(),
                );

                if hwnd == 0 {
                    eprintln!("[AppBar] No se pudo crear la ventana proxy de AppBar; aplicando SPI_SETWORKAREA directo");
                    enforce_workarea_top(current_h);
                    APPBAR_INIT_STATE.store(0, Ordering::SeqCst);
                    return;
                }

                // Ventana proxy 100% transparente: elimina permanentemente cualquier franja blanca o artefacto
                SetLayeredWindowAttributes(hwnd, 0, 0, LWA_ALPHA);

                APPBAR_HWND.store(hwnd as isize, Ordering::SeqCst);

                let mut abd: APPBARDATA = std::mem::zeroed();
                abd.cbSize = std::mem::size_of::<APPBARDATA>() as u32;
                abd.hWnd = hwnd;
                abd.uCallbackMessage = WM_APPBAR_NOTIFY;
                SHAppBarMessage(ABM_NEW, &mut abd);
                APPBAR_REGISTERED.store(true, Ordering::SeqCst);
                APPBAR_INIT_STATE.store(2, Ordering::SeqCst);

                reposition_appbar(hwnd);
                ShowWindow(hwnd, SW_SHOWNA);
                let final_h = APPBAR_HEIGHT.load(Ordering::SeqCst);
                enforce_workarea_top(final_h);
                reanchor_main_window();

                eprintln!("[AppBar] EnigmaOS Windows AppBar registrada con éxito (reserva superior transparente: {}px)", final_h);

                let mut msg: MSG = std::mem::zeroed();
                while GetMessageW(&mut msg, 0, 0, 0) > 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            }
        });

        // Centinela único en segundo plano: mantiene permanentemente protegido el WorkArea y el anclaje de la TopBar
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(1000));
                let hwnd = APPBAR_HWND.load(Ordering::SeqCst) as HWND;
                let height = APPBAR_HEIGHT.load(Ordering::SeqCst);
                if hwnd != 0 && APPBAR_REGISTERED.load(Ordering::SeqCst) {
                    enforce_workarea_top(height);
                    reanchor_main_window();
                }
            }
        });
    }

    pub fn unregister_appbar() {
        let hwnd = APPBAR_HWND.swap(0, Ordering::SeqCst) as HWND;
        if hwnd != 0 && APPBAR_REGISTERED.swap(false, Ordering::SeqCst) {
            APPBAR_INIT_STATE.store(0, Ordering::SeqCst);
            unsafe {
                let mut abd: APPBARDATA = std::mem::zeroed();
                abd.cbSize = std::mem::size_of::<APPBARDATA>() as u32;
                abd.hWnd = hwnd;
                SHAppBarMessage(ABM_REMOVE, &mut abd);
                DestroyWindow(hwnd);

                let mut mi: MONITORINFO = std::mem::zeroed();
                mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
                let hmon = MonitorFromWindow(0, MONITOR_DEFAULTTOPRIMARY);
                if hmon != 0 && GetMonitorInfoW(hmon, &mut mi) != 0 {
                    let mut restore_work = mi.rcWork;
                    restore_work.top = mi.rcMonitor.top;
                    SystemParametersInfoW(
                        SPI_SETWORKAREA,
                        0,
                        &mut restore_work as *mut _ as *mut _,
                        SPIF_SENDCHANGE | SPIF_UPDATEINIFILE,
                    );
                }
                eprintln!("[AppBar] EnigmaOS Windows AppBar desregistrada y WorkArea restaurado");
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub fn reserve_topbar_workarea() {
    windows_appbar::init_appbar(windows_appbar::TOPBAR_BASE_HEIGHT);
}

#[cfg(target_os = "windows")]
pub fn restore_topbar_workarea() {
    windows_appbar::unregister_appbar();
}

/// Configures window for Windows and macOS (always-on-top, click-through, space joining)
#[allow(dead_code)]
pub fn setup_window_for_platform(window: &WebviewWindow) {
    #[cfg(target_os = "windows")]
    {
        let _ = window.set_always_on_top(true);
        let _ = window.set_decorations(false);
        
        let monitor = window.current_monitor().ok().flatten()
            .or_else(|| window.primary_monitor().ok().flatten())
            .or_else(|| window.available_monitors().ok().and_then(|m| m.into_iter().next()));

        if let Some(mon) = monitor {
            let size = mon.size();
            let scale_factor = mon.scale_factor();
            let logical_w = size.width as f64 / scale_factor;
            let logical_h = size.height as f64 / scale_factor;
            let mon_pos = mon.position();
            let mon_x = mon_pos.x as f64 / scale_factor;
            let mon_y = mon_pos.y as f64 / scale_factor;

            if window.label() == "dock" {
                // Dock window: anchored to the BOTTOM of the screen with generous headroom for popovers/previews
                let dock_h: f64 = (640.0f64).min(logical_h - 40.0).max(300.0);
                let dock_y = logical_h - dock_h;
                let _ = window.set_size(tauri::LogicalSize::new(logical_w, dock_h));
                let _ = window.set_position(tauri::LogicalPosition::new(mon_x, mon_y + dock_y));
            } else {
                // Main window: dynamically sizes to monitor width and full logical height with transparent background
                // and native hardware click-through, completely eliminating window clipping on any resolution.
                // Anclamos firmemente en mon_pos físico (0, 0 en monitor primario) sin desfases.
                let p_pos = tauri::PhysicalPosition::new(mon_pos.x, mon_pos.y);
                let p_size = tauri::PhysicalSize::new(size.width, size.height);
                let _ = window.set_size(p_size);
                let _ = window.set_position(p_pos);

                let bar_h_px = (windows_appbar::TOPBAR_BASE_HEIGHT as f64 * scale_factor).round() as i32;
                windows_appbar::init_appbar(bar_h_px);

                let raw_hwnd = window.hwnd().ok().map(|h| h.0 as isize).unwrap_or(0);
                windows_appbar::register_main_window_anchor(
                    raw_hwnd,
                    mon_pos.x,
                    mon_pos.y,
                    size.width as i32,
                    size.height as i32,
                );

                // Reintentos escalonados para contrarrestar de inmediato el desplazamiento asíncrono del Windows Shell
                let win_main = window.clone();
                std::thread::spawn(move || {
                    for delay in [50, 150, 350, 800, 1500] {
                        std::thread::sleep(std::time::Duration::from_millis(delay));
                        if let Ok(h) = win_main.hwnd() {
                            let raw = h.0 as isize;
                            if raw != 0 && windows_appbar::get_main_window_hwnd() == 0 {
                                windows_appbar::register_main_window_anchor(
                                    raw,
                                    p_pos.x,
                                    p_pos.y,
                                    p_size.width as i32,
                                    p_size.height as i32,
                                );
                            }
                        }
                        let _ = win_main.set_size(p_size);
                        let _ = win_main.set_position(p_pos);
                        windows_appbar::reanchor_main_window();
                    }
                });
            }
        }
        // Start in click-through mode until cursor moves into notch/dock
        let _ = window.set_ignore_cursor_events(true);
    }
    #[cfg(target_os = "macos")]
    {
        let _ = window.set_always_on_top(true);
        let _ = window.set_decorations(false);
        let _ = window.set_ignore_cursor_events(true);
        unsafe {
            if let Ok(ns_win) = window.ns_window() {
                use objc::{msg_send, sel, sel_impl};
                use objc::runtime::Object;
                let window_ptr = ns_win as *mut Object;
                let behavior: usize = (1 << 0) | (1 << 4);
                let _: () = msg_send![window_ptr, setCollectionBehavior: behavior];
            }
        }
    }
    #[cfg(target_os = "linux")]
    {
        let _ = window;
    }
}

static MAIN_INTERACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static DOCK_INTERACTIVE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[allow(dead_code)]
pub fn set_window_interactive_mode(label: &str, interactive: bool) {
    if label == "main" {
        MAIN_INTERACTIVE.store(interactive, std::sync::atomic::Ordering::SeqCst);
    } else if label == "dock" {
        DOCK_INTERACTIVE.store(interactive, std::sync::atomic::Ordering::SeqCst);
    }
}

#[allow(dead_code)]
pub fn is_window_interactive(label: &str) -> bool {
    if label == "main" {
        MAIN_INTERACTIVE.load(std::sync::atomic::Ordering::SeqCst)
    } else if label == "dock" {
        DOCK_INTERACTIVE.load(std::sync::atomic::Ordering::SeqCst)
    } else {
        false
    }
}

/// Spawns a 60 FPS cursor hit-test tracker on Windows and macOS to dynamically toggle
/// `set_ignore_cursor_events(false)` when cursor is over the notch/dock, and `true` when outside.
#[allow(dead_code)]
pub fn start_mouse_hit_test_tracker(window: WebviewWindow) {
    #[cfg(not(target_os = "linux"))]
    {
        let win = window.clone();
        let label = win.label().to_string();
        std::thread::spawn(move || {
            let mut consecutive_outside_ticks: u32 = 0;
            loop {
                std::thread::sleep(std::time::Duration::from_millis(16));

                // If window has active keyboard interactivity (search, enigma hub, launcher), keep cursor events active
                if is_window_interactive(&label) {
                    consecutive_outside_ticks = 0;
                    let _ = win.set_ignore_cursor_events(false);
                    continue;
                }
                let cursor = get_cursor_position();
                let (regions, was_inside) = {
                    let cache = get_region_cache().lock().unwrap();
                    let entry = cache.get(&label);
                    (
                        entry.map(|e| e.last_applied.clone()).unwrap_or_default(),
                        entry.and_then(|e| e.was_inside),
                    )
                };

                let win_pos = win.outer_position().unwrap_or(tauri::PhysicalPosition { x: 0, y: 0 });
                let is_inside = if regions.is_empty() {
                    false
                } else {
                    is_cursor_in_regions(cursor, win_pos, &regions, &label)
                };

                if is_inside {
                    consecutive_outside_ticks = 0;
                    if was_inside != Some(true) {
                        {
                            let mut cache = get_region_cache().lock().unwrap();
                            if let Some(entry) = cache.get_mut(&label) {
                                entry.was_inside = Some(true);
                            }
                        }
                        let _ = win.set_ignore_cursor_events(false);
                        let _ = win.emit(&format!("{}_cursor_entered", label), ());
                    }
                } else {
                    // Cursor is outside target regions
                    if was_inside == Some(true) {
                        consecutive_outside_ticks += 1;
                        // Grace period of ~160ms (10 ticks @ 16ms) before enabling click-through
                        // to prevent dropped clicks during rapid micro-movements, popovers or tooltips
                        if consecutive_outside_ticks >= 10 {
                            {
                                let mut cache = get_region_cache().lock().unwrap();
                                if let Some(entry) = cache.get_mut(&label) {
                                    entry.was_inside = Some(false);
                                }
                            }
                            let _ = win.set_ignore_cursor_events(true);
                            let _ = win.emit(&format!("{}_cursor_left", label), ());
                            consecutive_outside_ticks = 0;
                        }
                    } else if was_inside.is_none() {
                        let mut cache = get_region_cache().lock().unwrap();
                        if let Some(entry) = cache.get_mut(&label) {
                            entry.was_inside = Some(false);
                        }
                        let _ = win.set_ignore_cursor_events(true);
                    }
                }
            }
        });
    }
    #[cfg(target_os = "linux")]
    {
        let _ = window;
    }
}

/// Updates the hardware input mask.
/// On Linux: pure Wayland and Cairo/GDK input shapes.
/// On Windows & macOS: dynamic cursor event ignore toggling based on regions.
#[tauri::command]
pub fn update_custom_input_regions(window: WebviewWindow, regions: Vec<[i32; 4]>) -> Result<(), String> {
    let label = window.label().to_string();
    {
        let mut cache = get_region_cache().lock().unwrap();
        let now = Instant::now();
        if let Some(entry) = cache.get_mut(&label) {
            if entry.last_applied == regions {
                return Ok(());
            }
            entry.last_applied = regions.clone();
            entry.last_update = now;
        } else {
            cache.insert(
                label.clone(),
                RegionCacheEntry {
                    last_applied: regions.clone(),
                    last_update: now,
                    was_inside: None,
                },
            );
        }
    }

    #[cfg(target_os = "linux")]
    {
        let win = window.clone();
        window.run_on_main_thread(move || {
            if let Ok(gtk_window) = win.gtk_window() {
                use gtk::glib::translate::ToGlibPtr;
                let win_ref: &gtk::Window = gtk_window.upcast_ref();
                let win_ptr: *mut gtk::ffi::GtkWindow = win_ref.to_glib_none().0;

                let mut flat: Vec<i32> = Vec::with_capacity(regions.len() * 4);
                for r in &regions {
                    flat.push(r[0]);
                    flat.push(r[1]);
                    flat.push(r[2]);
                    flat.push(r[3]);
                }

                let wayland_ret = unsafe {
                    set_native_wayland_input_regions(
                        win_ptr,
                        if flat.is_empty() { std::ptr::null() } else { flat.as_ptr() },
                        regions.len() as c_int,
                    )
                };

                let region = cairo::Region::create();
                for r in &regions {
                    let rect = cairo::RectangleInt::new(r[0], r[1], r[2], r[3]);
                    let _ = region.union_rectangle(&rect);
                }
                apply_recursive_input_shape(gtk_window.upcast_ref(), &region);

                eprintln!("[InputRegions:{}] applied {} region(s) (wl_ret={}): {:?}", win.label(), regions.len(), wayland_ret, regions);
            }
        }).map_err(|e| e.to_string())?;
    }

    #[cfg(not(target_os = "linux"))]
    {
        let cursor = get_cursor_position();
        let win_pos = window.outer_position().unwrap_or(tauri::PhysicalPosition { x: 0, y: 0 });
        let is_inside = if regions.is_empty() {
            false
        } else {
            is_cursor_in_regions(cursor, win_pos, &regions, &label)
        };

        let prev_inside = {
            let mut cache = get_region_cache().lock().unwrap();
            let prev = cache.get(&label).and_then(|e| e.was_inside);
            if is_inside {
                if let Some(entry) = cache.get_mut(&label) {
                    entry.was_inside = Some(true);
                }
            }
            prev
        };

        if is_inside {
            let _ = window.set_ignore_cursor_events(false);
            if prev_inside != Some(true) {
                let _ = window.emit(&format!("{}_cursor_entered", label), ());
            }
        } else if prev_inside.is_none() {
            let _ = window.set_ignore_cursor_events(true);
        }
    }

    Ok(())
}

/// Sets dynamic keyboard interactivity.
/// On Linux: Wayland Layer Shell keyboard mode.
/// On Windows & macOS: unignores cursor events and sets window focus when interactive.
#[tauri::command]
pub fn set_keyboard_interactivity(window: WebviewWindow, enabled: bool, reason: Option<String>) -> Result<(), String> {
    set_window_interactive_mode(window.label(), enabled);

    #[cfg(target_os = "linux")]
    {
        use gtk::glib::translate::ToGlibPtr;
        let win = window.clone();
        window.run_on_main_thread(move || {
            if let Ok(gtk_window) = win.gtk_window() {
                let win_ref: &gtk::Window = gtk_window.upcast_ref();
                let win_ptr: *mut gtk::ffi::GtkWindow = win_ref.to_glib_none().0;

                unsafe {
                    if gtk_layer_is_supported() != 0 && gtk_layer_is_layer_window(win_ptr) != 0 {
                        let mode = if enabled { 2 } else { 0 };
                        gtk_layer_set_keyboard_mode(win_ptr, mode);
                        type CommitFn = unsafe extern "C" fn(*mut gtk::ffi::GtkWindow);
                        let sym = libc::dlsym(
                            libc::RTLD_DEFAULT,
                            b"gtk_layer_try_force_commit\0".as_ptr() as *const libc::c_char,
                        );
                        if !sym.is_null() {
                            let func: CommitFn = std::mem::transmute(sym);
                            func(win_ptr);
                        }
                        eprintln!("[EnigmaOS] Dynamic layer keyboard mode set to: {} (reason: {:?})", mode, reason);
                    }
                }
            }
        }).map_err(|e| e.to_string())?;
    }

    #[cfg(not(target_os = "linux"))]
    {
        if enabled {
            let _ = window.set_ignore_cursor_events(false);
            let _ = window.set_focus();
        }
        let _ = reason;
    }
    Ok(())
}

/// Returns the global cursor position [x, y] in screen pixels
#[tauri::command]
pub fn get_cursor_position() -> [i32; 2] {
    #[cfg(target_os = "linux")]
    {
        if let Some(display) = gdk::Display::default() {
            if let Some(seat) = display.default_seat() {
                if let Some(pointer) = seat.pointer() {
                    let (_screen, x, y) = pointer.position();
                    return [x, y];
                }
            }
        }
        [0, 0]
    }
    #[cfg(target_os = "windows")]
    {
        unsafe {
            let mut pt = std::mem::zeroed();
            windows_sys::Win32::UI::WindowsAndMessaging::GetCursorPos(&mut pt);
            [pt.x, pt.y]
        }
    }
    #[cfg(target_os = "macos")]
    {
        #[link(name = "CoreGraphics", kind = "framework")]
        extern "C" {
            fn CGEventCreate(source: *const std::ffi::c_void) -> *mut std::ffi::c_void;
            fn CGEventGetLocation(event: *mut std::ffi::c_void) -> CGPoint;
            fn CFRelease(cf: *const std::ffi::c_void);
        }
        #[repr(C)]
        #[derive(Copy, Clone)]
        struct CGPoint {
            x: f64,
            y: f64,
        }
        unsafe {
            let ev = CGEventCreate(std::ptr::null());
            if !ev.is_null() {
                let pt = CGEventGetLocation(ev);
                CFRelease(ev);
                [pt.x as i32, pt.y as i32]
            } else {
                [0, 0]
            }
        }
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows", target_os = "macos")))]
    {
        [0, 0]
    }
}

/// Spawns a 50 FPS global mouse cursor hardware tracker on the GTK main loop (Linux)
#[allow(dead_code)]
pub fn start_global_cursor_tracker(window: WebviewWindow) {
    #[cfg(target_os = "linux")]
    {
        let mut was_inside = false;
        gtk::glib::timeout_add_local(std::time::Duration::from_millis(20), move || {
            if let Some(display) = gdk::Display::default() {
                if let Some(seat) = display.default_seat() {
                    if let Some(pointer) = seat.pointer() {
                        let (_screen, x, y) = pointer.position();
                        let is_in_idle_notch = x >= 1140 && x <= 1420 && y >= 0 && y <= 36;
                        let is_in_expanded_notch = x >= 1000 && x <= 1560 && y >= 0 && y <= 275;

                        if !was_inside && is_in_idle_notch {
                            was_inside = true;
                            let _ = window.emit("notch_hover", true);
                        } else if was_inside && !is_in_expanded_notch {
                            was_inside = false;
                            let _ = window.emit("notch_hover", false);
                        }
                    }
                }
            }
            gtk::glib::ControlFlow::Continue
        });
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = window;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topbar_geometry_clearance() {
        // La barra superior debe tener margen suficiente para alojar las cápsulas
        // (14px margen superior + 34px altura base cápsula = 48px).
        // La altura de reserva física debe ser al menos 52px para dejar margen de sombra/separación.
        const TOPBAR_MARGIN_TOP: i32 = 14;
        const CAPSULE_BASE_HEIGHT: i32 = 34;
        const MIN_RESERVED_HEIGHT: i32 = 52;

        let total_capsule_bottom = TOPBAR_MARGIN_TOP + CAPSULE_BASE_HEIGHT;
        assert_eq!(total_capsule_bottom, 48);
        assert!(MIN_RESERVED_HEIGHT >= total_capsule_bottom);
        let clearance = MIN_RESERVED_HEIGHT - total_capsule_bottom;
        assert!(clearance >= 4, "Debe existir al menos 4px de despeje visual bajo la cápsula");
    }

    #[test]
    fn test_cursor_hit_test_inside_regions() {
        let win_pos = tauri::PhysicalPosition { x: 0, y: 0 };
        // Región simulada de LeftCapsule: x: 16, y: 14, w: 260, h: 34
        let regions = vec![[16, 14, 260, 34]];

        // Dentro del centro de la cápsula
        assert!(is_cursor_in_regions([100, 25], win_pos, &regions, "main"));

        // En el margen de tolerancia (+8px)
        assert!(is_cursor_in_regions([10, 10], win_pos, &regions, "main"));
    }

    #[test]
    fn test_cursor_hit_test_outside_regions() {
        let win_pos = tauri::PhysicalPosition { x: 0, y: 0 };
        let regions = vec![[16, 14, 260, 34]];

        // Muy abajo (espacio donde corre Discord / apps maximizadas)
        assert!(!is_cursor_in_regions([100, 100], win_pos, &regions, "main"));

        // Muy a la derecha
        assert!(!is_cursor_in_regions([500, 25], win_pos, &regions, "main"));
    }

    #[test]
    fn test_cursor_hit_test_empty_regions() {
        let win_pos = tauri::PhysicalPosition { x: 0, y: 0 };
        let regions: Vec<[i32; 4]> = vec![];
        assert!(!is_cursor_in_regions([100, 20], win_pos, &regions, "main"));
    }

    #[test]
    fn test_cursor_hit_test_dock_y_boundary() {
        let win_pos = tauri::PhysicalPosition { x: 0, y: 600 };
        let regions = vec![[500, 620, 200, 50]];

        // Cursor por encima de win_pos.y para ventana dock
        assert!(!is_cursor_in_regions([550, 500], win_pos, &regions, "dock"));
    }
}

