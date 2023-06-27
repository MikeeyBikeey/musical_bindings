//! Active window information
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW};

/// For retriving the currently active window's name.
#[derive(Default)]
pub struct ActiveWindow {
    window: HWND,
    window_name: Option<String>,
}

impl ActiveWindow {
    /// Returns the name of the currently active window.
    pub fn name(&mut self) -> Option<&str> {
        let window = unsafe { GetForegroundWindow() };
        if window == self.window {
            return self.window_name.as_deref();
        }
        if window.0 != 0 {
            // != NULL
            let mut title = [0; 255];
            let read_len = unsafe { GetWindowTextW(window, &mut title) } as usize;
            self.window_name = Some(String::from_utf16_lossy(&title[0..read_len]));
        } else {
            self.window_name = None;
        }
        self.window_name.as_deref()
    }

    /// Returns `true` if the active window has changed since the last call to `active_window_name`.
    pub fn changed(&self) -> bool {
        unsafe { GetForegroundWindow() != self.window }
    }
}

// TODO: Implement direct messaging for certain applications (because it only works for certain applications that actually check the messages)

// pub fn press_notepad_character(unicode_char: u16) {
//     unsafe {
//         let window = FindWindowW(None, w!("*Untitled - Notepad"));
//         if window.0 == 0 {
//             println!("Unable to find window!");
//             return;
//         }

//         if !PostMessageW(window, WM_KEYDOWN, WPARAM(unicode_char as usize), None).as_bool() {
//             println!("Unable to post message!");
//         }
//         if !PostMessageW(window, WM_CHAR, WPARAM(unicode_char as usize), None).as_bool() {
//             println!("Unable to post message!");
//         }
//     }
// }
