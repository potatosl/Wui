# Versioning

Wui follows [Semantic Versioning](https://semver.org). `ui_version()` returns the version of the library you linked.

While the major version is `0` (now), minor releases (`0.x.0`) may change the public API; patch releases (`0.x.y`) only fix bugs. From `1.0.0` on:

- **Public API** = everything exported by `src/lib.wave` (`ui_*`, `win_*`, `Theme`, `UI_*`, `EV_*`, `KEY_*`, `MOD_*`, `OVERLAY_*`). `src/core.wave`, `render`, `font`, `gfx`, the window backends and `winplat/` are internal and may change in any release.
- **Major** (`x.0.0`): a public function, constant or struct field is removed, renamed or changes meaning. Numeric constants (`UI_P_*`, `EV_*`, `KEY_*`) never change value inside a major version.
- **Minor**: new functions, constants, widgets, backends; new `Theme` fields are appended at the end.
- **Patch**: bug fixes, rendering fixes, performance.
- The C ABI of `winplat` (`wui_win_*`) is internal but versioned together with the library: build both from the same source tree.

## Platforms

| Target | Status |
|---|---|
| Linux x86_64 / aarch64 (X11) | tested |
| Windows x86_64 and ARM64 (Win32) | compiles; never run |
| macOS Intel and Apple silicon (Cocoa) | compiles; never run |
| FreeBSD (X11) | same code as Linux; untested |

A platform is called "supported" once someone has run the examples on a real machine; until then a release note says "untested" for it.

## Wave toolchain

Wui is developed against `wavec` 0.2.1-pre-beta (std revision 5) and also builds and passes its tests with the nightly build of Wave `master` (0.0.1-alpha-dev, commit d9fe76d, std revision 5). A release states the `wavec` versions it was tested with in `README.md`.
