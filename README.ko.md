# Wui

[English](README.md) | **한국어**

[Wave](https://github.com/wavefnd/Wave) 언어로 만든 작은 선언형 GUI 라이브러리입니다. 이벤트가 일어날 때마다
화면을 다시 기술하면(상태를 위젯 트리로 바꾸는 함수), Wui가 레이아웃을 계산하고, 픽셀 버퍼에 그려서, 창에 보여 줍니다.

```
 Wave 앱 ──► src/lib.wave ──► src/core.wave ──► src/render.wave ──► 픽셀
 모델, update,   Ui API         트리, 레이아웃,     래스터라이저          │
 view, 이벤트 루프 + win_*       입력, 스크롤        src/font.wave        ▼
                                그리기 목록         (TrueType / CFF)   src/window/x11.wave ──► libX11
```

Wui는 Wave로 작성되어 있습니다. 예외는 Windows와 macOS 창 백엔드뿐입니다. 이 둘은 OS에 함수 주소를 넘겨야 하는데
(윈도 프로시저, Cocoa 객체) Wave 0.2.x는 함수의 주소를 가져올 수 없어서, C ABI를 가진 작은 Rust 크레이트(`winplat/`)로
두었고 `src/window/win32.wave`가 이를 호출합니다. (디렉터리 목록도 이 크레이트가 맡습니다. 아래 참고.)

| 파일 | 하는 일 |
|---|---|
| `src/lib.wave` | 공개 API (`ui_*`, `win_*`, 키 / 이벤트 상수) |
| `src/core.wave` | 코어의 입구: `src/core/`의 `core_*` API를 다시 내보냄 |
| `src/core/types.wave` | 상수, `Node` / `Field` / `Ctx`, 작은 도우미 함수 |
| `src/core/field.wave`, `src/core/textarea.wave` | 텍스트 편집: UTF-8, 캐럿, 선택, 단어 이동, 여러 줄 입력 |
| `src/core/ctx.wave`, `src/core/build.wave` | 컨텍스트(노드, 텍스트 저장소, id로 찾는 상태)와 트리를 만드는 호출 |
| `src/core/arrange.wave`, `src/core/pick.wave` | flexbox 방식 레이아웃, 히트 테스트와 스크롤바 계산 |
| `src/core/widgets.wave`, `src/core/scroll.wave`, `src/core/focus.wave`, `src/core/input.wave` | 슬라이더 / 드롭다운 / 메뉴 / 툴팁, 스크롤, Tab 포커스와 캐럿 깜박임, 클릭·드래그·텍스트·키 |
| `src/core/paint.wave` | 그리기 목록 생성(위젯 종류마다 함수 하나) |
| `src/render.wave`, `src/gfx.wave` | 소프트웨어 래스터라이저: 채우기, 테두리, 둥근 도형, 선, 클립, 텍스트 |
| `src/font.wave`, `src/fontdb.wave` | TrueType(glyf)와 OpenType-CFF 읽기, 캐시가 있는 안티에일리어싱 글리프 래스터라이저, OS가 가진 폰트 찾기 (Wui는 폰트를 포함하지 않음) |
| `src/unicode.wave` | 입력창의 단어 이동에 쓰는 글자 / 숫자 표(생성됨) |
| `src/window/x11.wave` | X11 창, 이벤트, 입력기(한글 / CJK 조합), 클립보드 - `extern(c)`로 libX11 사용 |
| `src/window/win32.wave`, `winplat/` | Windows(x86_64, ARM64)와 macOS(Intel, Apple silicon) 창: Wave 래퍼가 호출하는 C ABI의 Rust 크레이트 (`platform/win32.rs` user32 / gdi32, `platform/macos.rs` Objective-C 런타임으로 쓰는 Cocoa, 프레임워크는 dlopen으로 로드). **네 타깃 모두 CI(GitHub의 Windows·macOS 러너)에서 빌드, 링크, 단위 테스트가 통과하지만, 실제 Windows나 Mac에서 창을 띄워 본 적은 아직 없습니다** |
| `src/dirlist.wave`, `src/dir/posix.wave`, `src/dir/native.wave` | 파일 선택기용 디렉터리 목록: Linux / FreeBSD는 libc `readdir`, Windows / macOS는 `winplat`(`std::fs`) |
| `src/window/none.wave` | 그 밖의 타깃용 자리표시: `win_ok()`가 false |
| `src/theme.wave`, `src/theme_json.wave` | `Theme`(색, 모서리 반지름, 여백)과, 선택적으로 wjson을 쓰는 JSON 테마 파일 |
| `src/wjson_glue.wave` | wjson의 C++ 계층 없이 wjson을 쓰게 해 주는 한 줄짜리 심볼 |
| `third_party/wjson/` | [wjson](https://github.com/potatosl/wjson), Wave용 JSON 라이브러리 (Wave 소스 사본, `SOURCE.md` 참고, `src/theme_json.wave`가 사용) |

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
리비전 5)로 Linux x86_64에서 시험했습니다. CI에서는 Windows x64 / ARM64와 macOS Intel / Apple silicon에서도 빌드, 링크, 단위 테스트를 돌립니다(아래 빌드 참고). 버전 정책: `docs/VERSIONING.md`. macOS arm64에서는 기본값 `-O0`을 유지하세요.
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

더 보기: [`docs/API.md`](docs/API.md)(레퍼런스), [`docs/VERSIONING.md`](docs/VERSIONING.md)(버전 정책).

## 모델

Wui는 정신적으로 즉시 모드입니다. **이벤트마다 트리를 다시 만들고, 그린다.**

```wave
fun view(ui: ptr<Ui>, m: ptr<Model>, w: i32, h: i32) {
    ui_clear(ui);
    var col: i32 = ui_column(ui, UI_ROOT, 16, 8);      // 부모, 여백, 간격
    ui_text(ui, col, "Count");
    ui_button(ui, col, "+1", MSG_INC);                  // 클릭하면 MSG_INC가 돌아옴
    ui_layout(ui, w, h);
}
```

프레임을 넘어 살아남아야 하는 상태(입력창에 친 글자, 스크롤 위치, 포커스, 선택)는 Wui 안에 있고, 여러분이 넘기는 id(`input_id`,
`scroll_id`)로 찾습니다. 이 id는 프레임마다 같게 유지하세요. 나머지는 모두 여러분 몫입니다: `Model`, `update(msg)`, `view()`.

### 이벤트 루프

모든 예제는 이 모양입니다(완전한 예는 `examples/scroll.wave`):

```wave
var win: Window = win_open("title", 480, 560);
var ui: Ui = ui_new();
view(...); win_draw(&win, &ui);
while (running) {
    var kind: i32 = win_next(&win, &ev);            // 다음 이벤트까지 기다림
    if (kind == EV_CLOSE) { running = false; }
    else {
        if (kind == EV_DOWN && ev.a == 1) {          // 왼쪽 버튼 누름
            var msg: i32 = ui_click(&ui, ev.x, ev.y, ev.mods);
            if (msg != UI_NO_MSG) { update(...); }
        }
        if (kind == EV_UP && ev.a == 1) { ui_release(&ui); }
        if (kind == EV_MOVE)  { ui_set_pointer(&ui, ev.x, ev.y, false); /* 누른 채면 ui_drag */ }
        if (kind == EV_WHEEL) { ui_scroll(&ui, ev.x, ev.y, ev.a); }
        if (kind == EV_TEXT)  { var n: i64 = win_text(&win, piece, 64); ui_feed_text(&ui, piece, n); }
        if (kind == EV_KEY)   { var r: i32 = win_feed_key(&win, &ui, ev.a, ev.mods); }
        if (kind == EV_EXPOSE) {
            if (!win_pending(&win)) { win_draw(&win, &ui); }   // 그림만 다시 보여 줌
        } else {
            view(...); win_draw(&win, &ui);
        }
    }
}
```

`win_draw`는 그림을 그리고, 그림이 바뀐 경우(또는 expose / resize 뒤)에만 화면에 내보냅니다.

## API

모두 `wui`에서 가져옵니다. 색은 `0xAARRGGBB`, 크기는 픽셀입니다. 전체 목록은 [`docs/API.md`](docs/API.md)에 있습니다.

**트리.** `ui_new` / `ui_free` / `ui_clear`, 컨테이너 `ui_column(parent, padding, gap)`, `ui_row(...)`, `ui_box(parent)`,
`ui_text`, `ui_button(text, msg)`, `ui_checkbox(label, checked, msg)`, `ui_input(input_id, placeholder)`,
`ui_scroll_box(scroll_id)`. 각각 노드 id를 돌려주며, 부모로 쓰거나 스타일을 줄 때 씁니다. 첫 부모는 `UI_ROOT`입니다.

**스타일** (모두 노드 id를 받음): `ui_padding`, `ui_gap`, `ui_width`, `ui_height`, `ui_grow`,
`ui_align`(`UI_START | UI_CENTER | UI_END | UI_STRETCH`), `ui_justify`(`UI_BETWEEN` 추가), `ui_direction`,
`ui_background`, `ui_border`, `ui_margin`, `ui_radius`, `ui_color`(글자색). 색은 `ui_rgb`로 글자로 쓸 수 있습니다: `ui_background(ui, box, ui_rgb("#1E3A8A"))` (`#RRGGBB`, `#AARRGGBB`, `#RGB`, `#ARGB`).
`ui_layout(w, h)`가 트리를 배치하고, `ui_rect(id)`가 노드의 사각형을 읽습니다.

**마우스.** `ui_click(x, y, mods)`는 맞힌 버튼 / 체크박스의 메시지 또는 `UI_NO_MSG`를 돌려주고, 입력창에 포커스를 주며 스크롤바 드래그를
시작합니다. 버튼을 누른 채 `ui_drag(x, y)`를 부르면 텍스트 선택이 늘어나거나 스크롤바 손잡이가 움직이고, `ui_release()`로 끝납니다.
`ui_set_pointer(x, y, down)`은 호버 / 눌림 모양을 움직입니다. `ui_hit(x, y)`는 부작용 없는 클릭입니다.

**텍스트 입력.** `ui_input_get / ui_input_set / ui_input_focus / ui_focused`, `ui_selection`, `ui_feed_text`(입력된 글자),
`ui_feed_key`(캐럿, 선택, 편집. `UI_CHANGED | UI_SUBMIT | UI_COPY | UI_CUT | UI_PASTE | UI_ACTIVATED | UI_FOCUS_MOVED`를 돌려줌).
클립보드까지 처리하는 `win_feed_key`를 권합니다. 한글 등 입력기로 조합된 글자는 `EV_TEXT`로 옵니다.

**시간.** `win_next`는 이벤트가 올 때까지 기다립니다. `win_next_timeout(win, ev, ms)`는 최대 `ms`밀리초만 기다리고 `EV_TIMEOUT`을
돌려줍니다(`poll` / `MsgWaitForMultipleObjects`에서 자므로 CPU를 쓰지 않음). 시계, 스톱워치, 애니메이션은 `EV_TIMEOUT`에서 다시 그리면
됩니다. `ms`가 음수면 무한정 기다립니다. `win_time_ms()`는 일정하게 가는 밀리초 시계입니다. 포커스가 있는 입력창의 캐럿은 깜박이며
(`Theme.caret_blink_ms`, 0이면 끔), `ui_wake_ms(ui)`는 그림이 스스로 바뀔 때까지 남은 시간(없으면 -1)을 알려 주므로
`win_next_timeout(&win, &ev, ui_wake_ms(&ui))`로 가만히 있을 때도 깜박이게 할 수 있습니다. `win_draw`와 `ui_feed_*` / `ui_click`이 Ui에
시간을 알려 줍니다(창 없이 쓸 때는 `ui_set_time`).

**테마.** 기본 위젯의 모양(색, 모서리 반지름, 여백, 체크박스와 스크롤바 크기, 휠 간격)은 `Theme`(`src/theme.wave`)입니다. Ui는
`theme_light`로 시작하고, 다른 프리셋은 `theme_dark`입니다. 현재 테마를 복사해서 필드를 바꾸고 설정합니다:
`var t: Theme = ui_theme_copy(&ui); theme_dark(&t); t.accent = 0xFF22C55E; ui_set_theme(&ui, &t);`
(`ui_theme(&ui)`는 직접 고칠 수 있는 포인터를 줍니다). 노드 하나에 준 스타일(`ui_background`, `ui_color`, `ui_radius` ...)이 테마보다 우선합니다.
wjson을 쓰면 `import("wui::theme_json")`로 테마를 JSON 파일로 읽고 쓸 수 있습니다(`theme_from_json`, `theme_to_json`):
`"light"` 또는 `"dark"`인 `"base"` 위에 일부 필드만 적고, 색은 `"#RRGGBB"` / `"#AARRGGBB"`로 씁니다.

**글자 크기, 굵기, 화면 배율.** `ui_font_size(node, px)`와 `ui_bold(node, bool)`은 그 노드와 안쪽 전부의 글자를 꾸밉니다(기본값은
`Theme.font_size`, 16px). 버튼, 입력창, 체크박스 등은 물려받은 크기로 측정하고 그립니다. 굵게는 합성입니다. 글리프를 1픽셀 어긋나게 두 번
그리므로 굵은 폰트가 필요 없습니다. 고해상도 화면에서는 `win_dpi_percent(win)`가 이 화면이 원하는 배율(96dpi는 100, 흔한 노트북 패널은 200;
X11은 `Xft.dpi`를 읽고 없으면 화면의 밀리미터 크기로 계산)을 알려 주고, `theme_scale(&theme, percent)`가 글자를 포함한 테마의 모든 픽셀 크기를
키웁니다. 노드 하나하나에 준 크기(`ui_font_size`, `ui_width`, `ui_padding` ...)는 같은 숫자로 직접 키워야 합니다.

**더 많은 위젯.** `ui_radio(label, selected, msg)` - "여럿 중 하나"를 위한 둥근 체크박스(체크박스처럼 어떤 것이 선택됐는지는 앱이 기억합니다),
`ui_tab(label, selected, msg)` - 선택된 것에 밑줄이 있는 납작한 버튼, `ui_progress(value, max)`, `ui_slider(id, min, max)` - 값이 `id`로
Ui 안에 저장되는 정수(`ui_slider_get`으로 읽고 `ui_slider_set`으로 설정, 마우스와 드래그, 좌우 / Home / End / PageUp / PageDown으로 움직임),
`ui_dropdown(id, options)` - 선택지는 줄바꿈으로 구분한 글 하나이고 선택(선택지 번호)이 Ui 안에 저장됨(`ui_dropdown_get` / `ui_dropdown_set`),
클릭이나 Enter / Space로 목록이 열리고 안에서 위 / 아래 / Enter / Escape가 동작, `ui_textarea(id, placeholder, rows)` - `ui_input`과 모든 것을
공유하는 여러 줄 입력(같은 종류의 id로 저장되는 글, 선택, 클립보드, 캐럿 깜박임)에 Enter = 줄바꿈(Ctrl+Enter = `UI_SUBMIT`), 줄 사이 위 / 아래,
줄 단위 Home / End, PageUp / PageDown, 스크롤 휠이 더해집니다. 입력창, 슬라이더, 드롭다운은 id 공간을 함께 씁니다.

**오버레이.** `ui_overlay(anchor, placement, dx, dy)`는 모든 것 위에 떠 있는 상자(대화상자, 메뉴, 툴팁)를 만듭니다. 메인 트리가 배치된 뒤에
배치되고(`OVERLAY_AT`은 창 좌표, `OVERLAY_BELOW` / `OVERLAY_ABOVE` / `OVERLAY_RIGHT`는 기준 노드 옆, 필요하면 `+ OVERLAY_MATCH_WIDTH`), 맨 나중에 그려지며,
자기 영역 위의 모든 클릭과 휠을 가져가고, Tab이 그 안에만 머물게 합니다. `OVERLAY_FULL` 오버레이는 모달 배경입니다. 돌려받은 id를 부모로 해서
다른 상자처럼 채우면 됩니다. 드롭다운의 목록은 라이브러리가 스스로 만드는 오버레이입니다.

**메뉴, 툴팁, 트리, 테이블, 대화상자.** `ui_menu_bar(parent)`는 메뉴들의 한 줄이고, `ui_menu(bar, id, "File\nOpen\nSave\n-\nQuit", msg_base)`는
목록을 여는 납작한 라벨입니다. *k*번째 항목은 버튼처럼 `msg_base + k`를 돌려줍니다(`-` 줄은 구분선, 메뉴가 열린 채 옆 메뉴 위로 가면 그쪽으로 바뀌고,
좌 / 우 / 위 / 아래 / Enter / Esc가 동작). `ui_tooltip(node, text)`는 포인터가 반 초 머물면 그 옆에 작은 상자를 보여 줍니다(시계가 필요합니다.
`win_draw`가 설정하고 `ui_wake_ms`가 루프를 깨웁니다). `ui_tree_item(parent, depth, label, state, selected, msg_toggle, msg_select)`은 트리의
들여쓰기 한 줄입니다(어떤 노드가 열려 있는지는 앱이 기억). `ui_table_row` + `ui_cell` / `ui_cell_button`은 너비가 고정된 셀의 행을 만들고, 셀보다 긴 글자는
가장자리에서 잘립니다. `ui_dialog(width)`는 어두운 배경 위의 모달 상자입니다. `DirList` / `dirlist_read(path)`는 디렉터리를(폴더 먼저, 정렬해서) 나열하고
`ui_file_list`가 보여 주며, `path_cwd` / `path_push` / `path_pop`은 경로를 다룹니다. 이것들이 모여 파일 선택기가 됩니다(`examples/files.wave` 참고).
디렉터리 읽기는 Linux / FreeBSD에서 libc, Windows / macOS에서 `winplat`의 `std::fs`입니다.

**키보드 포커스.** 입력창, 버튼, 체크박스가 포커스를 가질 수 있습니다. `win_feed_key`는 Tab / Shift+Tab으로 포커스를 옮기고(트리 순서, 끝에서 처음으로,
스크롤 상자 안에 있으면 보이도록 스크롤) Enter로 포커스된 버튼이나 체크박스를 누릅니다. `ui_feed_text`는 Space로 같은 일을 합니다. 둘 다 `UI_ACTIVATED`를
돌려주며 `ui_activated()`가 처리할 메시지입니다(클릭과 똑같습니다). 포커스 링은 키보드로 포커스가 온 동안에만 그려지고, 마우스 클릭은 링 없이 포커스를 옮깁니다.
버튼과 체크박스는 다시 만든 뒤에도 메시지 번호로 찾으므로 포커스를 받을 것마다 서로 다른 메시지를 주세요(메시지가 없는 버튼, `-1`은 Tab이 건너뜁니다).
`ui_focus_next(backward)`, `ui_focused_msg()`, `ui_input_focus(id)`.

**스크롤.** `ui_scroll_box(parent, scroll_id)`는 자식을 잘라서 보여 주고 넘치면 스크롤바를 보여 주는 컬럼입니다. 높이를 주세요(`ui_height` 또는 `ui_grow`).
`ui_scroll_box_xy`는 내용이 상자보다 넓으면 옆으로도 스크롤하며 아래쪽에 두 번째 스크롤바가 생깁니다. 너비도 필요합니다(컬럼에서 늘리거나, 행에서 `ui_grow`,
또는 `ui_width`). 휠: 세로 휠은 `ui_scroll(x, y, notches)`, `EV_HWHEEL`(가로 휠이나 터치패드 스와이프)과 Shift를 누른 세로 휠은 `ui_scroll_h(x, y, notches)`.
키: `ui_scroll_key(key)`(`win_feed_key`가 대신 부릅니다. PageUp / PageDown은 항상, 위 / 아래 / Home / End / 좌 / 우는 포커스된 입력창이 없을 때).
스크롤바: 손잡이를 끌거나 트랙을 누릅니다. 코드로: `ui_scroll_set(scroll_id, px)` / `ui_scroll_set_x`(아주 큰 값은 "끝까지"), `ui_scroll_get` / `ui_scroll_get_x`.
중첩된 스크롤 상자도 됩니다. 포인터 아래에서 아직 움직일 수 있는 가장 안쪽 상자가 입력을 받습니다. 스크롤로 가려진 위젯은 누를 수 없습니다.

**창.** `win_open`, `win_ok`, `win_close`, `win_next`(기다림, `EV_*`를 돌려줌), `win_pending`, `win_text`, `win_draw`, `win_width`, `win_height`,
`win_clipboard_get / win_clipboard_set`. `WinEvent`는 `kind, x, y, a, width, height, mods`를 담습니다.

**이벤트.** `EV_CLOSE, EV_MOVE, EV_DOWN, EV_UP, EV_RESIZE, EV_EXPOSE, EV_KEY, EV_TEXT, EV_WHEEL, EV_HWHEEL, EV_TIMEOUT`.
키: `KEY_BACKSPACE, KEY_TAB, KEY_ENTER, KEY_ESCAPE, KEY_DELETE, KEY_LEFT/RIGHT/UP/DOWN, KEY_HOME, KEY_END, KEY_PAGE_UP, KEY_PAGE_DOWN`,
수정 키 `MOD_SHIFT, MOD_CTRL, MOD_ALT`.

창 없이 쓰는 것(헤드리스)도 됩니다. 트리를 만들고 `ui_layout`을 부른 뒤 `ui_render`로 자기 버퍼에 그리면 됩니다. `*_headless.wave` 예제를 보세요.

## 예제

| 예제 | 보여 주는 것 | 실행 (`WUI_RUN=<이름>`) |
|---|---|---|
| `counter` | 버튼, 호버 / 눌림 모양, 키보드 포커스(Tab, Enter, Space), 가장 작은 model-update-view 루프 | `WUI_RUN=counter` |
| `typing` | 키보드 글자와 입력기(한글), Backspace / Enter | `WUI_RUN=typing` |
| `theme` | 실행 중 테마 바꾸기: 밝게 / 어둡게, 강조색, 둥근 / 각진 모서리, 키보드 포커스 링 | `WUI_RUN=theme` |
| `timer` | `win_next_timeout`: 켜져 있는 동안 50ms마다 다시 그리고 멈추면 자는 스톱워치 | `WUI_RUN=timer` |
| `widgets` | 탭, 라디오 버튼, 슬라이더 + 진행 막대, 드롭다운, 텍스트 영역, 모달 대화상자(오버레이) | `WUI_RUN=widgets` |
| `text_styles` | 글자 크기, 굵게, 고해상도 화면용으로 키운 테마 | `WUI_RUN=text_styles` |
| `menus` | 메뉴 바(File / Edit / Help)와 툴팁 | `WUI_RUN=menus` |
| `lists` | 열고 닫는 트리, 헤더와 줄무늬 행과 스크롤이 있는 테이블 | `WUI_RUN=lists` |
| `files` | 파일 대화상자: 디렉터리의 모달 목록, 폴더, 위로, 열기 / 취소 | `WUI_RUN=files` |
| `form` | 입력창 둘, 체크박스, Tab 포커스, 선택, 클립보드 | `WUI_RUN=form` |
| `scroll` | 스크롤 상자: 휠, 손잡이 드래그, PageUp / PageDown, 맨 위 / 맨 아래 | `WUI_RUN=scroll` |
| `*_headless` | 같은 트리를 창 없이 배치하고 그린 것 | `WUI_RUN=counter_headless` |

## 한계

- 스크롤 상자만 넘칠 수 있습니다. 일반 상자는 자식을 자기 크기 안에 두며, 줄바꿈이나 가로로 흐르는 레이아웃은 없습니다.
- 입력창은 한 줄이며 최대 4096바이트입니다. 아주 큰 클립보드 전송(X11 INCR)은 지원하지 않습니다.
- 글자와 위젯은 소프트웨어 래스터라이저가 그리며 GPU 경로는 없습니다.

## 테스트

`WUI_TASK=wtest`는 Wave 단위 테스트(`tests/core_test.wave`, 891개 검사: 레이아웃, 히트 테스트, 입력 편집과 선택, 그리기, 양방향 스크롤 상자,
스크롤바 드래그, 스크롤 키, 메뉴, 툴팁, 드롭다운, 오버레이, 슬라이더)를 실행합니다. `tests/dirlist_test.wave`(32개 검사)는 디렉터리 목록과 경로
도우미를 확인하고, `winplat/`의 `cargo test`는 그 디렉터리 함수를 확인합니다. 테스트는 폰트가 없는 텍스트 엔진을 써서 글자 하나가 8px 너비,
한 줄이 16px 높이입니다. `WUI_TASK=xtest`는 X11 백엔드 테스트(`tests/window_x11_test.wave`)를 실행합니다: 키와 수정 키 매핑, 텍스트 이벤트,
두 창 사이의 실제 클립보드 왕복(숨겨진 창을 쓰고 끝나면 시스템 클립보드를 되돌림). 디스플레이가 없으면 화면 관련 테스트는 건너뜁니다.

## 라이선스

MPL-2.0. [`LICENSE`](LICENSE)를 보세요.
