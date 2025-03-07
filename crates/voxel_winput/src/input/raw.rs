//= IMPORTS ==================================================================

use crate::has_flag;

use windows_sys::Win32::{
    Devices::HumanInterfaceDevice::{
        HID_USAGE_GENERIC_KEYBOARD, HID_USAGE_GENERIC_MOUSE, HID_USAGE_PAGE_GENERIC,
        MOUSE_MOVE_RELATIVE,
    },
    Foundation::{HWND, LPARAM},
    UI::Input::{
        GetRawInputData, RegisterRawInputDevices, RAWINPUT, RAWINPUTDEVICE, RAWINPUTHEADER,
        RIDEV_DEVNOTIFY, RID_INPUT, RIM_TYPEMOUSE,
    },
};

use core::ffi;
use std::mem;

//= GET KEYBOARD PHYSICAL KEY (took from winit 0.29.15) ======================

/*pub fn get_keyboard_physical_key(keyboard: RAWKEYBOARD) -> Option<KeyCode> {
    let extension = {
        if has_flag(keyboard.Flags, RI_KEY_E0 as _) {
            0xE000
        } else if has_flag(keyboard.Flags, RI_KEY_E1 as _) {
            0xE100
        } else {
            0x0000
        }
    };
    let scancode = if keyboard.MakeCode == 0 {
        // In some cases (often with media keys) the device reports a scancode of 0 but a
        // valid virtual key. In these cases we obtain the scancode from the virtual key.
        unsafe { MapVirtualKeyW(keyboard.VKey as u32, MAPVK_VK_TO_VSC_EX) as u16 }
    } else {
        keyboard.MakeCode | extension
    };
    if scancode == 0xE11D || scancode == 0xE02A {
        // At the hardware (or driver?) level, pressing the Pause key is equivalent to pressing
        // Ctrl+NumLock.
        // This equivalence means that if the user presses Pause, the keyboard will emit two
        // subsequent key-presses:
        // 1, 0xE11D - Which is a left Ctrl (0x1D) with an extension flag (0xE100)
        // 2, 0x0045 - Which on its own can be interpreted as Pause
        //
        // There's another combination which isn't quite an equivalence:
        // PrtSc used to be Shift+Asterisk. This means that on some keyboards, pressing
        // PrtSc (print screen) produces the following sequence:
        // 1, 0xE02A - Which is a left shift (0x2A) with an extension flag (0xE000)
        // 2, 0xE037 - Which is a numpad multiply (0x37) with an extension flag (0xE000). This on
        //             its own it can be interpreted as PrtSc
        //
        // For this reason, if we encounter the first keypress, we simply ignore it, trusting
        // that there's going to be another event coming, from which we can extract the
        // appropriate key.
        // For more on this, read the article by Raymond Chen, titled:
        // "Why does Ctrl+ScrollLock cancel dialogs?"
        // https://devblogs.microsoft.com/oldnewthing/20080211-00/?p=23503
        return None;
    }

    let key_code = if keyboard.VKey == VK_NUMLOCK.0 {
        // Historically, the NumLock and the Pause key were one and the same physical key.
        // The user could trigger Pause by pressing Ctrl+NumLock.
        // Now these are often physically separate and the two keys can be differentiated by
        // checking the extension flag of the scancode. NumLock is 0xE045, Pause is 0x0045.
        //
        // However in this event, both keys are reported as 0x0045 even on modern hardware.
        // Therefore we use the virtual key instead to determine whether it's a NumLock and
        // set the KeyCode accordingly.
        //
        // For more on this, read the article by Raymond Chen, titled:
        // "Why does Ctrl+ScrollLock cancel dialogs?"
        // https://devblogs.microsoft.com/oldnewthing/20080211-00/?p=23503
        Some(KeyCode::NumLock)
    } else {
        from_scancode(scancode)
    };

    if keyboard.VKey == VK_SHIFT.0 {
        if let Some(kc) = key_code {
            match kc {
                KeyCode::NumpadDecimal
                | KeyCode::Numpad0
                | KeyCode::Numpad1
                | KeyCode::Numpad2
                | KeyCode::Numpad3
                | KeyCode::Numpad4
                | KeyCode::Numpad5
                | KeyCode::Numpad6
                | KeyCode::Numpad7
                | KeyCode::Numpad8
                | KeyCode::Numpad9 => {
                    // On Windows, holding the Shift key makes numpad keys behave as if NumLock
                    // wasn't active. The way this is exposed to applications by the system is that
                    // the application receives a fake key release event for the shift key at the
                    // moment when the numpad key is pressed, just before receiving the numpad key
                    // as well.
                    //
                    // The issue is that in the raw device event (here), the fake shift release
                    // event reports the numpad key as the scancode. Unfortunately, the event doesn't
                    // have any information to tell whether it's the left shift or the right shift
                    // that needs to get the fake release (or press) event so we don't forward this
                    // event to the application at all.
                    //
                    // For more on this, read the article by Raymond Chen, titled:
                    // "The shift key overrides NumLock"
                    // https://devblogs.microsoft.com/oldnewthing/20040906-00/?p=37953
                    return None;
                }
                _ => (),
            }
        }
    }

    key_code
}
*/

pub(crate) fn register_keyboard_and_mouse(hwnd: HWND) -> bool {
    // RIDEV_DEVNOTIFY: receive hotplug events
    let flags = RIDEV_DEVNOTIFY;

    let devices: [RAWINPUTDEVICE; 2] = [
        RAWINPUTDEVICE {
            usUsagePage: HID_USAGE_PAGE_GENERIC,
            usUsage: HID_USAGE_GENERIC_MOUSE,
            dwFlags: flags,
            hwndTarget: hwnd,
        },
        RAWINPUTDEVICE {
            usUsagePage: HID_USAGE_PAGE_GENERIC,
            usUsage: HID_USAGE_GENERIC_KEYBOARD,
            dwFlags: flags,
            hwndTarget: hwnd,
        },
    ];

    let device_size = size_of::<RAWINPUTDEVICE>() as u32;
    unsafe {
        RegisterRawInputDevices(devices.as_ptr(), devices.len() as u32, device_size) == true.into()
    }
}

// Remember that, using MOUSE_VIRTUAL_DESKTOP, instead of controlling relative movement
// controls the absolute position in the possibly multi-monitor virtual screen.
pub(crate) fn process_wm_input(lparam: LPARAM) -> Option<(f32, f32)> {
    let mut data: RAWINPUT = unsafe { mem::zeroed() };
    let mut data_size = size_of::<RAWINPUT>() as u32;
    let header_size = size_of::<RAWINPUTHEADER>() as u32;

    let status = unsafe {
        GetRawInputData(
            lparam,
            RID_INPUT,
            &mut data as *mut _ as *mut ffi::c_void,
            &mut data_size,
            header_size,
        )
    };

    if status != u32::MAX && status != 0 {
        // The line below will only be useful if multiple mice are supported on a PC.
        //let device_id = data.header.hDevice.0 as u32;

        if data.header.dwType == RIM_TYPEMOUSE {
            let mouse = unsafe { data.data.mouse };
            if has_flag(mouse.usFlags as u32, MOUSE_MOVE_RELATIVE) {
                return Some((mouse.lLastX as f32, mouse.lLastY as f32));
            }
        }
    }

    None
}
