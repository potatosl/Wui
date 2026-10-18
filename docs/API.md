# Wui API reference

Generated from the comments in `src/lib.wave` (the only file a program imports: `import("wui")::{...}`).
Every function takes the `Ui` (or `Window`) first. Node ids are `i32`; `UI_ROOT` is the root.

Library version: `ui_version()`; the versioning policy is in [`VERSIONING.md`](VERSIONING.md).

- `UI_ROOT` = `-1` - parent id of the top-level node
- `UI_INVALID` = `-1` - returned when a node could not be added
- `UI_NO_MSG` = `-1` - no button under the pointer
- `UI_NO_INPUT` = `-1` - no input has focus
- `UI_CHANGED` = `1` - the focused input's text changed
- `UI_SUBMIT` = `2` - Enter was pressed in the focused input
- `UI_COPY` = `4` - Ctrl+C with a selection (win_feed_key does the copying)
- `UI_CUT` = `8` - Ctrl+X with a selection
- `UI_PASTE` = `16` - Ctrl+V
- `UI_ACTIVATED` = `32` - Enter / Space pressed the focused button or checkbox: its message is ui_activated
- `UI_FOCUS_MOVED` = `64` - Tab / Shift+Tab moved the keyboard focus
- `UI_P_DIRECTION` = `0`
- `UI_P_PADDING` = `1`
- `UI_P_GAP` = `2`
- `UI_P_WIDTH` = `3`
- `UI_P_HEIGHT` = `4`
- `UI_P_GROW` = `5`
- `UI_P_ALIGN` = `6`
- `UI_P_JUSTIFY` = `7`
- `UI_P_BACKGROUND` = `8`
- `UI_P_BORDER` = `9`
- `UI_P_MARGIN` = `10`
- `UI_P_RADIUS` = `11`
- `UI_P_COLOR` = `12`
- `UI_P_FONT_SIZE` = `13`
- `UI_P_BOLD` = `14`
- `UI_COLUMN` = `0`
- `UI_ROW` = `1`
- `UI_START` = `0`
- `UI_CENTER` = `1`
- `UI_END` = `2`
- `UI_STRETCH` = `3` - ui_align only
- `UI_BETWEEN` = `3` - ui_justify only
- `UI_AUTO` = `-1` - ui_width / ui_height
- `EV_NONE` = `0`
- `EV_CLOSE` = `1` - the user closed the window
- `EV_MOVE` = `2` - pointer moved: x, y
- `EV_DOWN` = `3` - button pressed: x, y, a = button (1 = left)
- `EV_UP` = `4` - button released: x, y, a = button
- `EV_RESIZE` = `5` - window resized: width, height
- `EV_EXPOSE` = `6` - needs a redraw
- `EV_KEY` = `7` - control key / shortcut: a = KEY_* code (or a lower-case letter with Ctrl/Alt)
- `EV_TEXT` = `8` - typed text: fetch it with win_text() (UTF-8, IME-composed text included)
- `EV_WHEEL` = `9` - mouse wheel: x, y = pointer, a = notches (positive = down); pass them to ui_scroll
- `EV_TIMEOUT` = `11` - win_next_timeout: nothing happened within the time
- `EV_HWHEEL` = `10` - horizontal wheel (touchpad, tilt wheel): a = notches (positive = right); pass them to ui_scroll_h
- `KEY_BACKSPACE` = `8`
- `KEY_TAB` = `9`
- `KEY_ENTER` = `13`
- `KEY_ESCAPE` = `27`
- `KEY_DELETE` = `127`
- `KEY_LEFT` = `1001`
- `KEY_RIGHT` = `1002`
- `KEY_UP` = `1003`
- `KEY_DOWN` = `1004`
- `KEY_HOME` = `1005`
- `KEY_END` = `1006`
- `KEY_PAGE_UP` = `1007`
- `KEY_PAGE_DOWN` = `1008`
- `MOD_SHIFT` = `1`
- `MOD_CTRL` = `2`
- `MOD_ALT` = `4`
### `ui_version() -> str`

### `ui_new() -> Ui`

### `ui_free(ui: ptr<Ui>)`

### `ui_clear(ui: ptr<Ui>)`

Starts a new frame: drops the previous tree.

### `ui_column(ui: ptr<Ui>, parent: i32, padding: i32, spacing: i32) -> i32`

### `ui_row(ui: ptr<Ui>, parent: i32, padding: i32, spacing: i32) -> i32`

### `ui_text(ui: ptr<Ui>, parent: i32, text: str) -> i32`

### `ui_button(ui: ptr<Ui>, parent: i32, text: str, msg: i32) -> i32`

### `ui_tooltip(ui: ptr<Ui>, id: i32, text: str)`

A tooltip for node `id` (any widget or container): a small box with `text` that appears
next to the pointer once it has rested on the node for half a second, and goes away when
the pointer leaves or clicks. It needs the clock (win_draw sets it; a headless program
calls ui_set_time) and ui_wake_ms to wake the loop up in time.

### `ui_box(ui: ptr<Ui>, parent: i32) -> i32`

A container (vertical, no padding); see ui_direction/ui_padding/ui_gap.


## overlays

### `ui_overlay(ui: ptr<Ui>, anchor: i32, placement: i32, dx: i32, dy: i32) -> i32`

An overlay is a box that floats above everything else - a dialog, a menu, a tooltip.
It is laid out after the main tree (at a window position, or next to another node),
painted last, and takes every click and wheel turn over its area; Tab stays inside
the topmost one. Build it like any box, with the returned id as the parent:

  var menu: i32 = ui_overlay(&ui, button, OVERLAY_BELOW, 0, 4);
  ui_button(&ui, menu, "Open", MSG_OPEN);
  ui_button(&ui, menu, "Save", MSG_SAVE);

An overlay as big as the window (OVERLAY_FULL, with a background colour if you like)
is a modal backdrop: nothing below it can be clicked; put the dialog box inside it
(ui_align / ui_justify centre it). `anchor` is a node of the main tree; the placements are OVERLAY_AT
(window position dx, dy), OVERLAY_BELOW, OVERLAY_ABOVE, OVERLAY_RIGHT (against the
anchor, moved by dx, dy), and OVERLAY_MATCH_WIDTH can be added to make it at least as
wide as the anchor. It is moved back inside the window when it would stick out.
(UI_ROOT as the anchor means none.)


## scrolling

### `ui_scroll_box(ui: ptr<Ui>, parent: i32, scroll_id: i32) -> i32`

### `ui_scroll_box_xy(ui: ptr<Ui>, parent: i32, scroll_id: i32) -> i32`

Like ui_scroll_box, but the box also scrolls sideways when its content is
wider than it (a horizontal scrollbar along the bottom edge, ui_scroll_h, the
Left / Right keys). It needs a width as well as a height: stretch it in a
column (ui_align(parent, UI_STRETCH)), grow it in a row, or ui_width it.

### `ui_scroll_h(ui: ptr<Ui>, x: i32, y: i32, notches: i32) -> bool`

The horizontal wheel turned `notches` (positive = right) over (x, y) - pass
EV_HWHEEL events here, and EV_WHEEL events with Shift held (MOD_SHIFT in
WinEvent.mods) too: scrolls the innermost sideways-scrolling box there that
can still move. Returns true when something scrolled.

### `ui_scroll(ui: ptr<Ui>, x: i32, y: i32, notches: i32) -> bool`

The wheel turned `notches` (positive = down) over (x, y): scrolls the
innermost scroll box there that can still move. Returns true when something
scrolled (the view needs rebuilding).

### `ui_scroll_key(ui: ptr<Ui>, key: i32) -> bool`

A scroll key: KEY_UP / KEY_DOWN scroll a step, KEY_PAGE_UP / KEY_PAGE_DOWN a
page, KEY_HOME / KEY_END jump to the ends, KEY_LEFT / KEY_RIGHT scroll a step
sideways (ui_scroll_box_xy boxes). The scroll box under the pointer
moves (else the first one that can). Returns true when something scrolled.
win_feed_key calls this for you whenever the key is not meant for an input.

### `ui_scroll_set(ui: ptr<Ui>, scroll_id: i32, offset: i32)`

Sets a scroll box's offset in pixels; a huge value scrolls to the end.

### `ui_scroll_set_x(ui: ptr<Ui>, scroll_id: i32, offset: i32)`

The horizontal offset of a scroll box made with ui_scroll_box_xy.

### `ui_scroll_get_x(ui: ptr<Ui>, scroll_id: i32) -> i32`

### `ui_scroll_get(ui: ptr<Ui>, scroll_id: i32) -> i32`

A scroll box's offset in pixels as of the last layout, or -1 if unknown.

### `ui_style(ui: ptr<Ui>, id: i32, prop: i32, value: i64) -> bool`

Sets one style property of a node. Returns false for a bad node/property.

### `ui_direction(ui: ptr<Ui>, id: i32, dir: i64) -> bool`

### `ui_padding(ui: ptr<Ui>, id: i32, px: i64) -> bool`

### `ui_gap(ui: ptr<Ui>, id: i32, px: i64) -> bool`

### `ui_width(ui: ptr<Ui>, id: i32, px: i64) -> bool`

### `ui_height(ui: ptr<Ui>, id: i32, px: i64) -> bool`

### `ui_grow(ui: ptr<Ui>, id: i32, share: i64) -> bool`

### `ui_align(ui: ptr<Ui>, id: i32, a: i64) -> bool`

### `ui_justify(ui: ptr<Ui>, id: i32, j: i64) -> bool`

### `ui_background(ui: ptr<Ui>, id: i32, argb: i64) -> bool`

Colors are 0xAARRGGBB; 0 means none.

### `ui_border(ui: ptr<Ui>, id: i32, argb: i64) -> bool`

### `ui_margin(ui: ptr<Ui>, id: i32, px: i64) -> bool`

Space around the node, on all sides; it counts in the parent's layout.

### `ui_radius(ui: ptr<Ui>, id: i32, px: i64) -> bool`

Corner radius in pixels (boxes are square by default; buttons, inputs and
checkboxes have a small default radius, -1 restores it).

### `ui_color(ui: ptr<Ui>, id: i32, argb: i64) -> bool`

Text colour (0xAARRGGBB). It is inherited: set it on a box and everything
inside uses it unless a child sets its own.

### `ui_font_size(ui: ptr<Ui>, id: i32, px: i64) -> bool`

Text size in pixels for this node and everything inside it (0 = inherit; the default is
Theme.font_size, 16). Boxes, buttons, inputs and the rest measure and draw with it.

### `ui_bold(ui: ptr<Ui>, id: i32, bold: bool) -> bool`

Bold text for this node and everything inside it (false = regular again, even inside a bold parent).

### `ui_checkbox(ui: ptr<Ui>, parent: i32, label: str, checked: bool, msg: i32) -> i32`

A checkbox with a label. `checked` is the app's current state; a click
yields `msg` through ui_click / ui_hit, and the app flips its state:

  ui_checkbox(&ui, col, "Subscribe", model.subscribed, MSG_SUBSCRIBE);

### `ui_radio(ui: ptr<Ui>, parent: i32, label: str, selected: bool, msg: i32) -> i32`

A radio button: a round checkbox for "one of several". `selected` is the app's state;
a click (or Enter / Space on the focused one) yields `msg`, the app selects that
choice and rebuilds. Tab walks through the choices (give each its own message).

### `ui_tab(ui: ptr<Ui>, parent: i32, label: str, selected: bool, msg: i32) -> i32`

A tab: a flat button that marks the `selected` one with an underline. The app keeps
which tab is selected and shows the matching content.

### `ui_progress(ui: ptr<Ui>, parent: i32, value: i32, max: i32) -> i32`

A progress bar showing `value` of `max` (the app's numbers; the bar is not interactive).
It is as wide as an input unless you ui_width / ui_grow / stretch it.

### `ui_slider(ui: ptr<Ui>, parent: i32, slider_id: i32, min: i32, max: i32) -> i32`

A slider for a whole number from `min` to `max`. Its value lives in the Ui under
`slider_id` (like an input's text, and in the same id space: pick numbers no input
uses): it starts at `min`; a click or drag moves it, and so do Left / Right, Home /
End and PageUp / PageDown while it has the keyboard focus. Read it with
ui_slider_get - in your view, every frame, no message needed.

### `ui_slider_set(ui: ptr<Ui>, slider_id: i32, value: i32)`

### `ui_slider_get(ui: ptr<Ui>, slider_id: i32) -> i32`

### `ui_textarea(ui: ptr<Ui>, parent: i32, input_id: i32, placeholder: str, rows: i32) -> i32`

A multi-line text input, `rows` lines tall (ui_height / ui_grow change that). It is an input
in every other respect - its text lives in the Ui under `input_id` (ui_input_get /
ui_input_set, newlines included); selection, clipboard, focus and the caret work the same.
Enter types a newline (Ctrl+Enter reports UI_SUBMIT); Up / Down move between lines; Home / End go
to the start / end of the line (with Ctrl: of the text); PageUp / PageDown move a page; the wheel
scrolls it. Lines are not wrapped: a long one scrolls sideways.

### `ui_dropdown(ui: ptr<Ui>, parent: i32, dropdown_id: i32, options: str) -> i32`

A drop-down: a box showing the chosen option; a click (or Enter / Space with the keyboard
focus) opens the list of options under it. `options` is one text, the options separated
by newlines: "Red\nGreen\nBlue". The choice - the number of the option, from 0 - lives in
the Ui under `dropdown_id` (the id space of inputs and sliders): read it with
ui_dropdown_get in your view, set it with ui_dropdown_set. While the list is closed Up /
Down change the choice; while it is open they move the highlight, Enter picks it and
Escape closes the list (win_feed_key delivers all of that).

### `ui_dropdown_set(ui: ptr<Ui>, dropdown_id: i32, index: i32)`

### `ui_dropdown_get(ui: ptr<Ui>, dropdown_id: i32) -> i32`

### `ui_dropdown_open(ui: ptr<Ui>) -> i32`

The id of the drop-down whose list is open, or UI_NO_INPUT.


## menus

### `ui_menu_bar(ui: ptr<Ui>, parent: i32) -> i32`

The row that holds the menus; give it ui_align(page, UI_STRETCH) in its parent to span the window.

### `ui_menu(ui: ptr<Ui>, parent: i32, menu_id: i32, items: str, msg_base: i32) -> i32`

A menu. `items` is the label, then the items, one per line; a line of "-" is a separator.
Picking item number k (counted from 0 after the label, separators included) yields msg_base + k.
`menu_id` (>= 0) is a control id in the space of inputs, sliders and drop-downs.


## tree

- `UI_LEAF` = `0` - no children: no marker
- `UI_COLLAPSED` = `1` - "+"
- `UI_EXPANDED` = `2` - "-"
- `UI_TREE_INDENT` = `18` - pixels per level
### `ui_tree_item(ui: ptr<Ui>, parent: i32, depth: i32, label: str, state: i32, selected: bool, msg_toggle: i32, msg_select: i32) -> i32`


## tables

### `ui_table_row(ui: ptr<Ui>, parent: i32, index: i32) -> i32`

### `ui_cell(ui: ptr<Ui>, row: i32, text: str, width: i32) -> i32`

A text cell `width` pixels wide (its text is indented 4 px, like the label of a ui_cell_button).

### `ui_cell_button(ui: ptr<Ui>, row: i32, text: str, width: i32, msg: i32) -> i32`

A cell that is a flat button yielding `msg`: make one per row to select it.


## dialogs and file lists

### `ui_dialog(ui: ptr<Ui>, width: i32) -> i32`

A modal dialog box (centred, `width` px wide, 0 = as wide as its content). Returns the box.

### `ui_file_list(ui: ptr<Ui>, parent: i32, scroll_id: i32, list: ptr<DirList>, selected: i32, msg_base: i32) -> i32`

A scroll box (id `scroll_id`) with one row per entry of `list`: folders get a "dir" tag. Clicking
entry i yields msg_base + i; `selected` (-1 for none) is underlined. Give it a height.


## inputs

### `ui_input(ui: ptr<Ui>, parent: i32, input_id: i32, placeholder: str) -> i32`

### `ui_input_set(ui: ptr<Ui>, input_id: i32, text: str) -> bool`

Replaces the text of an input; the caret goes to the end.

### `ui_input_get(ui: ptr<Ui>, input_id: i32, buf: ptr<u8>, cap: i64) -> i64`

Copies the text into buf (NUL-terminated UTF-8); returns its byte length,
or -1 if the input is unknown or cap is too small.

### `ui_input_focus(ui: ptr<Ui>, input_id: i32)`

### `ui_focused(ui: ptr<Ui>) -> i32`

Id of the focused input, or UI_NO_INPUT.


## theme

### `ui_theme(ui: ptr<Ui>) -> ptr<Theme>`

The Ui's theme, to read or change in place (edits apply from the next layout).

### `ui_set_theme(ui: ptr<Ui>, t: ptr<Theme>)`

Replaces the Ui's theme with a copy of `t`.

### `ui_theme_copy(ui: ptr<Ui>) -> Theme`

A copy of the Ui's current theme.


## time

### `ui_set_time(ui: ptr<Ui>, ms: i64)`

Sets the current time in milliseconds (any steady clock; only differences matter).

### `ui_wake_ms(ui: ptr<Ui>) -> i32`

Milliseconds until the picture next changes without input, or -1 when it will not.


## keyboard focus

### `ui_focus_next(ui: ptr<Ui>, backward: bool) -> bool`

Moves the focus to the next (or, with `backward`, the previous) focusable widget.

### `ui_activated(ui: ptr<Ui>) -> i32`

The message of the widget the last UI_ACTIVATED pressed, or UI_NO_MSG.

### `ui_focused_msg(ui: ptr<Ui>) -> i32`

The message of the focused button or checkbox, or UI_NO_MSG (an input may have the focus).

### `ui_click(ui: ptr<Ui>, x: i32, y: i32, mods: i32) -> i32`

A mouse press (mods = WinEvent.mods): focuses the input under the pointer
(or clears focus), puts the caret there - with Shift held the selection grows
to it - and returns the message of the button that was hit, or UI_NO_MSG.
Keep the button's state yourself and call ui_drag while it is held.

### `ui_drag(ui: ptr<Ui>, x: i32, y: i32) -> bool`

The pointer moved while the button is held after a press on an input: extends
the selection. Returns true when it changed (the view needs redrawing).

### `ui_release(ui: ptr<Ui>)`

The mouse button was released: ends the drag.

### `ui_selection(ui: ptr<Ui>, buf: ptr<u8>, cap: i64) -> i64`

The focused input's selected text, NUL-terminated UTF-8, in buf. Returns its
length in bytes (0 = nothing selected), or -1 if there is no focus / cap is too small.

### `ui_feed_text(ui: ptr<Ui>, text: ptr<u8>, n: i64) -> i32`

Typed text (UTF-8, n bytes) for the focused input; returns UI_* bits.

### `ui_feed_key(ui: ptr<Ui>, key: i32, mods: i32) -> i32`

A control key for the focused input (KEY_* code, or a lower-case letter with
MOD_CTRL), with WinEvent.mods. Editing and selecting happen here: arrows,
Home/End and Ctrl+arrows (extending the selection with Shift), Backspace,
Delete, Ctrl+A. Returns UI_* bits. UI_COPY / UI_CUT / UI_PASTE are left for
the app - or use win_feed_key, which does the clipboard part too.

### `ui_layout(ui: ptr<Ui>, width: i32, height: i32)`

Lays the tree out in a width x height area (usually the window); the root fills it.

### `ui_rect(ui: ptr<Ui>, id: i32) -> UiRect`

### `ui_set_pointer(ui: ptr<Ui>, x: i32, y: i32, down: bool)`

### `ui_hit(ui: ptr<Ui>, x: i32, y: i32) -> i32`

Message of the button at (x, y), or UI_NO_MSG.

### `ui_render(ui: ptr<Ui>, pixels: ptr<u8>, width: i32, height: i32) -> bool`

Paints the current tree into `pixels` (width * height u32, 0xAARRGGBB).
Returns false, and leaves `pixels` alone, when the picture is identical to
the previous one - so a view that did not change costs almost nothing.


## window

### `win_open(title: str, width: i32, height: i32) -> Window`

Opens a window. Check win_ok(): false when there is no display.

### `win_ok(win: ptr<Window>) -> bool`

### `win_close(win: ptr<Window>)`

### `win_next(win: ptr<Window>, ev: ptr<WinEvent>) -> i32`

Blocks until the next event and fills `ev`. Returns the event kind.

### `win_next_timeout(win: ptr<Window>, ev: ptr<WinEvent>, ms: i32) -> i32`

Like win_next, but waits at most `ms` milliseconds: when nothing happens by
then it returns EV_TIMEOUT (and `ev` is cleared). The wait costs no CPU. A
negative `ms` waits forever, like win_next. Use it for anything that moves
without user input - a clock, an animation, a blinking caret:

  var kind: i32 = win_next_timeout(&win, &ev, ui_wake_ms(&ui));
  if (kind == EV_TIMEOUT) { /* rebuild the view and win_draw it */ }

### `win_text(win: ptr<Window>, buf: ptr<u8>, cap: i64) -> i64`

Copies the text of the last EV_TEXT event into `buf` as NUL-terminated UTF-8
and returns its length in bytes (0 if `cap` is too small; 33 bytes always fit).

### `win_clipboard_set(win: ptr<Window>, text: ptr<u8>, n: i64) -> bool`

System clipboard. win_clipboard_set puts `n` bytes of UTF-8 on it;
win_clipboard_get copies its text into buf (NUL-terminated) and returns the
length, or -1 if there is no text or cap is too small.

### `win_clipboard_get(win: ptr<Window>, buf: ptr<u8>, cap: i64) -> i64`

### `win_feed_key(win: ptr<Window>, ui: ptr<Ui>, key: i32, mods: i32) -> i32`

ui_feed_key plus the clipboard: Ctrl+C copies the selection, Ctrl+X cuts it,
Ctrl+V pastes. Use this for EV_KEY events instead of ui_feed_key. It also
scrolls: PageUp / PageDown always, Up / Down / Home / End / Left / Right when no input has
focus (with an input focused they move its caret) - see ui_scroll_key.

### `win_dpi_percent(win: ptr<Window>) -> i32`

How much bigger than normal this screen wants things: 100 at 96 dpi, 150 at 144 dpi, ...
(200 on a typical high-resolution laptop). Scale the theme with it - theme_scale(&t, percent) -
and your own sizes. Always 100 where the window system does not say.

### `win_pending(win: ptr<Window>) -> bool`

### `win_time_ms() -> i64`

A steady clock in milliseconds (it only goes forward; its zero is arbitrary).

### `win_draw(win: ptr<Window>, ui: ptr<Ui>)`

Renders `ui` into the window's pixel buffer and shows it. Does nothing when
the picture did not change, except after an expose / resize, when the
existing picture is shown again.

### `win_width(win: ptr<Window>) -> i32`

### `win_height(win: ptr<Window>) -> i32`

