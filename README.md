<div align="center">

<img src="docs/wui.svg" alt="Wui" height="90">

<hr>

**A small declarative GUI library for Wave, written in Wave.**

Describe your window as a function of your state; Wui lays it out, paints it and shows it.

[Guide](docs/GUIDE.md) · [API](docs/API.md) · [Examples](examples) · [Building](docs/BUILDING.md) · [Changelog](CHANGELOG.md)

[![CI](https://github.com/potatosl/Wui/actions/workflows/ci.yml/badge.svg)](https://github.com/potatosl/Wui/actions/workflows/ci.yml) [![License](https://img.shields.io/github/license/potatosl/Wui?color=6654F1)](LICENSE) [![Made for Wave](https://img.shields.io/badge/made%20for-Wave-6654F1)](https://wave-lang.dev)

**English** | [한국어](README.ko.md)

</div>

After every event Wui takes the tree your code built, lays it out, paints it with its own software renderer and shows it. There is no GPU code, no
widget toolkit underneath, and nothing to install besides the Wave compiler (and libX11 on Linux).

## A taste

```wave
struct Model {
    count: i32;
}

fun update(m: ptr<Model>, msg: i32) {
    if (msg == MSG_INC) { m.count += 1; }
    if (msg == MSG_RESET) { m.count = 0; }
}

fun view(ui: ptr<Ui>, m: ptr<Model>, w: i32, h: i32) {
    ui_clear(ui);
    var card: i32 = ui_column(ui, UI_ROOT, 16, 10);      // parent, padding, gap
    ui_text(ui, card, "Hello from Wui");
    var row: i32 = ui_row(ui, card, 0, 8);
    ui_button(ui, row, "+1", MSG_INC);                   // a click comes back as MSG_INC
    ui_button(ui, row, "Reset", MSG_RESET);
    ui_layout(ui, w, h);
}
```

Buttons carry a message number instead of a callback. Your event loop passes clicks and keys to Wui,
handles the message in `update`, and calls `view` again. State that has to survive a frame (what is typed
in an input, scroll positions, focus) lives inside Wui under the ids you give. The whole counter
program, event loop included, is [`examples/counter.wave`](examples/counter.wave).

## What is in it

- **Layout**: rows and columns with padding, gaps, growing and aligning children, like a small flexbox.
- **Widgets**: text, buttons, checkboxes, radio buttons, tabs, sliders, progress bars, drop-downs,
  single-line and multi-line text inputs, scroll boxes, menus, tooltips, trees, tables, dialogs and a
  file list.
- **Text**: TrueType and OpenType fonts found on the system, with selection, clipboard and input
  methods (Hangul and other composed text on X11).
- **Look**: light and dark themes, colours written as `"#1E3A8A"`, themes as JSON files, text sizes and
  screen scaling.
- **Keyboard**: Tab order, Enter and Space, a focus ring.
- **Headless**: lay a tree out and render it into your own buffer without a window.

Everything is Wave except the Windows and macOS window code, which needs function addresses that Wave
cannot take yet; that part (which also lists directories there) is a small Rust crate (`winplat/`).

## Platforms

| | Status |
|---|---|
| Linux (X11) | works; this is where Wui is developed and tested |
| Windows (x64, ARM64) | builds, links and passes the unit tests in CI; no window has been opened on a real machine yet |
| macOS (Intel, Apple silicon) | the same |
| FreeBSD (X11) | same code as Linux, untested |

If you try it on Windows or macOS, please tell me what happens.

## Get started

You need the [Wave compiler](https://github.com/wavefnd/Wave) (`wavec`). On Linux, FreeBSD and macOS
the installer sets Wui up and gives you a `wui-wavec` wrapper that knows the link flags (read the script
first, it is short):

```
curl -fsSL https://raw.githubusercontent.com/potatosl/Wui/main/install.sh | sh
~/.wui/bin/wui-wavec run app.wave
```

In `app.wave`: `import("wui")::{Ui, ui_new, ui_column, ui_button, win_open, ...};`.

To try the examples, clone the repository and run one:

```
git clone https://github.com/potatosl/Wui && cd Wui
WUI_RUN=counter wavec --std-root=$HOME/.wave/bin/std run build.wave
```

Other examples: `widgets`, `form`, `menus`, `lists`, `files`, `theme`, `timer`, `scroll`. Windows, macOS,
manual setup and `vex` are covered in [Building and using Wui](docs/BUILDING.md).

## Learn more

- [Guide](docs/GUIDE.md): the model, the event loop, themes, focus, scrolling, overlays, the examples.
- [API reference](docs/API.md): every function.
- [Architecture](docs/ARCHITECTURE.md): how the source is organised.
- [Building and using Wui](docs/BUILDING.md): install, link flags, other platforms, tests.
- [Versioning](docs/VERSIONING.md) and the [changelog](CHANGELOG.md).

## Status

Wui is young. Before 1.0 the API can change in minor releases. Inputs are single-line (and multi-line
text areas are plain text), there is no text wrapping, and everything is drawn on the CPU. Bug reports
and ideas are welcome as issues; for security problems see [SECURITY.md](SECURITY.md).

## License

[MPL-2.0](LICENSE). [wjson](https://github.com/potatosl/wjson), used for JSON themes, is MPL-2.0 too.

<p align="center">
  <a href="https://wave-lang.dev"><img src="docs/wave.svg" alt="Wave" height="28"></a>
</p>
