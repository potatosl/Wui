# 빌드와 사용

설치, 링크, 예제 빌드, 테스트를 설명합니다. [README](../README.ko.md)로 돌아가기.

## 내 프로젝트에서 쓰기

Wui는 Wave 소스 폴더입니다. 쉬운 방법은 설치 스크립트입니다(Linux, FreeBSD, macOS. 짧으니 먼저 읽어 보세요).

```
curl -fsSL https://raw.githubusercontent.com/potatosl/Wui/main/install.sh | sh
~/.wui/bin/wui-wavec run app.wave          # 또는: wui-wavec app.wave -o app
```

소스를 `~/.wui`에 두고, 이 시스템에 맞는 `--dep`와 링크 플래그를 붙여 주는 `wavec` 래퍼 `wui-wavec`를 만듭니다(macOS에서는
먼저 cargo로 창 크레이트를 빌드합니다). 손으로 하려면 소스를 내려받아(또는 저장소를 클론해서) `wavec`에 `--dep`로 알려 주면 됩니다.

```
curl -L https://github.com/potatosl/Wui/archive/refs/heads/main.tar.gz | tar xz            # -> Wui-main/
wavec build app.wave --dep=wui=./Wui-main --link=:libX11.so.6 --link=m --link=c -o app     # Linux / FreeBSD
```

`app.wave`에는 `import("wui")::{Ui, ui_new, ui_column, ui_button, win_open, ...};`처럼 씁니다. 패키지 이름은
반드시 `wui`여야 합니다(라이브러리가 자기 파일을 `wui::...`로 가져옵니다). 선택 기능은 추가 설정이 필요합니다.
`import("wui::theme_json")`(JSON 테마 파일)을 쓰려면 `--dep=wjson=./Wui-main/third_party/wjson`도 줘야 합니다.

**Windows와 macOS**는 먼저 창 크레이트(Rust, `cargo`)를 빌드하고 그 라이브러리를 링크해야 합니다.

```
cargo build --release --manifest-path Wui-main/winplat/Cargo.toml --target-dir build/cargo
wavec build app.wave --dep=wui=./Wui-main build/cargo/release/wui_win.lib \
      --link=user32 --link=gdi32 --link=kernel32 --link=ntdll --link=userenv --link=ws2_32 --link=dbghelp -o app   # Windows
wavec build app.wave --dep=wui=./Wui-main build/cargo/release/libwui_win.a --link=System --link=c --link=m -o app    # macOS
```

(`build.wave`의 `WUI_TASK=...`가 예제에 대해 이 작업을 해 줍니다. 직접 빌드 스크립트를 만들 때는 `add_link_flags`를 참고하세요.)

**vex.** 경로(또는 git) 의존성은 해석과 컴파일까지는 됩니다.
`vex.ws`에 `dependencies = [ { name = "wui", path = "../Wui-main" } ]`처럼 씁니다. 하지만 `vex`는 링크 플래그를 넘길
방법이 없고 Wave에는 소스 안의 링크 지시자가 없어서, libX11 / `wui_win`을 링크하는 마지막 단계가
("undefined symbol: XOpenDisplay") 실패합니다. 그런 기능이 생기기 전까지는 위처럼 `wavec`로 직접 빌드하세요.
(`vex`는 `wavec`의 std가 `~/.wave/lib/wave/std`에 있다고 가정합니다. 설치 위치가 다르면 `VEX_WAVEC`를 지정하세요.)

**컴파일러.** `wavec` 0.2.1-pre-beta와 Wave `master`의 nightly 빌드(`0.0.1-alpha-dev`, 커밋 d9fe76d, std 호환
리비전 5)로 Linux x86_64에서 시험했습니다. CI에서는 Windows x64 / ARM64와 macOS Intel / Apple silicon에서도 빌드, 링크, 단위 테스트를 돌립니다(아래 빌드 절 참고). 버전 정책: [`VERSIONING.md`](VERSIONING.md). macOS arm64에서는 기본값 `-O0`을 유지하세요.
`-O2`면 Wave std의 `std::mem::alloc` 안에서 실패합니다(Wave 이슈 #933, wjson CI에서 발견). CI는 push마다 돕니다(`.github/workflows/ci.yml`).
Windows와 macOS 작업도 네이티브로 빌드, 링크, 테스트합니다.

**배포용 압축 파일 만들기.** `WUI_TASK=package wavec --std-root=$HOME/.wave/bin/std run build.wave`가
`dist/wui-<version>.tar.gz`를 만듭니다(`WUI_VERSION`으로 버전 지정). 라이브러리, `winplat/`, 문서, 예제, 테스트,
wjson 사본, 라이선스가 들어가고 빌드 결과물은 들어가지 않습니다.

## 빌드

```
wavec --std-root=$HOME/.wave/bin/std run build.wave                 테스트 실행
WUI_TASK=wtest     ... run build.wave     Wave 단위 테스트만
WUI_TASK=jtest     ... run build.wave     JSON 테마 테스트만 (wjson)
WUI_TASK=xtest     ... run build.wave     X11 백엔드 테스트만 (창은 화면에 뜨지 않음)
WUI_TASK=examples  ... run build.wave     모든 예제 빌드
WUI_TASK=bench     ... run build.wave     프레임 시간 벤치마크
WUI_TASK=check WUI_TARGET=<triple> ...    다른 OS / CPU용으로 전부 컴파일 (오브젝트만)
WUI_RUN=scroll      ... run build.wave     examples/scroll.wave를 빌드해서 실행
```

`wavec`가 필요하고, Linux에서는 `libX11.so.6`(개발 패키지가 아니라 라이브러리 자체)이 필요합니다. wui를 쓰는 프로그램은
`--link=:libX11.so.6 --link=m --link=c`로 링크합니다. 심볼릭 링크 `libX11.so`는 개발 패키지에만 있습니다.
Windows와 macOS에서는 빌드 드라이버가 먼저 cargo로 `winplat/`을 빌드해서 링크합니다(Windows: `wui_win.lib` + Rust std가 요구하는
시스템 라이브러리, macOS: `libwui_win.a` + `libSystem`). Windows ARM64와 macOS는 같은 명령으로 네이티브 빌드되며,
cargo가 호스트 타깃을 고릅니다.

다른 OS에서의 교차 확인(링크를 하지 않으므로 SDK 불필요): `WUI_TASK=check WUI_TARGET=aarch64-pc-windows-msvc ...`
(`x86_64-pc-windows-msvc`, `aarch64-apple-darwin`, `x86_64-apple-darwin`도 가능)는 그 타깃용으로 `winplat/`을 빌드하고
모든 예제를 오브젝트로 컴파일합니다. 링크와 실행은 해당 기기(또는 SDK)가 필요합니다.

macOS 제한: 입력기 조합 아직 없음(한글 / CJK 입력은 `characters`가 주는 것만 전달됨), 메뉴 바 없음(Cmd+Q는 창 닫기),
버퍼가 기기 픽셀 단위(Retina에서 `win_dpi_percent()`는 200이므로 `theme_scale`로 테마를 키우세요).
Windows 제한: 프로세스가 아직 DPI를 인식하지 않음(`win_dpi_percent()`는 100).

더 보기: [`API.md`](API.md)(레퍼런스), [`VERSIONING.md`](VERSIONING.md)(버전 정책).
