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
}

fn get_region_cache() -> &'static Mutex<HashMap<String, RegionCacheEntry>> {
    static CACHE: std::sync::OnceLock<Mutex<HashMap<String, RegionCacheEntry>>> = std::sync::OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
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

                gtk_window.set_default_size(2560, 900);
                gtk_window.resize(2560, 900);
                gtk::prelude::WidgetExt::set_size_request(gtk_window, 2560, 900);

                gtk_layer_set_exclusive_zone(win_ptr, 36);
                gtk_layer_set_keyboard_mode(win_ptr, 0);
                eprintln!("[EnigmaOS] Native Wayland Layer Shell initialized (layer=TOP, exclusive_zone=36, size=2560x900, top_margin=0)");
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
pub fn reserve_topbar_workarea() {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPI_GETWORKAREA, SPI_SETWORKAREA, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE,
    };
    unsafe {
        let mut rc: RECT = std::mem::zeroed();
        if SystemParametersInfoW(SPI_GETWORKAREA, 0, &mut rc as *mut _ as *mut _, 0) != 0 {
            if rc.top < 38 {
                rc.top = 38;
                SystemParametersInfoW(
                    SPI_SETWORKAREA,
                    0,
                    &mut rc as *mut _ as *mut _,
                    SPIF_SENDCHANGE | SPIF_UPDATEINIFILE,
                );
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub fn restore_topbar_workarea() {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPI_GETWORKAREA, SPI_SETWORKAREA, SPIF_SENDCHANGE, SPIF_UPDATEINIFILE,
    };
    unsafe {
        let mut rc: RECT = std::mem::zeroed();
        if SystemParametersInfoW(SPI_GETWORKAREA, 0, &mut rc as *mut _ as *mut _, 0) != 0 {
            if rc.top == 38 {
                rc.top = 0;
                SystemParametersInfoW(
                    SPI_SETWORKAREA,
                    0,
                    &mut rc as *mut _ as *mut _,
                    SPIF_SENDCHANGE | SPIF_UPDATEINIFILE,
                );
            }
        }
    }
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
            .or_else(|| window.available_monitors().ok().and_then(|mut m| m.next()));

        if let Some(mon) = monitor {
            let size = mon.size();
            let scale_factor = mon.scale_factor();
            let logical_w = size.width as f64 / scale_factor;
            let logical_h = size.height as f64 / scale_factor;

            if window.label() == "dock" {
                // Dock window: anchored to the BOTTOM of the screen
                let dock_h: f64 = (580.0f64).min(logical_h - 40.0);
                let dock_y = logical_h - dock_h;
                let _ = window.set_size(tauri::LogicalSize::new(logical_w, dock_h));
                let _ = window.set_position(tauri::LogicalPosition::new(0.0, dock_y));
            } else {
                // Main window: anchored to the TOP of the screen
                let main_h: f64 = (500.0f64).min(logical_h - 100.0);
                let _ = window.set_size(tauri::LogicalSize::new(logical_w, main_h));
                let _ = window.set_position(tauri::LogicalPosition::new(0.0, 0.0));
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

/// Spawns a 60 FPS cursor hit-test tracker on Windows and macOS to dynamically toggle
/// `set_ignore_cursor_events(false)` when cursor is over the notch/dock, and `true` when outside.
#[allow(dead_code)]
pub fn start_mouse_hit_test_tracker(window: WebviewWindow) {
    #[cfg(not(target_os = "linux"))]
    {
        let win = window.clone();
        let label = win.label().to_string();
        std::thread::spawn(move || {
            let mut was_inside: Option<bool> = None;
            loop {
                std::thread::sleep(std::time::Duration::from_millis(16));
                let cursor = get_cursor_position();
                let regions = {
                    let cache = get_region_cache().lock().unwrap();
                    cache.get(&label).map(|e| e.last_applied.clone()).unwrap_or_default()
                };

                if regions.is_empty() {
                    if was_inside != Some(false) {
                        was_inside = Some(false);
                        let _ = win.set_ignore_cursor_events(true);
                        let _ = win.emit(&format!("{}_cursor_left", label), ());
                    }
                    continue;
                }

                // Window position in physical screen coordinates
                let win_pos = win.outer_position().unwrap_or(tauri::PhysicalPosition { x: 0, y: 0 });
                let rel_x = cursor[0] - win_pos.x;
                let rel_y = cursor[1] - win_pos.y;

                let mut is_inside = false;
                for r in &regions {
                    let rx = r[0];
                    let ry = r[1];
                    let rw = r[2];
                    let rh = r[3];
                    if rel_x >= rx && rel_x <= rx + rw && rel_y >= ry && rel_y <= ry + rh {
                        is_inside = true;
                        break;
                    }
                }

                // Crosstalk isolation between notch and dock:
                if label == "dock" && cursor[1] < win_pos.y {
                    is_inside = false;
                }
                if label == "main" && rel_y > 520 {
                    is_inside = false;
                }

                if was_inside != Some(is_inside) {
                    was_inside = Some(is_inside);
                    let should_ignore = !is_inside;
                    let _ = win.set_ignore_cursor_events(should_ignore);
                    if is_inside {
                        let _ = win.emit(&format!("{}_cursor_entered", label), ());
                    } else {
                        let _ = win.emit(&format!("{}_cursor_left", label), ());
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
        if regions.is_empty() {
            let _ = window.set_ignore_cursor_events(true);
        } else {
            let cursor = get_cursor_position();
            let mut inside = false;
            for r in &regions {
                let rx = r[0];
                let ry = r[1];
                let rw = r[2];
                let rh = r[3];
                if cursor[0] >= rx && cursor[0] <= rx + rw && cursor[1] >= ry && cursor[1] <= ry + rh {
                    inside = true;
                    break;
                }
            }
            let _ = window.set_ignore_cursor_events(!inside);
        }
    }

    Ok(())
}

/// Sets dynamic keyboard interactivity.
/// On Linux: Wayland Layer Shell keyboard mode.
/// On Windows & macOS: unignores cursor events and sets window focus when interactive.
#[tauri::command]
pub fn set_keyboard_interactivity(window: WebviewWindow, enabled: bool, reason: Option<String>) -> Result<(), String> {
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
