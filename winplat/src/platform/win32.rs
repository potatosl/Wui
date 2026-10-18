// SPDX-License-Identifier: MPL-2.0
//
// Win32 window backend: user32 for the window and message loop, gdi32 to
// show the pixel buffer. No crates; the few structs and functions needed are
// declared here by hand (the layouts are the same on x86_64 and ARM64: LLP64, no packing).
//
// The window procedure runs inside DispatchMessage, i.e. while `next_event`
// is on the stack, so the queue it writes to lives in a separate allocation
// reached through interior mutability instead of through `&mut Win`.

use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::ffi::c_void;
use std::ptr;

use super::{
    Backend, EV_CLOSE, EV_HWHEEL, EV_TIMEOUT, EV_WHEEL, EV_DOWN, EV_EXPOSE, EV_KEY, EV_MOVE, EV_RESIZE, EV_UP, Event, KEY_BACKSPACE,
    KEY_DELETE, KEY_DOWN, KEY_END, KEY_ENTER, KEY_ESCAPE, KEY_HOME, KEY_LEFT, KEY_PAGE_DOWN,
    KEY_PAGE_UP, KEY_RIGHT, KEY_TAB, KEY_UP, MOD_ALT, MOD_CTRL, MOD_SHIFT, Surface, is_text,
    valid_size,
};

type Handle = *mut c_void;
type WndProc = unsafe extern "system" fn(Handle, u32, usize, isize) -> isize;

#[repr(C)]
struct WndClassExW {
    size: u32,
    style: u32,
    wnd_proc: Option<WndProc>,
    cls_extra: i32,
    wnd_extra: i32,
    instance: Handle,
    icon: Handle,
    cursor: Handle,
    background: Handle,
    menu_name: *const u16,
    class_name: *const u16,
    icon_sm: Handle,
}

#[repr(C)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
struct Msg {
    hwnd: Handle,
    message: u32,
    w_param: usize,
    l_param: isize,
    time: u32,
    pt: Point,
}

#[repr(C)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct PaintStruct {
    hdc: Handle,
    erase: i32,
    paint: Rect,
    restore: i32,
    inc_update: i32,
    reserved: [u8; 32],
}

#[repr(C)]
struct BitmapInfoHeader {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bit_count: u16,
    compression: u32,
    size_image: u32,
    x_pels: i32,
    y_pels: i32,
    clr_used: u32,
    clr_important: u32,
}

#[repr(C)]
struct BitmapInfo {
    header: BitmapInfoHeader,
    colors: [u32; 1],
}

#[link(name = "user32")]
unsafe extern "system" {
    fn RegisterClassExW(class: *const WndClassExW) -> u16;
    fn CreateWindowExW(
        ex_style: u32, class: *const u16, title: *const u16, style: u32,
        x: i32, y: i32, w: i32, h: i32,
        parent: Handle, menu: Handle, instance: Handle, param: *mut c_void,
    ) -> Handle;
    fn DestroyWindow(hwnd: Handle) -> i32;
    fn DefWindowProcW(hwnd: Handle, msg: u32, w: usize, l: isize) -> isize;
    fn ShowWindow(hwnd: Handle, cmd: i32) -> i32;
    fn GetMessageW(msg: *mut Msg, hwnd: Handle, min: u32, max: u32) -> i32;
    fn PeekMessageW(msg: *mut Msg, hwnd: Handle, min: u32, max: u32, remove: u32) -> i32;
    fn MsgWaitForMultipleObjects(count: u32, handles: *const Handle, wait_all: i32, ms: u32, mask: u32) -> u32;
    fn TranslateMessage(msg: *const Msg) -> i32;
    fn DispatchMessageW(msg: *const Msg) -> isize;
    fn SetWindowLongPtrW(hwnd: Handle, index: i32, value: isize) -> isize;
    fn GetWindowLongPtrW(hwnd: Handle, index: i32) -> isize;
    fn LoadCursorW(instance: Handle, name: usize) -> Handle;
    fn AdjustWindowRectEx(rect: *mut Rect, style: u32, menu: i32, ex_style: u32) -> i32;
    fn GetDC(hwnd: Handle) -> Handle;
    fn ReleaseDC(hwnd: Handle, hdc: Handle) -> i32;
    fn BeginPaint(hwnd: Handle, ps: *mut PaintStruct) -> Handle;
    fn EndPaint(hwnd: Handle, ps: *const PaintStruct) -> i32;
    fn SetCapture(hwnd: Handle) -> Handle;
    fn ReleaseCapture() -> i32;
    fn GetKeyState(vkey: i32) -> i16;
    fn ScreenToClient(hwnd: Handle, point: *mut [i32; 2]) -> i32;
    fn OpenClipboard(hwnd: Handle) -> i32;
    fn CloseClipboard() -> i32;
    fn EmptyClipboard() -> i32;
    fn SetClipboardData(format: u32, mem: Handle) -> Handle;
    fn GetClipboardData(format: u32) -> Handle;
}

#[link(name = "gdi32")]
unsafe extern "system" {
    fn SetDIBitsToDevice(
        hdc: Handle, x: i32, y: i32, w: u32, h: u32, src_x: i32, src_y: i32,
        start_scan: u32, lines: u32, bits: *const c_void, info: *const BitmapInfo, usage: u32,
    ) -> i32;
}

#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetModuleHandleW(name: *const u16) -> Handle;
    fn GlobalAlloc(flags: u32, bytes: usize) -> Handle;
    fn GlobalFree(mem: Handle) -> Handle;
    fn GlobalLock(mem: Handle) -> *mut c_void;
    fn GlobalUnlock(mem: Handle) -> i32;
    fn GlobalSize(mem: Handle) -> usize;
}

const WS_OVERLAPPEDWINDOW: u32 = 0x00CF0000;
const CW_USEDEFAULT: i32 = i32::MIN;
const SW_SHOW: i32 = 5;
const GWLP_USERDATA: i32 = -21;
const IDC_ARROW: usize = 32512;
const PM_NOREMOVE: u32 = 0;
const PM_REMOVE: u32 = 1;
const QS_ALLINPUT: u32 = 0x04FF;
const WM_QUIT: u32 = 0x0012;
const CF_UNICODETEXT: u32 = 13;
const GMEM_MOVEABLE: u32 = 0x0002;

const WM_SIZE: u32 = 0x0005;
const WM_PAINT: u32 = 0x000F;
const WM_CLOSE: u32 = 0x0010;
const WM_ERASEBKGND: u32 = 0x0014;
const WM_KEYDOWN: u32 = 0x0100;
const WM_CHAR: u32 = 0x0102;
const WM_MOUSEMOVE: u32 = 0x0200;
const WM_LBUTTONDOWN: u32 = 0x0201;
const WM_LBUTTONUP: u32 = 0x0202;
const WM_RBUTTONDOWN: u32 = 0x0204;
const WM_RBUTTONUP: u32 = 0x0205;
const WM_MBUTTONDOWN: u32 = 0x0207;
const WM_MBUTTONUP: u32 = 0x0208;
const WM_MOUSEWHEEL: u32 = 0x020A;
const WM_MOUSEHWHEEL: u32 = 0x020E;
const SIZE_MINIMIZED: usize = 1;

/// Events the window procedure has produced but `next_event` has not returned.
struct Shared {
    queue: RefCell<VecDeque<Event>>,
    /// First half of a UTF-16 surrogate pair, waiting for the second.
    high_surrogate: Cell<u16>,
}

pub struct Win {
    hwnd: Handle,
    shared: *mut Shared,
    surface: Surface,
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn lo(l: isize) -> i32 {
    (l as u16 as i16) as i32
}

fn hi(l: isize) -> i32 {
    ((l >> 16) as u16 as i16) as i32
}

fn mods() -> i32 {
    let down = |vk: i32| unsafe { GetKeyState(vk) } < 0; // high bit = pressed
    (if down(0x10) { MOD_SHIFT } else { 0 })
        | (if down(0x11) { MOD_CTRL } else { 0 })
        | (if down(0x12) { MOD_ALT } else { 0 })
}

/// Windows virtual-key code -> wui key code (0 = not a key wui reports).
fn key_from_vk(vk: u32) -> i32 {
    match vk {
        0x08 => KEY_BACKSPACE,
        0x09 => KEY_TAB,
        0x0D => KEY_ENTER,
        0x1B => KEY_ESCAPE,
        0x2E => KEY_DELETE,
        0x25 => KEY_LEFT,
        0x26 => KEY_UP,
        0x27 => KEY_RIGHT,
        0x28 => KEY_DOWN,
        0x24 => KEY_HOME,
        0x23 => KEY_END,
        0x21 => KEY_PAGE_UP,
        0x22 => KEY_PAGE_DOWN,
        _ => 0,
    }
}

unsafe extern "system" fn wnd_proc(hwnd: Handle, msg: u32, w: usize, l: isize) -> isize {
    let shared = unsafe { GetWindowLongPtrW(hwnd, GWLP_USERDATA) } as *const Shared;
    if shared.is_null() {
        return unsafe { DefWindowProcW(hwnd, msg, w, l) };
    }
    // Shared access only: `next_event` is running further up the stack.
    let q = unsafe { &(*shared).queue };
    let push = |e: Event| q.borrow_mut().push_back(e);
    let button = |kind: i32, a: i32| Event { kind, x: lo(l), y: hi(l), a, mods: mods(), ..Default::default() };
    match msg {
        WM_CLOSE => push(Event { kind: EV_CLOSE, ..Default::default() }), // the app decides
        WM_MOUSEMOVE => push(button(EV_MOVE, 0)),
        WM_LBUTTONDOWN | WM_MBUTTONDOWN | WM_RBUTTONDOWN => {
            unsafe { SetCapture(hwnd) }; // keep getting events while dragging outside
            let a = match msg { WM_LBUTTONDOWN => 1, WM_MBUTTONDOWN => 2, _ => 3 };
            push(button(EV_DOWN, a));
        }
        WM_LBUTTONUP | WM_MBUTTONUP | WM_RBUTTONUP => {
            unsafe { ReleaseCapture() };
            let a = match msg { WM_LBUTTONUP => 1, WM_MBUTTONUP => 2, _ => 3 };
            push(button(EV_UP, a));
        }
        WM_MOUSEWHEEL => {
            // lparam is in screen coordinates; the high word of wparam is the
            // delta (120 per notch, positive = away from the user = up)
            let mut p = [lo(l), hi(l)];
            unsafe { ScreenToClient(hwnd, &mut p) };
            let notches = -((w >> 16) as u16 as i16 as i32) / 120;
            if notches != 0 {
                push(Event { kind: EV_WHEEL, x: p[0], y: p[1], a: notches, mods: mods(), ..Default::default() });
            }
        }
        WM_MOUSEHWHEEL => {
            // like WM_MOUSEWHEEL; the delta is positive = right
            let mut p = [lo(l), hi(l)];
            unsafe { ScreenToClient(hwnd, &mut p) };
            let notches = ((w >> 16) as u16 as i16 as i32) / 120;
            if notches != 0 {
                push(Event { kind: EV_HWHEEL, x: p[0], y: p[1], a: notches, mods: mods(), ..Default::default() });
            }
        }
        WM_KEYDOWN => {
            let m = mods();
            let key = key_from_vk(w as u32);
            if key != 0 {
                push(Event { kind: EV_KEY, a: key, mods: m, ..Default::default() });
            } else if m & (MOD_CTRL | MOD_ALT) != 0 && (0x41..=0x5A).contains(&w) {
                // shortcut letters: report them lower case, like the X11 backend
                push(Event { kind: EV_KEY, a: w as i32 + 32, mods: m, ..Default::default() });
            }
        }
        WM_CHAR => {
            // Typed text arrives as UTF-16 code units (IME-composed text too);
            // characters outside the BMP come as a surrogate pair.
            let unit = w as u16;
            let shared = unsafe { &*shared };
            let cp = match unit {
                0xD800..=0xDBFF => {
                    shared.high_surrogate.set(unit);
                    None
                }
                0xDC00..=0xDFFF => {
                    let hi = shared.high_surrogate.replace(0);
                    (hi != 0).then(|| 0x10000 + (((hi as u32) - 0xD800) << 10) + (unit as u32 - 0xDC00))
                }
                _ => Some(unit as u32),
            };
            if let Some(c) = cp.and_then(char::from_u32) {
                let mut buf = [0u8; 4];
                let bytes = c.encode_utf8(&mut buf).as_bytes();
                if is_text(bytes) {
                    push(Event::text(bytes));
                }
            }
        }
        WM_SIZE if w != SIZE_MINIMIZED => {
            // client size: low/high word, unsigned
            let (cw, ch) = ((l as usize & 0xFFFF) as i32, ((l as usize >> 16) & 0xFFFF) as i32);
            push(Event { kind: EV_RESIZE, width: cw, height: ch, ..Default::default() });
        }
        WM_PAINT => {
            let mut ps: PaintStruct = unsafe { std::mem::zeroed() };
            unsafe {
                BeginPaint(hwnd, &mut ps); // validates the region, else WM_PAINT repeats
                EndPaint(hwnd, &ps);
            }
            push(Event { kind: EV_EXPOSE, ..Default::default() });
        }
        WM_ERASEBKGND => return 1, // we repaint everything ourselves
        _ => return unsafe { DefWindowProcW(hwnd, msg, w, l) },
    }
    0
}

impl Backend for Win {
    fn open(title: &[u8], width: i32, height: i32) -> Option<Box<Win>> {
        if !valid_size(width, height) {
            return None;
        }
        unsafe {
            let instance = GetModuleHandleW(ptr::null());
            let class = wide("WuiWindow");
            let wc = WndClassExW {
                size: std::mem::size_of::<WndClassExW>() as u32,
                style: 0,
                wnd_proc: Some(wnd_proc),
                cls_extra: 0,
                wnd_extra: 0,
                instance,
                icon: ptr::null_mut(),
                cursor: LoadCursorW(ptr::null_mut(), IDC_ARROW),
                background: ptr::null_mut(),
                menu_name: ptr::null(),
                class_name: class.as_ptr(),
                icon_sm: ptr::null_mut(),
            };
            RegisterClassExW(&wc); // fails harmlessly when a second window re-registers it

            // Ask for a client area of the requested size.
            let mut r = Rect { left: 0, top: 0, right: width, bottom: height };
            AdjustWindowRectEx(&mut r, WS_OVERLAPPEDWINDOW, 0, 0);
            let title = wide(&String::from_utf8_lossy(title));
            let hwnd = CreateWindowExW(
                0, class.as_ptr(), title.as_ptr(), WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT, CW_USEDEFAULT, r.right - r.left, r.bottom - r.top,
                ptr::null_mut(), ptr::null_mut(), instance, ptr::null_mut(),
            );
            if hwnd.is_null() {
                return None;
            }
            let shared = Box::into_raw(Box::new(Shared { queue: RefCell::new(VecDeque::new()), high_surrogate: Cell::new(0) }));
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, shared as isize);
            ShowWindow(hwnd, SW_SHOW);
            Some(Box::new(Win { hwnd, shared, surface: Surface::new(width, height) }))
        }
    }

    fn size(&self) -> (i32, i32) {
        (self.surface.width, self.surface.height)
    }

    fn pixels_mut(&mut self) -> &mut [u32] {
        &mut self.surface.pixels
    }

    fn pending(&self) -> bool {
        if !unsafe { &(*self.shared).queue }.borrow().is_empty() {
            return true;
        }
        let mut msg: Msg = unsafe { std::mem::zeroed() };
        unsafe { PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, PM_NOREMOVE) != 0 }
    }

    fn next_event(&mut self) -> Event {
        loop {
            let next = unsafe { &(*self.shared).queue }.borrow_mut().pop_front();
            if let Some(e) = next {
                if e.kind == EV_RESIZE && !self.surface.resize(e.width, e.height) {
                    continue; // same size as before
                }
                return e;
            }
            let mut msg: Msg = unsafe { std::mem::zeroed() };
            // 0 = WM_QUIT, -1 = error: either way nothing more will arrive.
            if unsafe { GetMessageW(&mut msg, ptr::null_mut(), 0, 0) } <= 0 {
                return Event { kind: EV_CLOSE, ..Default::default() };
            }
            unsafe {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    fn next_event_timeout(&mut self, ms: i32) -> Event {
        let deadline = std::time::Instant::now() + std::time::Duration::from_millis(ms.max(0) as u64);
        loop {
            let next = unsafe { &(*self.shared).queue }.borrow_mut().pop_front();
            if let Some(e) = next {
                if e.kind == EV_RESIZE && !self.surface.resize(e.width, e.height) {
                    continue; // same size as before
                }
                return e;
            }
            let mut msg: Msg = unsafe { std::mem::zeroed() };
            if unsafe { PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, PM_REMOVE) } != 0 {
                if msg.message == WM_QUIT {
                    return Event { kind: EV_CLOSE, ..Default::default() };
                }
                unsafe {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
                continue;
            }
            let now = std::time::Instant::now();
            if now >= deadline {
                return Event { kind: EV_TIMEOUT, ..Default::default() };
            }
            // sleep until a message arrives or the time is up
            let left = (deadline - now).as_millis().min(u32::MAX as u128 - 1) as u32;
            unsafe { MsgWaitForMultipleObjects(0, ptr::null(), 0, left, QS_ALLINPUT) };
        }
    }

    fn clipboard_set(&mut self, text: &str) {
        let units: Vec<u16> = text.encode_utf16().chain(std::iter::once(0)).collect();
        let bytes = units.len() * 2;
        unsafe {
            let mem = GlobalAlloc(GMEM_MOVEABLE, bytes);
            if mem.is_null() {
                return;
            }
            let dst = GlobalLock(mem) as *mut u16;
            if dst.is_null() {
                GlobalFree(mem);
                return;
            }
            ptr::copy_nonoverlapping(units.as_ptr(), dst, units.len());
            GlobalUnlock(mem);
            if OpenClipboard(self.hwnd) == 0 {
                GlobalFree(mem);
                return;
            }
            EmptyClipboard();
            // on success the system owns `mem`; on failure it is still ours
            if SetClipboardData(CF_UNICODETEXT, mem).is_null() {
                GlobalFree(mem);
            }
            CloseClipboard();
        }
    }

    fn clipboard_get(&mut self) -> Option<String> {
        unsafe {
            if OpenClipboard(self.hwnd) == 0 {
                return None;
            }
            let mut text = None;
            let mem = GetClipboardData(CF_UNICODETEXT);
            if !mem.is_null() {
                let p = GlobalLock(mem) as *const u16;
                if !p.is_null() {
                    let max = GlobalSize(mem) / 2; // never read past the block
                    let mut n = 0;
                    while n < max && *p.add(n) != 0 {
                        n += 1;
                    }
                    text = Some(String::from_utf16_lossy(std::slice::from_raw_parts(p, n)));
                    GlobalUnlock(mem);
                }
            }
            CloseClipboard();
            text
        }
    }

    fn present(&mut self) {
        let (w, h) = (self.surface.width, self.surface.height);
        let info = BitmapInfo {
            header: BitmapInfoHeader {
                size: std::mem::size_of::<BitmapInfoHeader>() as u32,
                width: w,
                height: -h, // negative: rows run top to bottom
                planes: 1,
                bit_count: 32,
                compression: 0, // BI_RGB; a 0xAARRGGBB word is B, G, R, A in memory
                size_image: 0,
                x_pels: 0,
                y_pels: 0,
                clr_used: 0,
                clr_important: 0,
            },
            colors: [0],
        };
        unsafe {
            let dc = GetDC(self.hwnd);
            if dc.is_null() {
                return;
            }
            SetDIBitsToDevice(
                dc, 0, 0, w as u32, h as u32, 0, 0, 0, h as u32,
                self.surface.pixels.as_ptr() as *const c_void, &info, 0,
            );
            ReleaseDC(self.hwnd, dc);
        }
    }
}

impl Drop for Win {
    fn drop(&mut self) {
        unsafe {
            // Detach the queue first: DestroyWindow sends more messages.
            SetWindowLongPtrW(self.hwnd, GWLP_USERDATA, 0);
            DestroyWindow(self.hwnd);
            drop(Box::from_raw(self.shared));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_keys_map_to_wui_keys() {
        assert_eq!(key_from_vk(0x08), KEY_BACKSPACE);
        assert_eq!(key_from_vk(0x0D), KEY_ENTER);
        assert_eq!(key_from_vk(0x25), KEY_LEFT);
        assert_eq!(key_from_vk(0x41), 0); // letters are text (WM_CHAR), not keys
    }
}
