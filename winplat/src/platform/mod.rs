// SPDX-License-Identifier: MPL-2.0
//
// Window backends. Each OS provides one `Win` type implementing `Backend`;
// the rest of wui only talks to this trait, so a new platform is one file.
//
//   Windows     : win32.rs      (user32 / gdi32)
//   macOS       : macos.rs      (Cocoa through the Objective-C runtime)
//   other       : unsupported.rs (X11 is Wave code now, src/window_x11.wave)

// Event kinds of the wui C ABI (see wui_win_next in lib.rs).
pub const EV_NONE: i32 = 0;
pub const EV_CLOSE: i32 = 1;
pub const EV_MOVE: i32 = 2;
pub const EV_DOWN: i32 = 3;
pub const EV_UP: i32 = 4;
pub const EV_RESIZE: i32 = 5;
pub const EV_EXPOSE: i32 = 6;
pub const EV_KEY: i32 = 7; // a control/shortcut key: `a` is a KEY_* code or a lower-case ASCII letter
pub const EV_TEXT: i32 = 8; // typed text (UTF-8, IME-composed text included): `text`
pub const EV_WHEEL: i32 = 9; // mouse wheel: x, y = pointer, a = notches (positive = down / away from the user)
pub const EV_HWHEEL: i32 = 10; // horizontal wheel: x, y = pointer, a = notches (positive = right)
pub const EV_TIMEOUT: i32 = 11; // next_event_timeout: nothing happened within the time

// Cross-platform key codes for EV_KEY (each backend maps its native codes).
pub const KEY_BACKSPACE: i32 = 8;
pub const KEY_TAB: i32 = 9;
pub const KEY_ENTER: i32 = 13;
pub const KEY_ESCAPE: i32 = 27;
pub const KEY_DELETE: i32 = 127;
pub const KEY_LEFT: i32 = 1001;
pub const KEY_RIGHT: i32 = 1002;
pub const KEY_UP: i32 = 1003;
pub const KEY_DOWN: i32 = 1004;
pub const KEY_HOME: i32 = 1005;
pub const KEY_END: i32 = 1006;
pub const KEY_PAGE_UP: i32 = 1007;
pub const KEY_PAGE_DOWN: i32 = 1008;

// Modifier bits in `Event::mods`.
pub const MOD_SHIFT: i32 = 1;
pub const MOD_CTRL: i32 = 2;
pub const MOD_ALT: i32 = 4;

pub const TEXT_MAX: usize = 32;

/// One decoded event: `kind` is an EV_* code; the other fields depend on it
/// (x, y: pointer position; a: button (1 left, 2 middle, 3 right) or key code;
/// width, height: new client size; mods: MOD_* bits for pointer and key
/// events; text/text_len: UTF-8 bytes of an EV_TEXT event).
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Event {
    pub kind: i32,
    pub x: i32,
    pub y: i32,
    pub a: i32,
    pub width: i32,
    pub height: i32,
    pub mods: i32,
    pub text: [u8; TEXT_MAX],
    pub text_len: u8,
}

impl Event {
    /// An EV_TEXT event carrying `bytes` (cut at the last whole character
    /// that fits).
    pub fn text(bytes: &[u8]) -> Event {
        let mut n = bytes.len().min(TEXT_MAX);
        while n > 0 && n < bytes.len() && (bytes[n] & 0xC0) == 0x80 {
            n -= 1;
        }
        let mut e = Event { kind: EV_TEXT, ..Default::default() };
        e.text[..n].copy_from_slice(&bytes[..n]);
        e.text_len = n as u8;
        e
    }
}

/// True for text worth delivering: not empty, not a control character.
pub fn is_text(bytes: &[u8]) -> bool {
    !bytes.is_empty() && bytes[0] >= 0x20 && bytes[0] != 0x7F
}

pub const BACKGROUND: u32 = 0xFFF0F0F0;

/// The window's pixel buffer (0xAARRGGBB, row-major, no padding).
pub struct Surface {
    pub width: i32,
    pub height: i32,
    pub pixels: Vec<u32>,
}

impl Surface {
    pub fn new(width: i32, height: i32) -> Surface {
        Surface { width, height, pixels: vec![BACKGROUND; (width as usize) * (height as usize)] }
    }

    /// Reallocates for a new size. Returns false when nothing changed.
    pub fn resize(&mut self, width: i32, height: i32) -> bool {
        if width <= 0 || height <= 0 || (width == self.width && height == self.height) {
            return false;
        }
        *self = Surface::new(width, height);
        true
    }
}

pub const MAX_SIDE: i32 = 16384;

pub fn valid_size(width: i32, height: i32) -> bool {
    width > 0 && height > 0 && width <= MAX_SIDE && height <= MAX_SIDE
}

pub trait Backend: Sized {
    /// Opens and shows a window with a `width` x `height` client area.
    /// None when the platform has no display or no backend.
    fn open(title: &[u8], width: i32, height: i32) -> Option<Box<Self>>;
    fn size(&self) -> (i32, i32);
    fn pixels_mut(&mut self) -> &mut [u32];
    /// True when `next_event` will not block.
    fn pending(&self) -> bool;
    /// Blocks until the next event wui cares about. A resize event has
    /// already replaced the pixel buffer when it is returned.
    fn next_event(&mut self) -> Event;
    /// Like `next_event`, but gives up after `ms` milliseconds with an EV_TIMEOUT
    /// event. Backends without a timed wait just block like `next_event`.
    fn next_event_timeout(&mut self, _ms: i32) -> Event {
        self.next_event()
    }
    /// Shows the pixel buffer in the window.
    fn present(&mut self);
    /// Display scale in percent (100 = 96 dpi, 200 = Retina); the pixel buffer is in device pixels.
    fn dpi_percent(&self) -> i32 {
        100
    }
    /// Puts text on the system clipboard. Default: platforms without one ignore it.
    fn clipboard_set(&mut self, _text: &str) {}
    /// The system clipboard's text, if it holds any.
    fn clipboard_get(&mut self) -> Option<String> {
        None
    }
}

#[cfg(windows)]
mod win32;
#[cfg(windows)]
pub use win32::Win;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
pub use macos::Win;

#[cfg(not(any(windows, target_os = "macos")))]
mod unsupported;
#[cfg(not(any(windows, target_os = "macos")))]
pub use unsupported::Win;
