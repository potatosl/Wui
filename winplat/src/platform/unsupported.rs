// SPDX-License-Identifier: MPL-2.0
//
// Placeholder for platforms without a backend yet (macOS: Cocoa is planned).
// `open` always fails, so Wave programs see win_ok() == false.

use super::{Backend, EV_CLOSE, Event};

pub struct Win;

impl Backend for Win {
    fn open(_title: &[u8], _width: i32, _height: i32) -> Option<Box<Self>> {
        None
    }
    fn size(&self) -> (i32, i32) {
        (0, 0)
    }
    fn pixels_mut(&mut self) -> &mut [u32] {
        &mut []
    }
    fn pending(&self) -> bool {
        false
    }
    fn next_event(&mut self) -> Event {
        Event { kind: EV_CLOSE, ..Default::default() }
    }
    fn present(&mut self) {}
}
