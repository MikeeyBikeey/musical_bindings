//! Keyboard `END` key status
use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
use windows::Win32::UI::Input::KeyboardAndMouse::VK_END;

/// Returns `true` if the `END` key is currently being pressed (down) on the keyboard.
pub fn key_end() -> bool {
    let key_state = unsafe { GetAsyncKeyState(VK_END.0 as i32) };
    (1 << 15) & key_state != 0
}
