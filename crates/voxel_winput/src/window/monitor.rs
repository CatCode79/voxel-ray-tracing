//= IMPORTS ==================================================================

use crate::window::DEFAULT_FRAMERATE;

use windows_sys::{
    core::PCWSTR,
    Win32::{
        Foundation::HWND,
        Graphics::Gdi::{
            EnumDisplaySettingsExW, GetMonitorInfoW, MonitorFromWindow, DEVMODEW,
            ENUM_CURRENT_SETTINGS, HMONITOR, MONITORINFO, MONITORINFOEXW, MONITOR_DEFAULTTONEAREST,
        },
    },
};

use std::{io, mem, time};

//= MONITOR ==================================================================

pub struct Monitor {
    _handle: HMONITOR,
    device_name: Option<String>,
    refresh_rate: Option<f64>,
}

impl Monitor {
    pub fn new(hwnd: HWND) -> Self {
        let handle = get_current_monitor(hwnd);
        let device_name = get_device_name(handle);
        let refresh_rate = device_name.and(get_refresh_rate(device_name.unwrap().as_ptr()));
        let device_name = device_name.and(String::from_utf16(&device_name.unwrap()).ok());
        Self {
            _handle: handle,
            device_name,
            refresh_rate,
        }
    }

    //- Getters --------------------------------------------------------------

    pub fn device_name(&self) -> Option<&String> {
        self.device_name.as_ref()
    }

    pub fn refresh_rate(&self) -> Option<f64> {
        self.refresh_rate
    }

    //- Refresh Rate ---------------------------------------------------------

    pub(crate) fn calculate_frame_budget(&self) -> time::Duration {
        let Some(mut refresh_rate) = self.refresh_rate else {
            return time::Duration::from_secs_f64(1.0 / DEFAULT_FRAMERATE);
        };

        // 48 is because 24 is the minimum FPS for videos.
        // it will never happen given that all monitors have a minimum value of around 60,
        // but it is precisely because it could be just under 60 that I chose this low value.
        while refresh_rate > 48.0 {
            refresh_rate /= 2.0;
        }
        time::Duration::from_secs_f64(1.0 / refresh_rate)
    }
}

fn get_current_monitor(hwnd: HWND) -> HMONITOR {
    let hmonitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    hmonitor
}

fn get_monitor_info(handle: HMONITOR) -> Option<MONITORINFOEXW> {
    let mut monitor_info: MONITORINFOEXW = unsafe { mem::zeroed() };
    monitor_info.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;
    let status = unsafe {
        GetMonitorInfoW(
            handle,
            &mut monitor_info as *mut MONITORINFOEXW as *mut MONITORINFO,
        )
    };
    if status == false.into() {
        log::error!("{}", io::Error::last_os_error().to_string());
        None
    } else {
        Some(monitor_info)
    }
}

fn get_device_name(handle: HMONITOR) -> Option<[u16; 32]> {
    let monitor_info = get_monitor_info(handle)?;
    Some(monitor_info.szDevice)
}

fn get_refresh_rate(device_name: PCWSTR) -> Option<f64> {
    unsafe {
        let mut mode: DEVMODEW = mem::zeroed();
        mode.dmSize = size_of_val(&mode) as u16;
        if EnumDisplaySettingsExW(device_name, ENUM_CURRENT_SETTINGS, &mut mode, 0) == true.into() {
            Some(mode.dmDisplayFrequency as f64) // as millihertz
        } else {
            None
        }
    }
}
