//! Global keyboard shortcut monitor (evdev on Linux, tauri-plugin-global-shortcut cross-platform)

#[cfg(target_os = "linux")]
mod linux_shortcuts {
    use evdev::Key;
    use std::time::Duration;
    use tauri::{AppHandle, Emitter};

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
                                                    eprintln!("[GlobalShortcut] evdev detected Super/Windows key! Emitting toggle-launcher");
                                                    let _ = app.emit_to("dock", "toggle-launcher", ());
                                                    let _ = app.emit("toggle-launcher", ());
                                                }
                                            } else if code == Key::KEY_SPACE.code() {
                                                if alt_pressed && value == 1 {
                                                    if last_search_time.elapsed() >= Duration::from_millis(250) {
                                                        last_search_time = std::time::Instant::now();
                                                        eprintln!("[GlobalShortcut] evdev detected ALT + Space! Emitting trigger-search");
                                                        let _ = app.emit("trigger-search", ());
                                                    }
                                                }
                                            } else if code == Key::KEY_3.code() {
                                                if (alt_pressed || ctrl_pressed) && value == 1 {
                                                    eprintln!("[GlobalShortcut] evdev detected ALT + 3! Emitting toggle-enigma");
                                                    let _ = app.emit("toggle-enigma", ());
                                                }
                                            } else if code == Key::KEY_V.code() {
                                                if alt_pressed && value == 1 {
                                                    eprintln!("[GlobalShortcut] evdev detected ALT + V! Emitting open-clipboard-history");
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

                std::thread::sleep(Duration::from_millis(1000));
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
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::OnceLock;
    use tauri::{AppHandle, Emitter};
    use windows_sys::Win32::Foundation::{LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{VK_LWIN, VK_RWIN};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CallNextHookEx, DispatchMessageW, GetMessageW, SetWindowsHookExW, TranslateMessage,
        UnhookWindowsHookEx, KBDLLHOOKSTRUCT, MSG, WH_KEYBOARD_LL, WM_KEYDOWN, WM_KEYUP,
        WM_SYSKEYDOWN, WM_SYSKEYUP,
    };
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

    static HOOK_APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();
    static WIN_KEY_DOWN: AtomicBool = AtomicBool::new(false);
    static WIN_KEY_COMBINED: AtomicBool = AtomicBool::new(false);

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

            if vk == VK_LWIN || vk == VK_RWIN {
                if is_down {
                    WIN_KEY_DOWN.store(true, Ordering::SeqCst);
                    WIN_KEY_COMBINED.store(false, Ordering::SeqCst);
                } else if is_up {
                    let was_down = WIN_KEY_DOWN.swap(false, Ordering::SeqCst);
                    let combined = WIN_KEY_COMBINED.swap(false, Ordering::SeqCst);
                    if was_down && !combined {
                        // El usuario presionó y soltó únicamente la tecla Windows: abrir/conmutar Enigma Launcher
                        if let Some(app) = HOOK_APP_HANDLE.get() {
                            eprintln!("[GlobalShortcut:WinHook] Windows key pressed alone! Toggling Enigma Launcher");
                            let _ = app.emit_to("dock", "toggle-launcher", ());
                            let _ = app.emit("toggle-launcher", ());
                        }
                        // Suprime el menú de inicio nativo de Windows
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
                eprintln!("[GlobalShortcut:WinHook] WH_KEYBOARD_LL hook installed successfully for Windows Key");

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
