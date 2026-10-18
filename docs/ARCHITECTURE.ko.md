# 구조

Wui가 어떻게 짜여 있는지 설명합니다. [README](../README.ko.md)로 돌아가기.

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
