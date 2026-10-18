#!/bin/sh
# SPDX-License-Identifier: MPL-2.0
#
# Wui installer for Linux, FreeBSD and macOS.
#
#   curl -fsSL https://raw.githubusercontent.com/potatosl/Wui/main/install.sh | sh
#
# Downloads the Wui sources into ~/.wui and writes a small wrapper, `wui-wavec`, that compiles a Wave
# program against it with the right link flags for this system:
#
#   ~/.wui/bin/wui-wavec app.wave -o app       # build
#   ~/.wui/bin/wui-wavec run app.wave          # build and run
#
# It does not install the tools Wui needs (the Wave compiler, and on macOS Rust's cargo); if one is
# missing it says how to get it and stops. Read it before piping it into a shell; it is short.
# Settings (environment):
#   WUI_REF      git tag, branch or commit to install                     (default: main)
#   WUI_DIR      where to install                                         (default: ~/.wui)
#   WUI_SHA256   expected SHA-256 of the source tarball; the install stops if the download differs
#                (the actual value is always printed)
#   WUI_TARBALL  use this local file (or URL) instead of downloading
#   WAVE_STD     Wave's standard library folder, if wavec cannot find it
#
# To remove Wui, delete the install folder.

set -eu

REPO="potatosl/Wui"
REF="${WUI_REF:-main}"
DIR="${WUI_DIR:-$HOME/.wui}"

say() { printf '%s\n' "$*"; }
die() { printf 'wui install: %s\n' "$*" >&2; exit 1; }
need() { command -v "$1" > /dev/null 2>&1; }

case "${1:-}" in
    -h | --help)
        sed -n '4,26p' "$0" 2> /dev/null | sed 's/^# \{0,1\}//' || true
        exit 0
        ;;
esac

OS="$(uname -s)"
case "$OS" in
    Linux | FreeBSD | Darwin) ;;
    *) die "this installer supports Linux, FreeBSD and macOS. On Windows download the sources (see the README) and build with wavec and cargo." ;;
esac

[ -n "$DIR" ] && [ "$DIR" != "/" ] || die "WUI_DIR must be a real folder"

need curl || die "curl is required"
need tar || die "tar is required"
need wavec || die "the Wave compiler (wavec) is not on PATH. Install it from https://github.com/wavefnd/Wave and open a new shell."
if [ "$OS" = "Darwin" ]; then
    need cargo || die "macOS needs Rust (cargo) to build Wui's window crate: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
fi

sha256_of() {
    if need sha256sum; then
        sha256sum "$1" | cut -d ' ' -f 1
    elif need shasum; then
        shasum -a 256 "$1" | cut -d ' ' -f 1
    else
        echo "unavailable"
    fi
}

# Wave's standard library: wavec looks in a fixed place; installers do not always put it there.
STD_ROOT=""
if [ -n "${WAVE_STD:-}" ]; then
    STD_ROOT="$WAVE_STD"
else
    default_std="$(wavec print std-path 2> /dev/null || true)"
    if [ -z "$default_std" ] || [ ! -d "$default_std" ]; then
        for candidate in "$HOME/.wave/bin/std" "$HOME/wave/std" "$HOME/.wave/std" /usr/local/lib/wave/std; do
            if [ -d "$candidate" ]; then
                STD_ROOT="$candidate"
                break
            fi
        done
        [ -n "$STD_ROOT" ] || die "wavec cannot find Wave's standard library. Set WAVE_STD to the folder that holds it (often ~/.wave/bin/std)."
    fi
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

tarball="$tmp/wui.tar.gz"
if [ -n "${WUI_TARBALL:-}" ]; then
    case "$WUI_TARBALL" in
        http://* | https://*) curl -fsSL -o "$tarball" "$WUI_TARBALL" ;;
        *) cp "$WUI_TARBALL" "$tarball" ;;
    esac
    say "using $WUI_TARBALL"
else
    url="https://codeload.github.com/$REPO/tar.gz/$REF"
    say "downloading $url"
    curl -fsSL -o "$tarball" "$url" || die "could not download $url (is '$REF' a tag, branch or commit of $REPO?)"
fi

actual="$(sha256_of "$tarball")"
say "sha256 $actual"
if [ -n "${WUI_SHA256:-}" ] && [ "$WUI_SHA256" != "$actual" ]; then
    die "the download does not match WUI_SHA256 ($WUI_SHA256); nothing was installed"
fi

mkdir -p "$tmp/src"
tar -xzf "$tarball" -C "$tmp/src" --strip-components=1 || die "could not unpack the archive"
[ -f "$tmp/src/src/lib.wave" ] || die "the archive does not look like Wui (no src/lib.wave)"

# Replace an earlier install, but only a folder this script made (or an empty / missing one).
if [ -e "$DIR" ] && [ ! -f "$DIR/.wui-install" ] && [ -n "$(ls -A "$DIR" 2> /dev/null)" ]; then
    die "$DIR exists and was not made by this installer; choose another WUI_DIR or remove it"
fi
rm -rf "$DIR"
mkdir -p "$DIR/bin"
cp -R "$tmp/src" "$DIR/wui"
printf '%s\n' "$REF" > "$DIR/.wui-install"

LINK="--link=:libX11.so.6 --link=m --link=c"
if [ "$OS" = "Darwin" ]; then
    say "building the window crate with cargo (first install only)"
    cargo build --release --quiet --manifest-path "$DIR/wui/winplat/Cargo.toml" --target-dir "$DIR/cargo" \
        || die "cargo could not build winplat"
    LINK="$DIR/cargo/release/libwui_win.a --link=System --link=c --link=m"
fi

STD_FLAG=""
[ -z "$STD_ROOT" ] || STD_FLAG="--std-root=$STD_ROOT"

cat > "$DIR/bin/wui-wavec" << WRAPPER
#!/bin/sh
# Compiles a Wave program against the Wui installed in $DIR.
#   wui-wavec app.wave -o app        wui-wavec run app.wave
mode=build
if [ "\${1:-}" = "run" ]; then mode=run; shift; fi
exec wavec $STD_FLAG \$mode "\$@" --dep=wui="$DIR/wui" --dep=wjson="$DIR/wui/third_party/wjson" $LINK
WRAPPER
chmod +x "$DIR/bin/wui-wavec"

say ""
say "Wui installed in $DIR"
say "  compile a program:  $DIR/bin/wui-wavec app.wave -o app"
say "  run it directly:    $DIR/bin/wui-wavec run app.wave"
say "  add to PATH:        export PATH=\"$DIR/bin:\$PATH\""
if [ "$OS" != "Darwin" ]; then
    say "(programs need libX11.so.6 at run time)"
fi
