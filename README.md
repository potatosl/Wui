<p align="center">
  <a href="https://wave-lang.dev"><img src="docs/wave.svg" alt="Wave" width="220"></a>
</p>

# Wui

**English** | [한국어](README.ko.md)

A small declarative GUI library for [Wave](https://wave-lang.dev). You describe the
screen again after every event (a function from your state to a tree of widgets); Wui lays it
out, paints it into a pixel buffer and shows it in a window.

```
 your Wave app ──► src/lib.wave ──► src/core.wave ──► src/render.wave ──► pixels
  model, update,     the Ui API       tree, layout,      rasterizer          │
  view, event loop   + win_*          input, scroll      src/font.wave       ▼
                                      draw list          (TrueType / CFF)  src/window/x11.wave ──► libX11
```

Wui is written in Wave. The only things that are not are the Windows and macOS window backends, which need
a window procedure / Cocoa objects (function addresses handed to the OS) - Wave 0.2.x cannot take the
address of a function - so those parts are a small Rust crate (`winplat/`) with a C ABI that
`src/window/win32.wave` calls (it also lists directories there, see below).

| File | What it does |
|---|---|
| `src/lib.wave` | the public API (`ui_*`, `win_*`, key / event constants) |
| `src/core.wave` | front door of the core: re-exports the `core_*` API from `src/core/` |
| `src/core/types.wave` | constants, `Node` / `Field` / `Ctx`, small helpers |
| `src/core/field.wave`, `src/core/textarea.wave` | text editing: UTF-8, caret, selection, word movement; multi-line areas |
| `src/core/ctx.wave`, `src/core/build.wave` | the context (nodes, text store, id-keyed state) and the calls that build the tree |
| `src/core/arrange.wave`, `src/core/pick.wave` | flexbox-style layout; hit testing and scrollbar geometry |
| `src/core/widgets.wave`, `src/core/scroll.wave`, `src/core/focus.wave`, `src/core/input.wave` | sliders / drop-downs / menus / tooltips; scrolling; Tab focus and the caret blink; clicks, drags, text and keys |
| `src/core/paint.wave` | draw-list generation, one function per widget kind |
| `src/render.wave`, `src/gfx.wave` | software rasterizer: fills, borders, rounded shapes, strokes, clips, text |
| `src/font.wave`, `src/fontdb.wave` | TrueType (glyf) and OpenType-CFF reader, anti-aliased glyph rasterizer with a cache; finds the OS's own fonts (wui bundles none) |
| `src/unicode.wave` | generated letter / digit table for word movement in inputs |
| `src/window/x11.wave` | X11 window, events, input method (Hangul / CJK composition), clipboard - libX11 through `extern(c)` |
| `src/window/win32.wave`, `winplat/` | Windows (x86_64, ARM64) and macOS (Intel, Apple silicon) windows: a Rust crate with a C ABI that the Wave wrapper calls (`platform/win32.rs` user32 / gdi32, `platform/macos.rs` Cocoa through the Objective-C runtime, frameworks loaded with dlopen). **Builds, links and passes the unit tests on all four targets in CI (GitHub's Windows and macOS runners); no window has been opened on a real Windows or Mac yet** |
| `src/dirlist.wave`, `src/dir/posix.wave`, `src/dir/native.wave` | directory listing for file pickers: libc `readdir` on Linux / FreeBSD, `winplat` (`std::fs`) on Windows / macOS |
| `src/window/none.wave` | stand-in for any other target: `win_ok()` is false |
| `src/theme.wave`, `src/theme_json.wave` | the `Theme` (colours, radii, paddings) and, optionally, themes as JSON files with wjson |
| `src/wjson_glue.wave` | one-line symbol that lets programs use wjson without its C++ layer |
| `third_party/wjson/` | [wjson](https://github.com/potatosl/wjson), the JSON library for Wave (a copy of its Wave sources, see `SOURCE.md`; used by `src/theme_json.wave`) |

## Use it in your project

Wui is a folder of Wave sources. The easy way is the installer (Linux, FreeBSD, macOS; read it first, it is short):

```
curl -fsSL https://raw.githubusercontent.com/potatosl/Wui/main/install.sh | sh
~/.wui/bin/wui-wavec run app.wave          # or: wui-wavec app.wave -o app
```

It puts the sources in `~/.wui` and writes `wui-wavec`, a wrapper around `wavec` with the right `--dep` and link
flags for your system (on macOS it builds the window crate with cargo first). Or do it by hand: download the
sources (or clone the repository) and point `wavec` at them with `--dep`:

```
curl -L https://github.com/potatosl/Wui/archive/refs/heads/main.tar.gz | tar xz            # -> Wui-main/
wavec build app.wave --dep=wui=./Wui-main --link=:libX11.so.6 --link=m --link=c -o app     # Linux / FreeBSD
```

and in `app.wave` write `import("wui")::{Ui, ui_new, ui_column, ui_button, win_open, ...};`. The
package name must be `wui` (the library imports its own files as `wui::...`). Optional parts need more:
`import("wui::theme_json")` (themes as JSON files) also needs `--dep=wjson=./Wui-main/third_party/wjson`.

**Windows and macOS** need the window crate first (Rust, `cargo`), then link its library:

```
cargo build --release --manifest-path Wui-main/winplat/Cargo.toml --target-dir build/cargo
wavec build app.wave --dep=wui=./Wui-main build/cargo/release/wui_win.lib \
      --link=user32 --link=gdi32 --link=kernel32 --link=ntdll --link=userenv --link=ws2_32 --link=dbghelp -o app   # Windows
wavec build app.wave --dep=wui=./Wui-main build/cargo/release/libwui_win.a --link=System --link=c --link=m -o app    # macOS
```

(`WUI_TASK=...` in `build.wave` does this for the examples; copy its `add_link_flags` for your own build.)

**vex.** A path (or git) dependency works for resolving and compiling:
`dependencies = [ { name = "wui", path = "../Wui-main" } ]` in your `vex.ws`. But `vex` has no way to pass link
flags and Wave has no in-source link directive, so the final link against libX11 / `wui_win` fails
("undefined symbol: XOpenDisplay"); build with `wavec` as above until that exists. (`vex` also expects
`wavec`'s std at `~/.wave/lib/wave/std`; set `VEX_WAVEC` if your install differs.)

**Compilers.** Tested with `wavec` 0.2.1-pre-beta and with the nightly build of Wave `master`
(`0.0.1-alpha-dev`, commit d9fe76d, std compatibility revision 5) on Linux x86_64. CI also builds,
links and runs the unit tests on Windows x64 / ARM64 and macOS Intel / Apple silicon (see Build). Versioning: `docs/VERSIONING.md`. On macOS arm64 keep the default `-O0`:
with `-O2` Wave's std fails inside `std::mem::alloc` (Wave issue #933, found by wjson's CI). CI runs on every push
(`.github/workflows/ci.yml`); its Windows and macOS jobs build, link and test natively.

**Making the archive.** `WUI_TASK=package wavec --std-root=$HOME/.wave/bin/std run build.wave` writes
`dist/wui-<version>.tar.gz` (`WUI_VERSION` sets the version): the library, `winplat/`, docs, examples,
tests, the vendored wjson and the license, no build output.

## Build

```
wavec --std-root=$HOME/.wave/bin/std run build.wave                 run the tests
WUI_TASK=wtest     ... run build.wave     only the Wave unit tests
WUI_TASK=jtest     ... run build.wave     only the theme JSON tests (wjson)
WUI_TASK=xtest     ... run build.wave     only the X11 backend tests (no window is ever shown)
WUI_TASK=examples  ... run build.wave     build every example
WUI_TASK=bench     ... run build.wave     frame-time benchmark
WUI_TASK=check WUI_TARGET=<triple> ...    compile everything for another OS / CPU (objects only)
WUI_RUN=scroll      ... run build.wave     build and start examples/scroll.wave
```

Needs `wavec` and, on Linux, `libX11.so.6` (the library itself, not the -dev package). Programs
that use wui link it with `--link=:libX11.so.6 --link=m --link=c`; the plain `libX11.so` symlink
only exists with the -dev package. On Windows and macOS the driver builds `winplat/` with cargo first and links
it (Windows: `wui_win.lib` + the system libraries Rust's std needs; macOS: `libwui_win.a` + `libSystem`).
Windows ARM64 and macOS build natively with the same commands; cargo picks the host target.

Cross check from another OS (no linking, so no SDK needed): `WUI_TASK=check WUI_TARGET=aarch64-pc-windows-msvc ...`
(also `x86_64-pc-windows-msvc`, `aarch64-apple-darwin`, `x86_64-apple-darwin`) builds `winplat/` for that target
and compiles every example to objects. Linking and running need the target machine (or its SDK).

macOS limits: no input-method composition yet (Korean / CJK typing delivers what `characters` gives), no menu bar
(Cmd+Q closes the window), the buffer is in device pixels (`win_dpi_percent()` is 200 on Retina, scale the theme
with `theme_scale`). Windows limits: the process is not DPI aware yet (`win_dpi_percent()` is 100).

More: [`docs/API.md`](docs/API.md) (reference), [`docs/VERSIONING.md`](docs/VERSIONING.md) (version policy).

## The model

Wui is immediate-mode in spirit: **rebuild the tree on every event, then draw.**

```wave
fun view(ui: ptr<Ui>, m: ptr<Model>, w: i32, h: i32) {
    ui_clear(ui);
    var col: i32 = ui_column(ui, UI_ROOT, 16, 8);      // parent, padding, gap
    ui_text(ui, col, "Count");
    ui_button(ui, col, "+1", MSG_INC);                  // clicking yields MSG_INC
    ui_layout(ui, w, h);
}
```

State that must outlive a frame (text typed in inputs, scroll offsets, focus, selection) lives
inside Wui and is keyed by the ids you pass (`input_id`, `scroll_id`). Keep those ids the same
every frame. Everything else is yours: a `Model`, an `update(msg)` and `view()`.

### The event loop

Every example has this shape (see `examples/scroll.wave` for a complete one):

```wave
var win: Window = win_open("title", 480, 560);
var ui: Ui = ui_new();
view(...); win_draw(&win, &ui);
while (running) {
    var kind: i32 = win_next(&win, &ev);            // blocks for the next event
    if (kind == EV_CLOSE) { running = false; }
    else {
        if (kind == EV_DOWN && ev.a == 1) {          // left press
            var msg: i32 = ui_click(&ui, ev.x, ev.y, ev.mods);
            if (msg != UI_NO_MSG) { update(...); }
        }
        if (kind == EV_UP && ev.a == 1) { ui_release(&ui); }
        if (kind == EV_MOVE)  { ui_set_pointer(&ui, ev.x, ev.y, false); /* ui_drag while held */ }
        if (kind == EV_WHEEL) { ui_scroll(&ui, ev.x, ev.y, ev.a); }
        if (kind == EV_TEXT)  { var n: i64 = win_text(&win, piece, 64); ui_feed_text(&ui, piece, n); }
        if (kind == EV_KEY)   { var r: i32 = win_feed_key(&win, &ui, ev.a, ev.mods); }
        if (kind == EV_EXPOSE) {
            if (!win_pending(&win)) { win_draw(&win, &ui); }   // only re-show the picture
        } else {
            view(...); win_draw(&win, &ui);
        }
    }
}
```

`win_draw` renders, and presents only when the picture changed (or after an expose / resize).

## API

Everything is imported from `wui`. Colours are `0xAARRGGBB`; sizes are pixels.

**Tree.** `ui_new` / `ui_free` / `ui_clear`; containers `ui_column(parent, padding, gap)`,
`ui_row(...)`, `ui_box(parent)`; `ui_text`, `ui_button(text, msg)`, `ui_checkbox(label, checked,
msg)`, `ui_input(input_id, placeholder)`, `ui_scroll_box(scroll_id)`. Each returns a node id to
use as a parent or to style. The first parent is `UI_ROOT`.

**Style** (all take a node id): `ui_padding`, `ui_gap`, `ui_width`, `ui_height`, `ui_grow`,
`ui_align` (`UI_START | UI_CENTER | UI_END | UI_STRETCH`), `ui_justify` (adds `UI_BETWEEN`),
`ui_direction`, `ui_background`, `ui_border`, `ui_margin`, `ui_radius`, `ui_color` (text). Colours can be written as text with `ui_rgb`: `ui_background(ui, box, ui_rgb("#1E3A8A"))` (`#RRGGBB`, `#AARRGGBB`, `#RGB`, `#ARGB`).
`ui_layout(w, h)` lays the tree out; `ui_rect(id)` reads a node's rectangle.

**Mouse.** `ui_click(x, y, mods)` returns the message of the button / checkbox hit, or
`UI_NO_MSG`; it also focuses inputs and starts a scrollbar drag. `ui_drag(x, y)` while the
button is held extends a text selection or moves the scrollbar thumb; `ui_release()` ends it.
`ui_set_pointer(x, y, down)` drives hover / pressed looks. `ui_hit(x, y)` is a click without side
effects.

**Text input.** `ui_input_get / ui_input_set / ui_input_focus / ui_focused`, `ui_selection`,
`ui_feed_text` (typed text), `ui_feed_key` (caret, selection, editing; returns `UI_CHANGED |
UI_SUBMIT | UI_COPY | UI_CUT | UI_PASTE | UI_ACTIVATED | UI_FOCUS_MOVED`). Prefer `win_feed_key`, which also does the clipboard.
Hangul and other IME-composed text arrive as `EV_TEXT`.

**Time.** `win_next` waits for an event for as long as it takes; `win_next_timeout(win, ev, ms)` waits
at most `ms` milliseconds and then returns `EV_TIMEOUT` (it sleeps in `poll` / `MsgWaitForMultipleObjects`,
so it costs no CPU): a clock, a stopwatch or an animation redraws on `EV_TIMEOUT`. A negative `ms`
waits forever. `win_time_ms()` is a steady millisecond clock. The text caret of a focused input blinks
(`Theme.caret_blink_ms`, 0 turns it off); `ui_wake_ms(ui)` says how long until the picture next changes
by itself (or -1), so `win_next_timeout(&win, &ev, ui_wake_ms(&ui))` keeps it blinking while idle.
`win_draw` and the `ui_feed_*` / `ui_click` functions tell the Ui the time (`ui_set_time` for headless use).

**Themes.** The look of the built-in widgets - colours, corner radii, paddings, checkbox and
scrollbar sizes, wheel step - is a `Theme` (`src/theme.wave`). A Ui starts with `theme_light`;
`theme_dark` is the other preset. Copy the current one, change fields, set it:
`var t: Theme = ui_theme_copy(&ui); theme_dark(&t); t.accent = 0xFF22C55E; ui_set_theme(&ui, &t);`
(`ui_theme(&ui)` gives a pointer to edit in place). Styles set on single nodes (`ui_background`,
`ui_color`, `ui_radius`, ...) still win over the theme. With wjson, `import("wui::theme_json")`
reads and writes themes as JSON files (`theme_from_json`, `theme_to_json`): a partial object over a
`"base"` of `"light"` or `"dark"`, colours as `"#RRGGBB"` / `"#AARRGGBB"`.

**Text size, weight and screen scale.** `ui_font_size(node, px)` and `ui_bold(node, bool)` style the
text of a node and everything inside it (the default is `Theme.font_size`, 16 px); buttons, inputs,
checkboxes and the rest measure and draw with the size they inherit. Bold is synthetic - every glyph
is drawn twice, one pixel apart - so it needs no bold font. For high-resolution screens
`win_dpi_percent(win)` says how much this screen wants (100 at 96 dpi, 200 on a typical laptop panel;
X11 reads `Xft.dpi`, else the screen's size in millimetres) and `theme_scale(&theme, percent)` scales
every pixel size of a theme, text included. Sizes you give single nodes (`ui_font_size`, `ui_width`,
`ui_padding`, ...) are yours to scale with the same number.

**More widgets.** `ui_radio(label, selected, msg)` - a round checkbox for "one of several" (the app
keeps which is selected, as with a checkbox); `ui_tab(label, selected, msg)` - a flat button with an
underline on the selected one; `ui_progress(value, max)`; `ui_slider(id, min, max)` - a whole number
whose value lives in the Ui under `id` (read it with `ui_slider_get`, set it with `ui_slider_set`;
mouse, drag and Left / Right / Home / End / PageUp / PageDown move it); `ui_dropdown(id, options)` -
the options are one text separated by newlines, the choice (the option's number) lives in the Ui
(`ui_dropdown_get` / `ui_dropdown_set`), the list opens on a click or Enter / Space and Up / Down /
Enter / Escape work in it; `ui_textarea(id, placeholder, rows)` - a multi-line input that shares
everything with `ui_input` (text under the same kind of id, selection, clipboard, caret blink) and adds
Enter = newline (Ctrl+Enter = `UI_SUBMIT`), Up / Down between lines, line-wise Home / End, PageUp /
PageDown and a scrolling wheel. Inputs, sliders and drop-downs share one id space.

**Overlays.** `ui_overlay(anchor, placement, dx, dy)` makes a box that floats above everything: a
dialog, a menu, a tooltip. It is laid out after the main tree (at a window position with `OVERLAY_AT`,
or `OVERLAY_BELOW` / `OVERLAY_ABOVE` / `OVERLAY_RIGHT` an anchor node, optionally `+ OVERLAY_MATCH_WIDTH`),
painted last, takes every click and wheel turn over its area, and keeps Tab inside it. An
`OVERLAY_FULL` overlay is a modal backdrop. Fill it like any box with the returned id as the parent.
The drop-down's list is an overlay the library makes itself.

**Menus, tooltips, trees, tables, dialogs.** `ui_menu_bar(parent)` is a row of menus; `ui_menu(bar, id,
"File\nOpen\nSave\n-\nQuit", msg_base)` is a flat label that opens a list, item *k* yields `msg_base + k`
like a button (a `-` line is a separator; moving over a neighbouring menu while one is open switches to
it; Left / Right / Up / Down / Enter / Esc work). `ui_tooltip(node, text)` shows a small box next to the
pointer after half a second of rest (it needs the clock: `win_draw` sets it, and `ui_wake_ms` wakes the
loop). `ui_tree_item(parent, depth, label, state, selected, msg_toggle, msg_select)` is one indented
row of a tree (the app keeps which nodes are open). `ui_table_row` + `ui_cell` / `ui_cell_button` give
rows of fixed-width cells whose text is cut at the cell edge. `ui_dialog(width)` is a modal box on a
dimmed backdrop. `DirList` / `dirlist_read(path)` lists a directory (folders first, sorted) and
`ui_file_list` shows it; `path_cwd` / `path_push` / `path_pop` walk paths - together a file picker (see
`examples/files.wave`). Directory reading is libc on Linux / FreeBSD and `std::fs` in `winplat` on
Windows / macOS.

**Keyboard focus.** Inputs, buttons and checkboxes can have the focus. `win_feed_key` moves it with
Tab / Shift+Tab (in tree order, wrapping around, scrolling the widget into view when it sits in a
scroll box) and presses the focused button or checkbox on Enter; `ui_feed_text` does the same for
Space. Both return `UI_ACTIVATED` and `ui_activated()` is the message to handle, exactly like a
click. A focus ring is drawn while the focus came from the keyboard; a mouse click moves the focus
without it. Buttons and checkboxes are found again after a rebuild by their message number, so give
every focusable one its own message (a button with no message, `-1`, is skipped by Tab).
`ui_focus_next(backward)`, `ui_focused_msg()`, `ui_input_focus(id)`.

**Scrolling.** `ui_scroll_box(parent, scroll_id)` is a column that clips its children and shows a
scrollbar when they overflow. Give it a height (`ui_height` or `ui_grow`). `ui_scroll_box_xy` also
scrolls sideways when the content is wider than the box, with a second scrollbar along the bottom
edge; it needs a width too (stretch it in a column, `ui_grow` it in a row, or `ui_width` it).
Wheel: `ui_scroll(x, y, notches)` for the vertical wheel; `ui_scroll_h(x, y, notches)` for
`EV_HWHEEL` (a horizontal wheel or touchpad swipe) and for the vertical wheel with Shift held.
Keys: `ui_scroll_key(key)` (`win_feed_key` calls it for you: PageUp / PageDown always; Up / Down /
Home / End / Left / Right when no input has focus). Scrollbars: click and drag the thumb, or click
the track. Programmatic: `ui_scroll_set(scroll_id, px)` / `ui_scroll_set_x` (a huge value means "to
the end"), `ui_scroll_get` / `ui_scroll_get_x`. Nested scroll boxes work: the innermost one under
the pointer that can still move takes the input. Widgets scrolled out of view cannot be hit.

**Window.** `win_open`, `win_ok`, `win_close`, `win_next` (blocks; returns `EV_*`), `win_pending`,
`win_text`, `win_draw`, `win_width`, `win_height`, `win_clipboard_get / win_clipboard_set`.
`WinEvent` carries `kind, x, y, a, width, height, mods`.

**Events.** `EV_CLOSE, EV_MOVE, EV_DOWN, EV_UP, EV_RESIZE, EV_EXPOSE, EV_KEY, EV_TEXT, EV_WHEEL,
EV_HWHEEL, EV_TIMEOUT`.
Keys: `KEY_BACKSPACE, KEY_TAB, KEY_ENTER, KEY_ESCAPE, KEY_DELETE, KEY_LEFT/RIGHT/UP/DOWN,
KEY_HOME, KEY_END, KEY_PAGE_UP, KEY_PAGE_DOWN`; modifiers `MOD_SHIFT, MOD_CTRL, MOD_ALT`.

Headless use (no window) is possible too: build a tree, `ui_layout`, then `ui_render` into your own
buffer. See the `*_headless.wave` examples.

## Examples

| Example | Shows | Run (`WUI_RUN=<name>`) |
|---|---|---|
| `counter` | buttons, hover / pressed looks, keyboard focus (Tab, Enter, Space), the smallest model-update-view loop | `WUI_RUN=counter` |
| `typing` | keyboard text and IME (Hangul), Backspace / Enter | `WUI_RUN=typing` |
| `theme` | switching themes at run time: light / dark, accent colour, round or square corners; keyboard focus ring | `WUI_RUN=theme` |
| `timer` | `win_next_timeout`: a stopwatch that redraws every 50 ms while running and sleeps while stopped | `WUI_RUN=timer` |
| `widgets` | tabs, radio buttons, slider + progress bar, drop-down, text area and a modal dialog (an overlay) | `WUI_RUN=widgets` |
| `text_styles` | text sizes, bold, and a theme scaled for high-resolution screens | `WUI_RUN=text_styles` |
| `menus` | a menu bar (File / Edit / Help) and tooltips | `WUI_RUN=menus` |
| `lists` | a tree with open / close and a table with a header, zebra rows and scrolling | `WUI_RUN=lists` |
| `files` | a file dialog: modal list of a directory, folders, Up, Open / Cancel | `WUI_RUN=files` |
| `form` | two inputs, a checkbox, Tab focus, selection, clipboard | `WUI_RUN=form` |
| `scroll` | scroll box: wheel, thumb drag, PageUp / PageDown, Top / Bottom | `WUI_RUN=scroll` |
| `*_headless` | the same trees laid out and rendered without a window | `WUI_RUN=counter_headless` |

## Limits

- Only scroll boxes overflow: ordinary boxes keep their children inside their size, and there is
  no wrapping or horizontal flow layout.
- Inputs are single-line and hold at most 4096 bytes. Clipboard transfers of huge data (X11 INCR)
  are not supported.
- Text and widgets are drawn by the software rasterizer; there is no GPU path.

## Tests

`WUI_TASK=wtest` runs the Wave unit tests (`tests/core_test.wave`, 891 checks: layout, hit testing,
input editing and selection, painting, scroll boxes in both directions, scrollbar drag, scroll keys, menus, tooltips, drop-downs, overlays, sliders). `tests/dirlist_test.wave` (32 checks) covers the directory listing and path helpers. `cargo test` in `winplat/` covers its directory functions. They use a
font-less text engine, so every character is 8 px wide and a line 16 px high. `WUI_TASK=xtest`
runs the X11 backend tests (`tests/window_x11_test.wave`): key and modifier mapping, text events,
and a real clipboard round trip between two windows (hidden windows, the system clipboard is
restored afterwards). Without a display the display tests are skipped.

## License

MPL-2.0.
