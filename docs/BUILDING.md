# Building and using Wui

Installing, linking, building the examples and the tests. Back to the [README](../README.md).

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
links and runs the unit tests on Windows x64 / ARM64 and macOS Intel / Apple silicon (see the Build section below). Versioning: [`VERSIONING.md`](VERSIONING.md). On macOS arm64 keep the default `-O0`:
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

More: [`API.md`](API.md) (reference), [`VERSIONING.md`](VERSIONING.md) (version policy).
