//= IMPORTS ==================================================================

use crate::input::{
    from_scancode, process_wm_input, register_keyboard_and_mouse, InputFlags, InputSource,
    InputState, MouseButton,
};
use crate::mapping::{InputKind, InputMapping};
use crate::window::{Event, Monitor, WindowSize, DEFAULT_FRAMERATE};
use crate::{signed_hiword, signed_loword, unsigned_hiword, unsigned_loword};

use glam::{U16Vec2, Vec2};
use hashbrown::HashMap;
use raw_window_handle as rwh;
use windows_sys::{
    core::{self, PCWSTR},
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*,
    },
};

use std::num::{NonZeroIsize, NonZeroU16};
use std::{io, mem, process, ptr, sync, thread, time};

//= WINDOW ===================================================================

static RESIZE_NEEDED: sync::Mutex<Option<U16Vec2>> = sync::Mutex::new(None);

pub struct Window {
    hwnd: HWND,
    monitor: Monitor,

    inner_size: WindowSize,
    minimized: bool,

    input_mapping: InputMapping,
    inputs: HashMap<InputSource, InputState>,
    delta_cursor: Vec2,
    delta_scroll: i16,

    frame_budget: time::Duration,
    last_frame_instant: time::Instant,
    last_frame_duration: time::Duration, // It serves to compensate the velocity of the entities even during slow frames
}

impl Window {
    pub fn new(
        title: String,
        width: u16,
        height: u16,
        input_mapping: InputMapping,
    ) -> Result<Window, String> {
        let instance = unsafe { GetModuleHandleW(0 as PCWSTR) };
        if instance == 0 {
            return Err(io::Error::last_os_error().to_string());
        }

        let window_class_name = core::w!("game_window");

        let cursor = unsafe { LoadCursorW(0, IDC_ARROW) };
        if cursor == 0 {
            return Err(io::Error::last_os_error().to_string());
        }

        let wc = WNDCLASSW {
            hCursor: cursor,
            hbrBackground: 0,
            hInstance: instance.into(),
            lpszClassName: window_class_name,
            style: CS_HREDRAW | CS_VREDRAW,
            lpfnWndProc: Some(wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hIcon: 0,
            lpszMenuName: 0 as PCWSTR,
        };

        if unsafe { RegisterClassW(&wc) } == 0 {
            return Err(io::Error::last_os_error().to_string());
        }

        let hwnd: HWND = 0;
        let monitor = Monitor::new(hwnd);
        let frame_budget = monitor.calculate_frame_budget();
        let mut window = Window {
            hwnd,
            monitor,

            inner_size: WindowSize::new(width, height), // TODO: non funziona
            minimized: false,

            input_mapping,
            inputs: HashMap::new(),
            delta_cursor: Vec2::new(0.0, 0.0),
            delta_scroll: 0,

            frame_budget,
            last_frame_instant: time::Instant::now(),
            last_frame_duration: time::Duration::ZERO,
        };

        let mut title_utf16 = title.encode_utf16().collect::<Vec<u16>>();
        title_utf16.push(0);

        let handle = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                window_class_name,
                title_utf16.as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                width as i32,
                height as i32,
                0,
                0,
                instance,
                ptr::null(),
            )
        };

        let success = register_keyboard_and_mouse(handle);
        if success == false {
            log::error!(
                "Error on register keyboard and mouse: {:?}",
                io::Error::last_os_error().to_string()
            );
        }

        // Capture mouse input, allowing `window` to receive mouse events when the cursor
        // is outside the window.
        // TODO: prob c'è qualcosa da cambiare nel minimize
        //use windows::Win32::UI::Input::KeyboardAndMouse::SetCapture;
        //SetCapture(handle);

        window.hide_cursor();

        window.hwnd = handle;
        Ok(window)
    }

    pub fn process_events(&mut self) -> Vec<Event> {
        // Remember: in case you want to emulate frames or physics, that MSG also contains the time.
        let mut msg = unsafe { mem::zeroed() };

        // Dummy local variable for fast access.
        let hwnd = self.hwnd;

        // Reset the delta cursor:
        self.delta_cursor.x = 0.0;
        self.delta_cursor.y = 0.0;

        // Removes all the KeyUp events
        self.inputs
            .retain(|_, state| !state.has_flag(InputFlags::Released));

        let mut events = Vec::new();

        const FIRST_WM: u32 = WM_CREATE;
        const LAST_WM: u32 = WM_DEVICECHANGE;

        loop {
            // Take a quick look at the event queue, if there aren't any, it comes out of the loop.
            // Note that PeekMessage always retrieves WM_QUIT messages, no matter which values
            // you specify for wMsgFilterMin and wMsgFilterMax.
            // I cannot use GetQueueStatus instead of PeekMessage because:
            // "The presence of a QS_ flag in the return value does not guarantee that a
            // subsequent call to the GetMessage or PeekMessage function will return a message."
            unsafe {
                if PeekMessageW(&mut msg, hwnd, FIRST_WM, LAST_WM, PM_REMOVE) == false.into() {
                    break;
                }
            }

            // https://learn.microsoft.com/en-us/windows/win32/inputdev/about-mouse-input
            match msg.message {
                // https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-input
                // https://learn.microsoft.com/en-us/windows/win32/dxtecharts/taking-advantage-of-high-dpi-mouse-movement
                WM_INPUT => {
                    log::trace!("WM_INPUT (255)");
                    if let Some((x, y)) = process_wm_input(msg.lParam) {
                        self.delta_cursor.x = x;
                        self.delta_cursor.y = y;
                        log::trace!("-> DeltaCursor {x} {y}");
                    }
                }

                // Posted to the window with the keyboard focus when a non-system key is pressed.
                // https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-keydown
                // Posted to the window with the keyboard focus when the user presses the F10 key
                // (which activates the menu bar) or holds down the ALT key and then presses
                // another key.
                // https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-syskeydown
                WM_KEYDOWN | WM_SYSKEYDOWN => {
                    log::trace!("WM_KEYDOWN (256) | WM_SYSKEYDOWN (260)");

                    let keycode = {
                        let scancode = (msg.lParam as u32 >> 16) as u8;
                        from_scancode(scancode as u16)
                    };

                    if keycode.is_some() {
                        let keycode = keycode.unwrap();
                        let source = InputSource::Key { source: keycode };

                        if let Some(mut state) = self.inputs.remove(&source) {
                            state.increment_pressure_time();
                            log::trace!("-> Key {:?} {:?}", source, state);
                            self.inputs.insert(source, state);
                        } else {
                            self.process_wm_key_down_or_up(&msg, source, false);
                        }
                    }
                }

                // Same as above, but for up events.
                WM_KEYUP | WM_SYSKEYUP => {
                    log::trace!("WM_KEYUP (257) | WM_SYSKEYUP (261)");

                    let keycode = {
                        let scancode = (msg.lParam as u32 >> 16) as u8;
                        from_scancode(scancode as u16)
                    };

                    if let Some(keycode) = keycode {
                        let source = InputSource::Key { source: keycode };
                        if self.inputs.get(&source).is_none() {
                            log::error!(
                                "Unexpected key not found after an up event: {:?} {:?}",
                                keycode,
                                msg.message
                            );
                        } else {
                            self.process_wm_key_down_or_up(&msg, source, true);
                        }
                    }
                }

                WM_LBUTTONDOWN => {
                    log::trace!("WM_LBUTTONDOWN (513)");
                    let x_pos = signed_loword(msg.lParam as u32);
                    let y_pos = signed_hiword(msg.lParam as u32);
                    let source = InputSource::Mouse {
                        source: MouseButton::Left,
                    };
                    let mut state = if let Some(mut s) = self.inputs.remove(&source) {
                        // Already pressed
                        s.increment_pressure_time();
                        s
                    } else {
                        // Newly pressed
                        InputState::default()
                    };
                    log::trace!("-> MouseLeftDown ({x_pos}, {y_pos})");
                    state.set_coords(x_pos, y_pos);
                    self.inputs.insert(source, state);
                }

                WM_LBUTTONUP => {
                    log::trace!("WM_LBUTTONUP (514)");
                    let x_pos = signed_loword(msg.lParam as u32);
                    let y_pos = signed_hiword(msg.lParam as u32);
                    log::trace!("-> MouseLeftUp ({x_pos}, {y_pos})");
                }

                WM_RBUTTONDOWN => {
                    log::trace!("WM_RBUTTONDOWN (516)");
                    let x_pos = signed_loword(msg.lParam as u32);
                    let y_pos = signed_hiword(msg.lParam as u32);
                    log::trace!("-> MouseRightDown ({x_pos}, {y_pos})");
                }

                WM_RBUTTONUP => {
                    log::trace!("WM_RBUTTONUP (517)");
                    let x_pos = signed_loword(msg.lParam as u32);
                    let y_pos = signed_hiword(msg.lParam as u32);
                    log::trace!("-> MouseRightUp ({x_pos}, {y_pos})");
                }

                WM_MBUTTONDOWN => {
                    log::trace!("WM_MBUTTONDOWN (519)");
                    let x_pos = signed_loword(msg.lParam as u32);
                    let y_pos = signed_hiword(msg.lParam as u32);
                    log::trace!("-> MouseMiddleDown ({x_pos}, {y_pos})");
                }

                WM_MBUTTONUP => {
                    log::trace!("WM_MBUTTONUP (520)");
                    let x_pos = signed_loword(msg.lParam as u32);
                    let y_pos = signed_hiword(msg.lParam as u32);
                    log::trace!("-> MouseMiddleUp ({x_pos}, {y_pos})");
                }

                // Sent to the focus window when the mouse wheel is rotated.
                // https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-mousewheel
                WM_MOUSEWHEEL => {
                    log::trace!("WM_MOUSEWHEEL (522)");
                    self.delta_scroll = signed_hiword(msg.wParam as u32);
                    log::trace!("-> MouseWheel {}", self.delta_scroll);
                }

                WM_XBUTTONDOWN => {
                    log::trace!("WM_XBUTTONDOWN (523)");
                    let kind = signed_hiword(msg.wParam as u32);
                    let x_pos = signed_loword(msg.lParam as u32);
                    let y_pos = signed_hiword(msg.lParam as u32);
                    if kind == 1 {
                        log::trace!("-> MouseXButton1Down ({x_pos}, {y_pos})");
                    } else if kind == 2 {
                        log::trace!("-> MouseXButton2Down ({x_pos}, {y_pos})");
                    }
                }

                WM_XBUTTONUP => {
                    log::trace!("WM_XBUTTONUP (524)");
                    let kind = signed_hiword(msg.wParam as u32);
                    let x_pos = signed_loword(msg.lParam as u32);
                    let y_pos = signed_hiword(msg.lParam as u32);
                    if kind == 1 {
                        log::trace!("-> MouseXButton1Up ({x_pos}, {y_pos})");
                    } else if kind == 2 {
                        log::trace!("-> MouseXButton2Up ({x_pos}, {y_pos})");
                    }
                }

                // Sent to the window that is losing the mouse capture.
                // https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-capturechanged
                // TODO: finirlo
                WM_CAPTURECHANGED => {
                    log::trace!("WM_CAPTURECHANGED (533)");
                }

                _ => {
                    //println!("others WM: {}", msg.message);
                    unsafe {
                        DispatchMessageW(&msg);
                    }
                }
            }
        }

        // We resize just one time for every frame.
        // Try to lock instead to block, the worst that can happen is that the resize is done later,
        // fortunately we have a time limit of one second before the swap-chain is no longer valid.
        match RESIZE_NEEDED.try_lock() {
            Ok(mut inner) => {
                if inner.is_some() {
                    let value = inner.take().unwrap();
                    drop(inner);

                    self.inner_size.width = value.x;
                    self.inner_size.height = value.y;
                    if value.x == 0 || value.y == 0 {
                        log::trace!("-> Window Minimize");
                        self.minimized = true;
                    } else {
                        log::trace!("-> Window Resize ({}, {})", value.x, value.y);
                        self.minimized = false;
                        let width = unsafe { NonZeroU16::new_unchecked(value.x) };
                        let height = unsafe { NonZeroU16::new_unchecked(value.y) };
                        events.push(Event::Resize { width, height });
                    }
                }
            }
            Err(err) => {
                log::error!("Push resize event: {}", err.to_string());
            }
        }

        events
    }

    //- Keyboard Related Events ----------------------------------------------

    // TODO: ricordarsi della context-mode flag:
    // https://learn.microsoft.com/en-us/windows/win32/inputdev/about-keyboard-input#context-code
    fn process_wm_key_down_or_up(&mut self, msg: &MSG, source: InputSource, is_up_event: bool) {
        let state = {
            let mut s = InputState::default();
            //let virtual_key = unsigned_loword(msg.wParam.0 as u32);
            let is_extended_key = ((msg.lParam as u32 >> 24) as u8 & (1u8 << 0)) == 1; // TODO: questa prob la devo usare nella wm_input, oppure me la gestisco io?
            if is_extended_key {
                s.set_flag(InputFlags::ExtendedKey);
            }
            if is_up_event {
                s.set_flag(InputFlags::Released);
            }
            s
        };

        //println!("Key: {:?} {:?}", source, state);
        self.inputs.insert(source, state);
    }

    pub fn get_input_state(&self, kind: InputKind) -> Option<&InputState> {
        let primary = self.input_mapping.get_primary(kind);
        if primary.is_some() {
            let source = primary.unwrap();
            let state = self.inputs.get(source);
            if state.is_some() {
                return state;
            }
        };

        let secondary = self.input_mapping.get_secondary(kind);
        if secondary.is_some() {
            let source = secondary.unwrap();
            let state = self.inputs.get(source);
            if state.is_some() {
                return state;
            }
        }

        None
    }

    //- Mouse Related Events -------------------------------------------------

    pub fn hide_cursor(&self) {
        while unsafe { ShowCursor(false.into()) } >= 0 {
            continue;
        }
    }

    pub fn show_cursor(&self) {
        while unsafe { ShowCursor(true.into()) } < 0 {
            continue;
        }
    }

    // Mouse (or other device) camera rotation
    pub fn handle_cursor_movement(&self) -> Option<Vec2> {
        if self.delta_cursor == Vec2::ZERO {
            return None;
        }

        const SENSITIVITY: f32 = 0.4;
        let mut rotation = Vec2::new(0.0, 0.0);

        // In model space, the camera is looking negative along the Z axis, so
        // moving the cursor up/down corresponds to rotation about the X axis
        rotation.x = {
            let r = SENSITIVITY * self.delta_cursor.y;
            r.clamp(-90.0, 90.0)
        };

        // Moving the cursor left/right corresponds to rotation about the Y axis
        rotation.y = SENSITIVITY * self.delta_cursor.x;

        // The camera does not rotate about the Z axis. That would be like tilting your head

        Some(rotation)
    }

    //- Monitor --------------------------------------------------------------

    #[inline(always)]
    pub fn current_monitor(&self) -> &Monitor {
        &self.monitor
    }

    //- Window Size Related Methods ------------------------------------------

    #[inline(always)]
    pub fn inner_size(&self) -> WindowSize {
        self.inner_size
    }

    #[inline(always)]
    pub fn inner_width(&self) -> u16 {
        self.inner_size.width
    }

    #[inline(always)]
    pub fn inner_height(&self) -> u16 {
        self.inner_size.height
    }

    #[inline(always)]
    pub fn is_minimized(&self) -> bool {
        self.minimized
    }

    //- Raw Window Display and Handle ----------------------------------------

    pub fn raw_display_handle(&self) -> Result<rwh::RawDisplayHandle, rwh::HandleError> {
        Ok(rwh::RawDisplayHandle::Windows(
            rwh::WindowsDisplayHandle::new(),
        ))
    }

    pub fn raw_window_handle(&self) -> Result<rwh::RawWindowHandle, rwh::HandleError> {
        let mut window_handle = rwh::Win32WindowHandle::new(unsafe {
            // SAFETY: Handle will never be zero.
            NonZeroIsize::new_unchecked(self.hwnd)
        });
        // We will only use 64-bit architecture, so there is no need to check the 32-bit version
        let hinstance = unsafe { GetWindowLongPtrW(self.hwnd, GWLP_HINSTANCE) };
        window_handle.hinstance = NonZeroIsize::new(hinstance);
        Ok(rwh::RawWindowHandle::Win32(window_handle))
    }

    //- Frame Time and Sync --------------------------------------------------

    // TODO: al posto del frame budget utilizzare una frazione di quel tempo tale per cui
    //  il resto del wait venga fatto dalla present
    // TODO: https://frankforce.com/frame-rate-delta-buffering/
    pub fn wait_for_frame_sync(&mut self) {
        loop {
            let now = time::Instant::now();
            let last_frame_duration = now.duration_since(self.last_frame_instant);
            if last_frame_duration < self.frame_budget {
                // This web page explains why it is so difficult to create a reliable method of waiting:
                // https://randomascii.wordpress.com/2012/06/05/in-praise-of-idleness/
                // Took from the spin-sleep crate.
                if cfg!(windows) {
                    std::hint::spin_loop();
                } else {
                    thread::yield_now();
                }
            } else {
                //println!("last_frame_duration: {:?}", last_frame_duration);
                self.last_frame_instant = now;
                self.last_frame_duration = last_frame_duration;
                break;
            }
        }
    }

    // Direct proportionality between the actual lowest supported default framerate
    // and the current frame budget.
    // TODO: attenzione che con 240Mhz di monitor potrei avere il frame slug
    pub fn get_frame_modifier(&self) -> f64 {
        let x = 2.0 * self.last_frame_duration.as_secs_f64() / (1.0 / DEFAULT_FRAMERATE);
        //println!("{} {} {}", self.frame_budget.as_secs_f64(), 1.0 / DEFAULT_FRAMERATE, x);
        x
    }
}

//= EXTERNAL SYSTEM WINDOW PROCEDURE =========================================

extern "system" fn wndproc(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // List of window notifications:
    // https://learn.microsoft.com/en-us/windows/win32/winmsg/window-notifications
    // https://wiki.winehq.org/List_Of_Windows_Messages
    match message {
        WM_SIZE => {
            log::trace!("WM_SIZE (5)");
            let width = unsigned_loword(lparam as u32);
            let height = unsigned_hiword(lparam as u32);
            let value = U16Vec2::new(width, height);
            match RESIZE_NEEDED.lock() {
                Ok(mut inner) => {
                    *inner = Some(value);
                }
                Err(err) => {
                    log::error!("WM_SIZE {}", err.to_string());
                }
            }
            0
        }

        // Sent as a signal that a window or an application should terminate.
        // https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-close
        WM_CLOSE => {
            log::trace!("WM_CLOSE (16)");
            process::exit(0);
        }

        _ => {
            //dbg!("wndproc message: ", message);
            unsafe { DefWindowProcW(hwnd, message, wparam, lparam) }
        }
    }
}
