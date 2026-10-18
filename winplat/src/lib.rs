// SPDX-License-Identifier: MPL-2.0
//
// wui winplat: the Win32 window backend, the one part of wui that cannot be
// written in Wave. A Win32 window needs a window procedure, i.e. a function
// whose address is handed to the OS, and Wave (0.2.x) cannot take the address
// of a function. Everything else - view tree, layout, text engine, rasterizer
// and the X11 backend - is Wave (src/*.wave). On Windows src/window_win32.wave
// calls the C ABI below; on every other target this crate is not needed.
//
// The ABI is the window part of the old Rust core, unchanged.

#![cfg_attr(not(any(windows, target_os = "macos")), allow(dead_code, unused_imports))]

use std::ptr;

mod platform;

use platform::{Backend, EV_NONE, Win};

unsafe fn bytes<'a>(p: *const u8, len: i64) -> &'a [u8] {
    if p.is_null() || len <= 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(p, len as usize) }
    }
}

// ---- window ----------------------------------------------------------------

/// What the C ABI hands out as a window: the platform backend plus the text
/// of the last text event (events carry it by value, the ABI returns it on
/// request).
pub struct Window {
    backend: Win,
    last_text: ([u8; platform::TEXT_MAX], u8),
    needs_present: bool, // the screen lost or never got the current picture
}

/// Opens a window. Returns null when there is no display / backend.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wui_win_open(title: *const u8, len: i64, width: i32, height: i32) -> *mut Window {
    match Win::open(unsafe { bytes(title, len) }, width, height) {
        Some(b) => Box::into_raw(Box::new(Window { backend: *b, last_text: ([0; platform::TEXT_MAX], 0), needs_present: true })),
        None => ptr::null_mut(),
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wui_win_close(w: *mut Window) {
    if !w.is_null() {
        drop(unsafe { Box::from_raw(w) });
    }
}

fn deliver(w: &mut Window, e: platform::Event, out: *mut i32) -> i32 {
    if e.kind == platform::EV_TEXT {
        w.last_text = (e.text, e.text_len);
    }
    if e.kind == platform::EV_EXPOSE || e.kind == platform::EV_RESIZE {
        w.needs_present = true;
    }
    let v = [e.kind, e.x, e.y, e.a, e.width, e.height, e.mods];
    for (i, x) in v.iter().enumerate() {
        unsafe { *out.add(i) = *x };
    }
    e.kind
}

/// Blocks for the next event and writes `[kind, x, y, a, width, height, mods]`
/// into `out[0..7]`. Kinds: 1 close, 2 move, 3 button down, 4 button up,
/// 5 resize, 6 expose, 7 key, 8 text, 9 wheel (`a` = notches, positive = down). `a` is the button (1 = left) or a KEY_*
/// code (or a lower-case ASCII letter with Ctrl/Alt); `mods` are MOD_* bits.
/// For kind 8 fetch the text with `wui_win_text`.
/// Returns the kind, or 0 on a bad argument.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wui_win_next(w: *mut Window, out: *mut i32) -> i32 {
    let (Some(w), false) = (unsafe { w.as_mut() }, out.is_null()) else { return EV_NONE };
    let e = w.backend.next_event();
    deliver(w, e, out)
}

/// Like `wui_win_next`, but returns kind 11 (timeout) when nothing happened within `ms` milliseconds.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wui_win_next_timeout(w: *mut Window, out: *mut i32, ms: i32) -> i32 {
    let (Some(w), false) = (unsafe { w.as_mut() }, out.is_null()) else { return EV_NONE };
    let e = w.backend.next_event_timeout(ms);
    deliver(w, e, out)
}

/// Copies the text of the last text event (UTF-8, NUL-terminated) into `buf`
/// of `cap` bytes. Returns its length in bytes (0 when there is none or the
/// buffer is too small).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wui_win_text(w: *mut Window, buf: *mut u8, cap: i64) -> i64 {
    let Some(w) = (unsafe { w.as_ref() }) else { return 0 };
    let (text, len) = w.last_text;
    let n = len as usize;
    if buf.is_null() || cap < n as i64 + 1 {
        return 0;
    }
    unsafe {
        ptr::copy_nonoverlapping(text.as_ptr(), buf, n);
        *buf.add(n) = 0;
    }
    n as i64
}

/// 1 when an event is waiting (wui_win_next will not block).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wui_win_pending(w: *mut Window) -> i32 {
    match unsafe { w.as_ref() } {
        Some(w) if w.backend.pending() => 1,
        _ => 0,
    }
}

/// The window's pixel buffer (0xAARRGGBB, width*height). It is replaced on
/// resize, so fetch it again after every resize event.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wui_win_pixels(w: *mut Window, width: *mut i32, height: *mut i32) -> *mut u32 {
    let Some(w) = (unsafe { w.as_mut() }) else { return ptr::null_mut() };
    let (cw, ch) = w.backend.size();
    if !width.is_null() {
        unsafe { *width = cw };
    }
    if !height.is_null() {
        unsafe { *height = ch };
    }
    w.backend.pixels_mut().as_mut_ptr()
}

/// Puts UTF-8 text on the system clipboard. Returns 1 when it was handed over.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wui_win_clipboard_set(w: *mut Window, text: *const u8, len: i64) -> i32 {
    let Some(w) = (unsafe { w.as_mut() }) else { return 0 };
    let Ok(t) = std::str::from_utf8(unsafe { bytes(text, len) }) else { return 0 };
    w.backend.clipboard_set(t);
    1
}

/// Copies the clipboard's text (UTF-8, NUL-terminated) into `buf`. Returns its
/// length in bytes, or -1 when the clipboard has no text or `cap` is too small.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wui_win_clipboard_get(w: *mut Window, buf: *mut u8, cap: i64) -> i64 {
    let Some(w) = (unsafe { w.as_mut() }) else { return -1 };
    let Some(t) = w.backend.clipboard_get() else { return -1 };
    if buf.is_null() || cap < t.len() as i64 + 1 {
        return -1;
    }
    unsafe {
        ptr::copy_nonoverlapping(t.as_ptr(), buf, t.len());
        *buf.add(t.len()) = 0;
    }
    t.len() as i64
}

/// The display scale in percent (100, 200 on Retina). The pixel buffer and all coordinates are in device pixels.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wui_win_dpi_percent(w: *mut Window) -> i32 {
    unsafe { w.as_ref() }.map_or(100, |w| w.backend.dpi_percent())
}

/// 1 when the window must be presented even if the picture did not change
/// (it was just exposed or resized).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wui_win_needs_present(w: *mut Window) -> i32 {
    unsafe { w.as_ref() }.map_or(0, |w| i32::from(w.needs_present))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wui_win_present(w: *mut Window) {
    if let Some(w) = unsafe { w.as_mut() } {
        w.backend.present();
        w.needs_present = false;
    }
}


// ---- directory listing -------------------------------------------------------------
//
// Wave's std cannot list a directory, and libc's `readdir` differs per OS (and Windows has
// none), so on Windows and macOS src/dir_native.wave asks this crate; Linux and FreeBSD call libc.
// std::fs does the work.

/// An open directory: wui_dir_next walks it.
pub struct Dir(std::fs::ReadDir);

/// Opens the directory `path` (UTF-8). Null when it cannot be read.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wui_dir_open(path: *const u8, len: i64) -> *mut Dir {
    let Ok(p) = std::str::from_utf8(unsafe { bytes(path, len) }) else { return ptr::null_mut() };
    match std::fs::read_dir(p) {
        Ok(rd) => Box::into_raw(Box::new(Dir(rd))),
        Err(_) => ptr::null_mut(),
    }
}

/// The next entry: its name (UTF-8, NUL-terminated) goes to `buf` of `cap` bytes, `*is_dir` is 1 for a
/// directory (a link to one counts), 0 otherwise. Returns the name's length, or -1 at the end.
/// Entries that are unreadable or whose name does not fit are skipped.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wui_dir_next(d: *mut Dir, buf: *mut u8, cap: i64, is_dir: *mut i32) -> i64 {
    let Some(d) = (unsafe { d.as_mut() }) else { return -1 };
    if buf.is_null() || cap < 2 {
        return -1;
    }
    for entry in d.0.by_ref() {
        let Ok(entry) = entry else { continue };
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if name.len() as i64 + 1 > cap {
            continue;
        }
        unsafe {
            ptr::copy_nonoverlapping(name.as_ptr(), buf, name.len());
            *buf.add(name.len()) = 0;
            if !is_dir.is_null() {
                *is_dir = i32::from(entry.path().is_dir());
            }
        }
        return name.len() as i64;
    }
    -1
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wui_dir_close(d: *mut Dir) {
    if !d.is_null() {
        drop(unsafe { Box::from_raw(d) });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lists_files_and_directories() {
        let dir = std::env::temp_dir().join(format!("wui_dir_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("sub")).unwrap();
        std::fs::write(dir.join("a.txt"), b"x").unwrap();
        std::fs::write(dir.join("한글.txt"), b"x").unwrap();
        let path = dir.to_str().unwrap();
        let d = unsafe { wui_dir_open(path.as_ptr(), path.len() as i64) };
        assert!(!d.is_null());
        let mut seen: Vec<(String, i32)> = Vec::new();
        let mut buf = [0u8; 64];
        loop {
            let mut is_dir = -1;
            let n = unsafe { wui_dir_next(d, buf.as_mut_ptr(), buf.len() as i64, &mut is_dir) };
            if n < 0 {
                break;
            }
            assert_eq!(buf[n as usize], 0, "NUL-terminated");
            seen.push((String::from_utf8(buf[..n as usize].to_vec()).unwrap(), is_dir));
        }
        unsafe { wui_dir_close(d) };
        seen.sort();
        assert_eq!(seen, vec![("a.txt".to_string(), 0), ("sub".to_string(), 1), ("한글.txt".to_string(), 0)]);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_missing_directory_is_null_and_long_names_are_skipped() {
        let missing = "/definitely/not/here";
        assert!(unsafe { wui_dir_open(missing.as_ptr(), missing.len() as i64) }.is_null());
        let dir = std::env::temp_dir().join(format!("wui_dir_long_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("a_very_long_file_name.txt"), b"x").unwrap();
        std::fs::write(dir.join("ok"), b"x").unwrap();
        let path = dir.to_str().unwrap();
        let d = unsafe { wui_dir_open(path.as_ptr(), path.len() as i64) };
        let mut buf = [0u8; 8];
        let n = unsafe { wui_dir_next(d, buf.as_mut_ptr(), 8, ptr::null_mut()) };
        assert_eq!((n, &buf[..2]), (2, &b"ok"[..]));
        assert_eq!(unsafe { wui_dir_next(d, buf.as_mut_ptr(), 8, ptr::null_mut()) }, -1);
        unsafe { wui_dir_close(d) };
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
