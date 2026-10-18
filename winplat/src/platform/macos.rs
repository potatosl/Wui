// SPDX-License-Identifier: MPL-2.0
//
// macOS window backend: Cocoa through the Objective-C runtime, no crates.
//
// Nothing here needs a delegate or a callback: the window is created with
// NSWindow, events are pulled with `nextEventMatchingMask:untilDate:...` and
// the pixel buffer is shown by handing the content view's layer a CGImage.
// Closing is noticed by polling `isVisible`, resizing by polling the content
// view's frame.
//
// Pixels are device pixels: on a Retina screen the buffer is `scale` times the
// size of the window in points, pointer coordinates are multiplied by `scale`,
// and `dpi_percent` reports it (200 on Retina) so a program can scale its theme.
//
// NOTE: written without access to a Mac; it is type-checked for aarch64 / x86_64
// but has never been run. Not supported yet: input methods (Korean / Chinese /
// Japanese composition: only what `characters` delivers arrives as text), a
// menu bar (Cmd+Q is handled as a close event).

#![allow(non_snake_case)]
use std::collections::VecDeque;
use std::ffi::{CStr, CString, c_char, c_void};
use std::sync::OnceLock;

use super::{
    Backend, EV_CLOSE, EV_DOWN, EV_EXPOSE, EV_HWHEEL, EV_KEY, EV_MOVE, EV_RESIZE, EV_TIMEOUT, EV_UP, EV_WHEEL, Event,
    KEY_BACKSPACE, KEY_DELETE, KEY_DOWN, KEY_END, KEY_ENTER, KEY_ESCAPE, KEY_HOME, KEY_LEFT, KEY_PAGE_DOWN, KEY_PAGE_UP,
    KEY_RIGHT, KEY_TAB, KEY_UP, MOD_ALT, MOD_CTRL, MOD_SHIFT, Surface, is_text, valid_size,
};

type Id = *mut c_void;
type Sel = *mut c_void;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Point {
    x: f64,
    y: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Size {
    width: f64,
    height: f64,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Rect {
    origin: Point,
    size: Size,
}

// Everything outside libSystem is loaded with dlopen / dlsym when the first window opens, so linking a
// program needs no framework flags at all (wavec's linker line stays the same as on Linux).
unsafe extern "C" {
    fn dlopen(path: *const c_char, flags: i32) -> *mut c_void;
    fn dlsym(handle: *mut c_void, name: *const c_char) -> *mut c_void;
}

struct Api {
    get_class: unsafe extern "C" fn(*const c_char) -> Id,
    sel_register: unsafe extern "C" fn(*const c_char) -> Sel,
    msg_send: *const (),
    #[cfg(target_arch = "x86_64")]
    msg_send_stret: *const (),
    pool_push: unsafe extern "C" fn() -> *mut c_void,
    pool_pop: unsafe extern "C" fn(*mut c_void),
    cf_data_create: unsafe extern "C" fn(*const c_void, *const u8, isize) -> *const c_void,
    cf_release: unsafe extern "C" fn(*const c_void),
    provider_create: unsafe extern "C" fn(*const c_void) -> *const c_void,
    color_space_create: unsafe extern "C" fn() -> *const c_void,
    image_create: unsafe extern "C" fn(
        usize, usize, usize, usize, usize, *const c_void, u32, *const c_void, *const f64, bool, i32,
    ) -> *const c_void,
}

// Only function pointers and code addresses: safe to share.
unsafe impl Sync for Api {}
unsafe impl Send for Api {}

static API: OnceLock<Option<Api>> = OnceLock::new();

fn load_api() -> Option<Api> {
    const LAZY_GLOBAL: i32 = 0x1 | 0x8; // RTLD_LAZY | RTLD_GLOBAL
    unsafe fn lib(path: &CStr) -> *mut c_void {
        unsafe { dlopen(path.as_ptr(), LAZY_GLOBAL) }
    }
    unsafe fn sym<T>(h: *mut c_void, name: &CStr) -> Option<T> {
        let p = unsafe { dlsym(h, name.as_ptr()) };
        if p.is_null() { None } else { Some(unsafe { std::mem::transmute_copy(&p) }) }
    }
    unsafe {
        let objc = lib(c"/usr/lib/libobjc.A.dylib");
        let appkit = lib(c"/System/Library/Frameworks/AppKit.framework/AppKit");
        let cf = lib(c"/System/Library/Frameworks/CoreFoundation.framework/CoreFoundation");
        let cg = lib(c"/System/Library/Frameworks/CoreGraphics.framework/CoreGraphics");
        if objc.is_null() || appkit.is_null() || cf.is_null() || cg.is_null() {
            return None;
        }
        Some(Api {
            get_class: sym(objc, c"objc_getClass")?,
            sel_register: sym(objc, c"sel_registerName")?,
            msg_send: sym::<*const ()>(objc, c"objc_msgSend")?,
            #[cfg(target_arch = "x86_64")]
            msg_send_stret: sym::<*const ()>(objc, c"objc_msgSend_stret")?,
            pool_push: sym(objc, c"objc_autoreleasePoolPush")?,
            pool_pop: sym(objc, c"objc_autoreleasePoolPop")?,
            cf_data_create: sym(cf, c"CFDataCreate")?,
            cf_release: sym(cf, c"CFRelease")?,
            provider_create: sym(cg, c"CGDataProviderCreateWithCFData")?,
            color_space_create: sym(cg, c"CGColorSpaceCreateDeviceRGB")?,
            image_create: sym(cg, c"CGImageCreate")?,
        })
    }
}

/// The loaded runtime. Only called after `Win::open` found it present.
fn api() -> &'static Api {
    API.get_or_init(load_api).as_ref().expect("macOS frameworks are loaded")
}

fn sel_registerName(name: *const c_char) -> Sel {
    unsafe { (api().sel_register)(name) }
}

fn objc_getClass(name: *const c_char) -> Id {
    unsafe { (api().get_class)(name) }
}

fn objc_autoreleasePoolPush() -> *mut c_void {
    unsafe { (api().pool_push)() }
}

fn objc_autoreleasePoolPop(pool: *mut c_void) {
    unsafe { (api().pool_pop)(pool) }
}

/// `msg!(receiver, c"selector:with:", arg => ArgType, ...; ReturnType)`: one typed objc_msgSend call.
macro_rules! msg {
    ($recv:expr, $sel:expr $(, $arg:expr => $t:ty)* ; $ret:ty) => {{
        let f: unsafe extern "C" fn(Id, Sel $(, $t)*) -> $ret = unsafe { std::mem::transmute(api().msg_send) };
        unsafe { f($recv, sel_registerName($sel.as_ptr()) $(, $arg)*) }
    }};
}

fn class(name: &CStr) -> Id {
    objc_getClass(name.as_ptr())
}

fn nsstring(s: &str) -> Id {
    let c = CString::new(s.replace('\0', "")).unwrap_or_default();
    msg!(class(c"NSString"), c"stringWithUTF8String:", c.as_ptr() => *const c_char; Id)
}

fn to_string(s: Id) -> Option<String> {
    if s.is_null() {
        return None;
    }
    let p = msg!(s, c"UTF8String"; *const c_char);
    if p.is_null() {
        return None;
    }
    Some(unsafe { CStr::from_ptr(p) }.to_string_lossy().into_owned())
}

/// A view's frame size in points. (NSRect is returned in memory on x86_64: objc_msgSend_stret.)
fn frame_size(view: Id) -> Size {
    #[cfg(target_arch = "x86_64")]
    let r = unsafe {
        let mut r = Rect::default();
        let f: unsafe extern "C" fn(*mut Rect, Id, Sel) = std::mem::transmute(api().msg_send_stret);
        f(&mut r, view, sel_registerName(c"frame".as_ptr()));
        r
    };
    #[cfg(not(target_arch = "x86_64"))]
    let r = msg!(view, c"frame"; Rect);
    r.size
}

// NSEventType values
const LEFT_DOWN: u64 = 1;
const LEFT_UP: u64 = 2;
const RIGHT_DOWN: u64 = 3;
const RIGHT_UP: u64 = 4;
const MOUSE_MOVED: u64 = 5;
const LEFT_DRAGGED: u64 = 6;
const RIGHT_DRAGGED: u64 = 7;
const KEY_DOWN_EVENT: u64 = 10;
const SCROLL_WHEEL: u64 = 22;
const OTHER_DOWN: u64 = 25;
const OTHER_UP: u64 = 26;
const OTHER_DRAGGED: u64 = 27;

// NSEventModifierFlags
const FLAG_SHIFT: u64 = 1 << 17;
const FLAG_CONTROL: u64 = 1 << 18;
const FLAG_OPTION: u64 = 1 << 19;
const FLAG_COMMAND: u64 = 1 << 20;

/// kCGImageAlphaNoneSkipFirst | kCGBitmapByteOrder32Little: 0xAARRGGBB words, alpha ignored.
const BITMAP_INFO: u32 = 6 | (2 << 12);

pub struct Win {
    app: Id,
    window: Id,
    view: Id,
    surface: Surface,
    scale: f64,
    points: (i32, i32), // content size in points, to notice a resize
    queue: VecDeque<Event>,
    closed: bool,
    scroll_x: f64, // fractions of a notch not delivered yet
    scroll_y: f64,
}

impl Win {
    fn mods(flags: u64) -> i32 {
        let mut m = 0;
        if flags & FLAG_SHIFT != 0 {
            m |= MOD_SHIFT;
        }
        if flags & (FLAG_CONTROL | FLAG_COMMAND) != 0 {
            m |= MOD_CTRL; // Cmd+C / Cmd+V act like Ctrl+C / Ctrl+V
        }
        if flags & FLAG_OPTION != 0 {
            m |= MOD_ALT;
        }
        m
    }

    fn key_code(code: u16) -> Option<i32> {
        Some(match code {
            51 => KEY_BACKSPACE,
            48 => KEY_TAB,
            36 | 76 => KEY_ENTER,
            53 => KEY_ESCAPE,
            117 => KEY_DELETE,
            123 => KEY_LEFT,
            124 => KEY_RIGHT,
            125 => KEY_DOWN,
            126 => KEY_UP,
            115 => KEY_HOME,
            119 => KEY_END,
            116 => KEY_PAGE_UP,
            121 => KEY_PAGE_DOWN,
            _ => return None,
        })
    }

    /// The pointer position of a mouse event in buffer pixels (None: outside the content view, e.g. the title bar).
    fn pointer(&self, ev: Id) -> Option<(i32, i32)> {
        if msg!(ev, c"window"; Id) != self.window {
            return None;
        }
        let p = msg!(ev, c"locationInWindow"; Point);
        let h = self.points.1 as f64;
        if p.x < 0.0 || p.y < 0.0 || p.x >= self.points.0 as f64 || p.y >= h {
            return None;
        }
        Some(((p.x * self.scale) as i32, ((h - p.y) * self.scale) as i32))
    }

    fn push_key(&mut self, ev: Id) {
        let flags = msg!(ev, c"modifierFlags"; u64);
        let mods = Self::mods(flags);
        let code = msg!(ev, c"keyCode"; u16);
        if let Some(k) = Self::key_code(code) {
            self.queue.push_back(Event { kind: EV_KEY, a: k, mods, ..Default::default() });
            return;
        }
        let shortcut = flags & (FLAG_CONTROL | FLAG_COMMAND) != 0;
        if shortcut {
            if let Some(s) = to_string(msg!(ev, c"charactersIgnoringModifiers"; Id)) {
                let b = s.as_bytes();
                if b.len() == 1 && b[0].is_ascii_alphabetic() {
                    let letter = b[0].to_ascii_lowercase();
                    if letter == b'q' && flags & FLAG_COMMAND != 0 {
                        self.queue.push_back(Event { kind: EV_CLOSE, ..Default::default() });
                    } else {
                        self.queue.push_back(Event { kind: EV_KEY, a: letter as i32, mods, ..Default::default() });
                    }
                }
            }
            return;
        }
        if let Some(s) = to_string(msg!(ev, c"characters"; Id)) {
            // arrow / function keys come as private-use characters (U+F700..): not text
            if is_text(s.as_bytes()) && !s.chars().any(|c| ('\u{F700}'..='\u{F8FF}').contains(&c)) {
                self.queue.push_back(Event::text(s.as_bytes()));
            }
        }
    }

    fn push_scroll(&mut self, ev: Id) {
        let Some((x, y)) = self.pointer(ev) else { return };
        let mods = Self::mods(msg!(ev, c"modifierFlags"; u64));
        let precise = msg!(ev, c"hasPreciseScrollingDeltas"; u8) != 0;
        let dy = msg!(ev, c"scrollingDeltaY"; f64);
        let dx = msg!(ev, c"scrollingDeltaX"; f64);
        // a wheel notch is ~1 line; a trackpad delivers pixels: 40 of them make a notch
        let unit = if precise { 40.0 } else { 1.0 };
        self.scroll_y -= dy / unit; // AppKit: positive = content moves down = scrolled up
        self.scroll_x -= dx / unit;
        let ny = self.scroll_y.trunc();
        let nx = self.scroll_x.trunc();
        self.scroll_y -= ny;
        self.scroll_x -= nx;
        if ny != 0.0 {
            self.queue.push_back(Event { kind: EV_WHEEL, x, y, a: ny as i32, mods, ..Default::default() });
        }
        if nx != 0.0 {
            self.queue.push_back(Event { kind: EV_HWHEEL, x, y, a: nx as i32, mods, ..Default::default() });
        }
    }

    /// Decodes one NSEvent into the queue and gives the system its turn (title bar, resize, ...).
    fn handle(&mut self, ev: Id) {
        let t = msg!(ev, c"type"; u64);
        let mods = Self::mods(msg!(ev, c"modifierFlags"; u64));
        match t {
            LEFT_DOWN | RIGHT_DOWN | OTHER_DOWN | LEFT_UP | RIGHT_UP | OTHER_UP => {
                if let Some((x, y)) = self.pointer(ev) {
                    let button = match t {
                        LEFT_DOWN | LEFT_UP => 1,
                        RIGHT_DOWN | RIGHT_UP => 3,
                        _ => 2,
                    };
                    let kind = if matches!(t, LEFT_DOWN | RIGHT_DOWN | OTHER_DOWN) { EV_DOWN } else { EV_UP };
                    self.queue.push_back(Event { kind, x, y, a: button, mods, ..Default::default() });
                }
            }
            MOUSE_MOVED | LEFT_DRAGGED | RIGHT_DRAGGED | OTHER_DRAGGED => {
                if let Some((x, y)) = self.pointer(ev) {
                    self.queue.push_back(Event { kind: EV_MOVE, x, y, mods, ..Default::default() });
                }
            }
            SCROLL_WHEEL => self.push_scroll(ev),
            KEY_DOWN_EVENT => {
                self.push_key(ev);
                return; // not sent on: nobody handles it and AppKit would beep
            }
            _ => {}
        }
        msg!(self.app, c"sendEvent:", ev => Id; ());
    }

    /// Looks for a closed window or a new size after the system had its turn.
    fn check_window(&mut self) {
        if !self.closed {
            let visible = msg!(self.window, c"isVisible"; u8) != 0;
            let minimized = msg!(self.window, c"isMiniaturized"; u8) != 0;
            if !visible && !minimized {
                self.closed = true;
                self.queue.push_back(Event { kind: EV_CLOSE, ..Default::default() });
                return;
            }
        }
        let s = frame_size(self.view);
        let scale = msg!(self.window, c"backingScaleFactor"; f64);
        let points = (s.width.round() as i32, s.height.round() as i32);
        if (points != self.points || scale != self.scale) && points.0 > 0 && points.1 > 0 {
            self.points = points;
            self.scale = scale;
            let (w, h) = ((points.0 as f64 * scale).round() as i32, (points.1 as f64 * scale).round() as i32);
            if valid_size(w, h) && self.surface.resize(w, h) {
                self.queue.push_back(Event { kind: EV_RESIZE, width: w, height: h, ..Default::default() });
            }
        }
    }

    /// The next event from the system; None when `until` (an NSDate) passes first.
    fn pull(&mut self, until: Id, dequeue: bool) -> Option<Id> {
        let mode = nsstring("kCFRunLoopDefaultMode");
        let ev = msg!(self.app, c"nextEventMatchingMask:untilDate:inMode:dequeue:",
            u64::MAX => u64, until => Id, mode => Id, dequeue as u8 => u8; Id);
        if ev.is_null() { None } else { Some(ev) }
    }

    fn date_in(ms: i32) -> Id {
        if ms < 0 {
            msg!(class(c"NSDate"), c"distantFuture"; Id)
        } else {
            msg!(class(c"NSDate"), c"dateWithTimeIntervalSinceNow:", ms as f64 / 1000.0 => f64; Id)
        }
    }

    /// `ms < 0`: wait for ever.
    fn wait(&mut self, ms: i32) -> Event {
        let started = std::time::Instant::now();
        loop {
            if let Some(e) = self.queue.pop_front() {
                return e;
            }
            if self.closed {
                return Event { kind: EV_CLOSE, ..Default::default() };
            }
            let left = if ms < 0 { -1 } else { (ms as i64 - started.elapsed().as_millis() as i64).max(0) as i32 };
            let pool = objc_autoreleasePoolPush();
            let ev = self.pull(Self::date_in(left), true);
            if let Some(ev) = ev {
                self.handle(ev);
            }
            self.check_window();
            objc_autoreleasePoolPop(pool);
            if self.queue.is_empty() && ms >= 0 && started.elapsed().as_millis() as i64 >= ms as i64 {
                return Event { kind: EV_TIMEOUT, ..Default::default() };
            }
        }
    }
}

impl Backend for Win {
    fn open(title: &[u8], width: i32, height: i32) -> Option<Box<Self>> {
        if !valid_size(width, height) {
            return None;
        }
        if API.get_or_init(load_api).is_none() {
            return None;
        }
        let pool = objc_autoreleasePoolPush();
        let app = msg!(class(c"NSApplication"), c"sharedApplication"; Id);
        if app.is_null() {
            objc_autoreleasePoolPop(pool);
            return None;
        }
        msg!(app, c"setActivationPolicy:", 0i64 => i64; u8); // regular app: Dock icon, can take the focus
        msg!(app, c"finishLaunching"; ());
        let frame = Rect { origin: Point { x: 0.0, y: 0.0 }, size: Size { width: width as f64, height: height as f64 } };
        let alloc = msg!(class(c"NSWindow"), c"alloc"; Id);
        // titled | closable | miniaturizable | resizable, buffered, not deferred
        let window = msg!(alloc, c"initWithContentRect:styleMask:backing:defer:",
            frame => Rect, 15u64 => u64, 2u64 => u64, 0u8 => u8; Id);
        if window.is_null() {
            objc_autoreleasePoolPop(pool);
            return None;
        }
        msg!(window, c"setReleasedWhenClosed:", 0u8 => u8; ());
        msg!(window, c"setAcceptsMouseMovedEvents:", 1u8 => u8; ());
        let t = String::from_utf8_lossy(title);
        msg!(window, c"setTitle:", nsstring(&t) => Id; ());
        let view = msg!(window, c"contentView"; Id);
        msg!(view, c"setWantsLayer:", 1u8 => u8; ());
        msg!(window, c"center"; ());
        msg!(window, c"makeKeyAndOrderFront:", std::ptr::null_mut::<c_void>() => Id; ());
        msg!(app, c"activateIgnoringOtherApps:", 1u8 => u8; ());
        let scale = msg!(window, c"backingScaleFactor"; f64);
        objc_autoreleasePoolPop(pool);

        let (pw, ph) = ((width as f64 * scale).round() as i32, (height as f64 * scale).round() as i32);
        let mut win = Win {
            app,
            window,
            view,
            surface: Surface::new(pw, ph),
            scale,
            points: (width, height),
            queue: VecDeque::new(),
            closed: false,
            scroll_x: 0.0,
            scroll_y: 0.0,
        };
        win.queue.push_back(Event { kind: EV_EXPOSE, ..Default::default() });
        Some(Box::new(win))
    }

    fn size(&self) -> (i32, i32) {
        (self.surface.width, self.surface.height)
    }

    fn pixels_mut(&mut self) -> &mut [u32] {
        &mut self.surface.pixels
    }

    fn pending(&self) -> bool {
        if !self.queue.is_empty() || self.closed {
            return true;
        }
        let past = msg!(class(c"NSDate"), c"distantPast"; Id);
        let mode = nsstring("kCFRunLoopDefaultMode");
        let ev = msg!(self.app, c"nextEventMatchingMask:untilDate:inMode:dequeue:",
            u64::MAX => u64, past => Id, mode => Id, 0u8 => u8; Id);
        !ev.is_null()
    }

    fn next_event(&mut self) -> Event {
        self.wait(-1)
    }

    fn next_event_timeout(&mut self, ms: i32) -> Event {
        self.wait(ms.max(0))
    }

    fn present(&mut self) {
        let (w, h) = (self.surface.width as usize, self.surface.height as usize);
        let bytes = w * h * 4;
        let pool = objc_autoreleasePoolPush();
        let a = api();
        let data = unsafe { (a.cf_data_create)(std::ptr::null(), self.surface.pixels.as_ptr() as *const u8, bytes as isize) };
        let provider = unsafe { (a.provider_create)(data) };
        let space = unsafe { (a.color_space_create)() };
        let image = unsafe { (a.image_create)(w, h, 8, 32, w * 4, space, BITMAP_INFO, provider, std::ptr::null(), false, 0) };
        let layer = msg!(self.view, c"layer"; Id);
        msg!(layer, c"setContentsScale:", self.scale => f64; ());
        msg!(layer, c"setContents:", image as Id => Id; ());
        unsafe {
            (a.cf_release)(image);
            (a.cf_release)(space);
            (a.cf_release)(provider);
            (a.cf_release)(data);
        }
        objc_autoreleasePoolPop(pool);
    }

    fn dpi_percent(&self) -> i32 {
        (self.scale * 100.0).round() as i32
    }

    fn clipboard_set(&mut self, text: &str) {
        let pb = msg!(class(c"NSPasteboard"), c"generalPasteboard"; Id);
        msg!(pb, c"clearContents"; i64);
        msg!(pb, c"setString:forType:", nsstring(text) => Id, nsstring("public.utf8-plain-text") => Id; u8);
    }

    fn clipboard_get(&mut self) -> Option<String> {
        let pb = msg!(class(c"NSPasteboard"), c"generalPasteboard"; Id);
        to_string(msg!(pb, c"stringForType:", nsstring("public.utf8-plain-text") => Id; Id))
    }
}

impl Drop for Win {
    fn drop(&mut self) {
        msg!(self.window, c"close"; ());
        msg!(self.window, c"release"; ());
    }
}
