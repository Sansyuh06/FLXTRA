//! UI event handling
use flxtra_core::ipc::{KeyEvent, MouseEvent, MouseButton, MouseEventType, KeyEventType, KeyModifiers};

pub fn mouse_event_from_coords(x: i32, y: i32, button: u32, event_type: u32) -> MouseEvent {
    MouseEvent {
        x, y,
        button: match button { 0 => MouseButton::Left, 1 => MouseButton::Right, 2 => MouseButton::Middle, _ => MouseButton::None },
        event_type: match event_type { 0 => MouseEventType::Move, 1 => MouseEventType::Down, 2 => MouseEventType::Up, 3 => MouseEventType::Click, _ => MouseEventType::Move },
    }
}

pub fn key_event_from_code(key: String, code: String, is_down: bool, ctrl: bool, alt: bool, shift: bool) -> KeyEvent {
    KeyEvent {
        key, code,
        event_type: if is_down { KeyEventType::Down } else { KeyEventType::Up },
        modifiers: KeyModifiers { ctrl, alt, shift, meta: false },
    }
}
