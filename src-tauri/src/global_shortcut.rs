//! Global keyboard shortcut monitor (evdev on Linux, tauri-plugin-global-shortcut cross-platform)

#[cfg(target_os = "linux")]
mod linux_shortcuts {
    use evdev::Key;
    use std::time::Duration;
    use tauri::{AppHandle, Emitter, Manager};

    pub fn start_global_shortcut_listener(app_handle: AppHandle) {
        std::thread::spawn(move || {
            loop {
                // Find keyboard devices that support both KEY_LEFTALT and KEY_SPACE
                let mut devices = Vec::new();
                for (_path, dev) in evdev::enumerate() {
                    if let Some(keys) = dev.supported_keys() {
                        if keys.contains(Key::KEY_LEFTALT) && keys.contains(Key::KEY_SPACE) {
                            devices.push(dev);
                        }
                    }
                }

                if devices.is_empty() {
                    eprintln!("[GlobalShortcut] evdev: No accessible keyboard in /dev/input/event*. (Run 'sudo usermod -aG input $USER' for direct evdev access)");
                    std::thread::sleep(Duration::from_millis(3000));
                    continue;
                } else {
                    eprintln!("[GlobalShortcut] evdev: Monitoring {} physical keyboard device(s)", devices.len());
                }

                // Spawn a monitor thread for each keyboard
                let mut handles = Vec::new();
                for mut dev in devices {
                    let app = app_handle.clone();
                    let handle = std::thread::spawn(move || {
                        let mut alt_pressed = false;
                        let mut ctrl_pressed = false;
                        let mut meta_pressed = false;
                        let mut meta_combined = false;
                        let mut last_search_time = std::time::Instant::now() - Duration::from_secs(1);

                        loop {
                            match dev.fetch_events() {
                                Ok(events) => {
                                    for ev in events {
                                        if ev.event_type() == evdev::EventType::KEY {
                                            let code = ev.code();
                                            let value = ev.value(); // 0 = release, 1 = press, 2 = repeat

                                            if code == Key::KEY_LEFTALT.code() || code == Key::KEY_RIGHTALT.code() {
                                                 alt_pressed = value > 0;
                                            } else if code == Key::KEY_LEFTCTRL.code() || code == Key::KEY_RIGHTCTRL.code() {
                                                 ctrl_pressed = value > 0;
                                            } else if code == Key::KEY_LEFTMETA.code() || code == Key::KEY_RIGHTMETA.code() {
                                                if value == 1 {
                                                    meta_pressed = true;
                                                    meta_combined = false;
                                                } else if value == 0 {
                                                    if meta_pressed && !meta_combined {
                                                        eprintln!("[GlobalShortcut] evdev detected Super/Windows key pressed alone! Emitting toggle-launcher");
                                                        if let Some(dock_win) = app.get_webview_window("dock") {
                                                            let _ = crate::window_strut::set_keyboard_interactivity(dock_win.clone(), true, Some("evdev_win_key".into()));
                                                            let _ = dock_win.set_focus();
                                                        }
                                                        let _ = app.emit("toggle-launcher", ());
                                                    }
                                                    meta_pressed = false;
                                                    meta_combined = false;
                                                }
                                            } else if meta_pressed && value > 0 {
                                                meta_combined = true;
                                            } else if code == Key::KEY_SPACE.code() {
                                                if alt_pressed && value == 1 {
                                                    if last_search_time.elapsed() >= Duration::from_millis(250) {
                                                        last_search_time = std::time::Instant::now();
                                                        eprintln!("[GlobalShortcut] evdev detected ALT + Space! Emitting toggle-search");
                                                        if let Some(main_win) = app.get_webview_window("main") {
                                                            let _ = crate::window_strut::set_keyboard_interactivity(main_win.clone(), true, Some("evdev_alt_space".into()));
                                                            let _ = main_win.set_focus();
                                                        }
                                                        let _ = app.emit("toggle-search", ());
                                                    }
                                                }
                                            } else if code == Key::KEY_3.code() {
                                                if (alt_pressed || ctrl_pressed) && value == 1 {
                                                    eprintln!("[GlobalShortcut] evdev detected ALT + 3! Emitting toggle-enigma");
                                                    if let Some(main_win) = app.get_webview_window("main") {
                                                        let _ = crate::window_strut::set_keyboard_interactivity(main_win.clone(), true, Some("evdev_alt_3".into()));
                                                        let _ = main_win.set_focus();
                                                    }
                                                    let _ = app.emit("toggle-enigma", ());
                                                }
                                            } else if code == Key::KEY_V.code() {
                                                if alt_pressed && value == 1 {
                                                    eprintln!("[GlobalShortcut] evdev detected ALT + V! Emitting open-clipboard-history");
                                                    if let Some(main_win) = app.get_webview_window("main") {
                                                        let _ = crate::window_strut::set_keyboard_interactivity(main_win.clone(), true, Some("evdev_alt_v".into()));
                                                        let _ = main_win.set_focus();
                                                    }
                                                    let _ = app.emit("open-clipboard-history", ());
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(_) => {
                                    // Device disconnected or error, break and rediscover
                                    break;
                                }
                            }
                        }
                    });
                    handles.push(handle);
                }

                // Wait for threads (if keyboards disconnect)
                for h in handles {
                    let _ = h.join();
                }

                std::thread::sleep(Duration::from_millis(2000));
            }
        });
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_enumerate_keyboards() {
            let mut count = 0;
            for (path, dev) in evdev::enumerate() {
                if let Some(keys) = dev.supported_keys() {
                    if keys.contains(Key::KEY_LEFTALT) && keys.contains(Key::KEY_SPACE) {
                        println!("[TEST] Found physical keyboard device: {:?} -> {:?}", path, dev.name());
                        count += 1;
                    }
                }
            }
            println!("[TEST] Physical keyboard devices with ALT+Space: {}", count);
            assert!(count > 0, "Should find at least one physical keyboard");
        }
    }
}

#[cfg(target_os = "linux")]
pub use linux_shortcuts::start_global_shortcut_listener;

#[cfg(target_os = "windows")]
mod windows_shortcuts {
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::OnceLock;
    use tauri::{AppHandle, Emitter, Manager};
    use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        GetAsyncKeyState, keybd_event, KEYEVENTF_KEYUP, VK_LWIN, VK_RWIN, VK_MENU, VK_LMENU, VK_RMENU, VK_SPACE,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
        UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP,
        WM_SYSKEYDOWN, WM_SYSKEYUP,
    };
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

    static HOOK_APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
    static WIN_KEY_DOWN: AtomicBool = AtomicBool::new(false);
    static WIN_KEY_COMBINED: AtomicBool = AtomicBool::new(false);
    static ALT_KEY_DOWN: AtomicBool = AtomicBool::new(false);
    static LAST_SHORTCUT_TICK: AtomicU64 = AtomicU64::new(0);

    fn current_timestamp_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }

    unsafe extern "system" fn low_level_keyboard_proc(
        n_code: i32,
        w_param: WPARAM,
        l_param: LPARAM,
    ) -> LRESULT {
        if n_code >= 0 {
            let kbd = *(l_param as *const KBDLLHOOKSTRUCT);
            let vk = kbd.vkCode as u16;
            let msg = w_param as u32;
            let is_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
            let is_up = msg == WM_KEYUP || msg == WM_SYSKEYUP;

            // Ignorar teclas inyectadas por el sistema o nuestra propia tecla máscara (0xE8)
            if (kbd.flags & 0x10) != 0 || vk == 0xE8 {
                return CallNextHookEx(0, n_code, w_param, l_param);
            }

            // 1. Rastrear pulsación de teclas ALT (Izquierda, Derecha o Genérica)
            if vk == VK_MENU || vk == VK_LMENU || vk == VK_RMENU {
                ALT_KEY_DOWN.store(is_down, Ordering::SeqCst);
            }

            let alt_is_pressed = (kbd.flags & 0x20) != 0 // LLKHF_ALTDOWN context flag
                || ALT_KEY_DOWN.load(Ordering::SeqCst)
                || (GetAsyncKeyState(VK_MENU as i32) as u16 & 0x8000) != 0;

            // 2. Interceptar Alt + Espacio (Notch Search)
            // Windows reserva normalmente Alt+Espacio para el menú del sistema (SC_KEYMENU);
            // interceptarlo a nivel LL suprime el bloqueo del shell y conmuta la búsqueda al instante.
            if vk == VK_SPACE && alt_is_pressed {
                if is_down {
                    let now = current_timestamp_ms();
                    let last = LAST_SHORTCUT_TICK.load(Ordering::SeqCst);
                    if now.saturating_sub(last) >= 250 {
                        LAST_SHORTCUT_TICK.store(now, Ordering::SeqCst);
                        if let Some(app) = HOOK_APP_HANDLE.get() {
                            eprintln!("[GlobalShortcut:WinHook] Intercepted Alt+Space! Focusing main and toggling Notch Search");
                            if let Some(main_win) = app.get_webview_window("main") {
                                let _ = crate::window_strut::set_keyboard_interactivity(main_win.clone(), true, Some("win_hook_alt_space".into()));
                                let _ = main_win.set_focus();
                            }
                            let _ = app.emit("toggle-search", ());
                        }
                    }
                    return 1; // Suprimir menú nativo de Windows (SC_KEYMENU)
                } else if is_up {
                    return 1;
                }
            }

            // 3. Interceptar Alt + 3 (Enigma AI Hub)
            // 0x33 es el Virtual Key Code Win32 para la tecla '3' (fila superior) y 0x63 para '3' (teclado numérico)
            if (vk == 0x33 || vk == 0x63) && alt_is_pressed {
                if is_down {
                    let now = current_timestamp_ms();
                    let last = LAST_SHORTCUT_TICK.load(Ordering::SeqCst);
                    if now.saturating_sub(last) >= 250 {
                        LAST_SHORTCUT_TICK.store(now, Ordering::SeqCst);
                        if let Some(app) = HOOK_APP_HANDLE.get() {
                            eprintln!("[GlobalShortcut:WinHook] Intercepted Alt+3! Focusing main and toggling Enigma Hub");
                            if let Some(main_win) = app.get_webview_window("main") {
                                let _ = crate::window_strut::set_keyboard_interactivity(main_win.clone(), true, Some("win_hook_alt_3".into()));
                                let _ = main_win.set_focus();
                            }
                            let _ = app.emit("toggle-enigma", ());
                        }
                    }
                    return 1; // Suprimir pitido o evento del sistema
                } else if is_up {
                    return 1;
                }
            }

            // 4. Interceptar Alt + V (Historial de Portapapeles Inteligente)
            // 0x56 es el Virtual Key Code Win32 para la tecla 'V'
            if vk == 0x56 && alt_is_pressed {
                if is_down {
                    let now = current_timestamp_ms();
                    let last = LAST_SHORTCUT_TICK.load(Ordering::SeqCst);
                    if now.saturating_sub(last) >= 250 {
                        LAST_SHORTCUT_TICK.store(now, Ordering::SeqCst);
                        if let Some(app) = HOOK_APP_HANDLE.get() {
                            eprintln!("[GlobalShortcut:WinHook] Intercepted Alt+V! Focusing main and opening Clipboard History");
                            if let Some(main_win) = app.get_webview_window("main") {
                                let _ = crate::window_strut::set_keyboard_interactivity(main_win.clone(), true, Some("win_hook_alt_v".into()));
                                let _ = main_win.set_focus();
                            }
                            let _ = app.emit("open-clipboard-history", ());
                        }
                    }
                    return 1;
                } else if is_up {
                    return 1;
                }
            }

            // 5. Tecla individual de Windows (Win / Super)
            if vk == VK_LWIN || vk == VK_RWIN {
                if is_down {
                    // Solo resetear combined si es una pulsación nueva (no auto-repetición mantenida)
                    if !WIN_KEY_DOWN.swap(true, Ordering::SeqCst) {
                        WIN_KEY_COMBINED.store(false, Ordering::SeqCst);
                    }
                } else if is_up {
                    let was_down = WIN_KEY_DOWN.swap(false, Ordering::SeqCst);
                    let combined = WIN_KEY_COMBINED.swap(false, Ordering::SeqCst);
                    if was_down && !combined {
                        // Inyectar tecla máscara para prevenir la apertura del Menú de Inicio en Windows Shell (explorer.exe)
                        keybd_event(0xE8, 0, 0, 0);
                        keybd_event(0xE8, 0, KEYEVENTF_KEYUP, 0);

                        // El usuario presionó y soltó únicamente la tecla Windows: abrir/conmutar Enigma Launcher
                        if let Some(app) = HOOK_APP_HANDLE.get() {
                            eprintln!("[GlobalShortcut:WinHook] Windows key pressed alone! Toggling Enigma Launcher");
                            if let Some(dock_win) = app.get_webview_window("dock") {
                                let _ = crate::window_strut::set_keyboard_interactivity(dock_win.clone(), true, Some("win_hook_launcher".into()));
                                let _ = dock_win.set_focus();
                            }
                            let _ = app.emit("toggle-launcher", ());
                        }
                        // Suprime el menú de inicio nativo de Windows sin bloquear la tecla
                        return 1;
                    }
                }
            } else if WIN_KEY_DOWN.load(Ordering::SeqCst) {
                // Otra tecla pulsada en combinación (ej: Win+E, Win+R, Win+D, Win+Tab)
                WIN_KEY_COMBINED.store(true, Ordering::SeqCst);
            }
        }

        CallNextHookEx(0, n_code, w_param, l_param)
    }

    pub fn start_global_shortcut_listener(app_handle: AppHandle) {
        let _ = HOOK_APP_HANDLE.set(app_handle);
        std::thread::spawn(|| {
            unsafe {
                let hinstance = GetModuleHandleW(std::ptr::null());
                let hook = SetWindowsHookExW(
                    WH_KEYBOARD_LL,
                    Some(low_level_keyboard_proc),
                    hinstance,
                    0,
                );
                if hook == 0 {
                    eprintln!("[GlobalShortcut:WinHook] Failed to install WH_KEYBOARD_LL hook");
                    return;
                }
                eprintln!("[GlobalShortcut:WinHook] WH_KEYBOARD_LL hook installed successfully for Win, Alt+Space, Alt+3, Alt+V");

                let mut msg: MSG = std::mem::zeroed();
                while GetMessageW(&mut msg, 0, 0, 0) > 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                UnhookWindowsHookEx(hook);
            }
        });
    }
}

#[cfg(target_os = "windows")]
pub use windows_shortcuts::start_global_shortcut_listener;

#[cfg(not(any(target_os = "linux", target_os = "windows")))]
pub fn start_global_shortcut_listener(_app_handle: tauri::AppHandle) {
    // On macOS, shortcuts are handled natively
}
