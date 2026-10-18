# Where this copy comes from

wjson by potatosl (MPL-2.0, see LICENSE), downloaded from https://github.com/potatosl/wjson
at commit `eab1c82` ("Cut the cost of parsing strings, and time more document shapes").

Only the Wave sources (`src/`), `vex.ws`, `LICENSE` and the README are copied. The Rust core,
the C++ layer and the Windows link driver of wjson are not needed by Wui: its Wave API is
self-contained except for one function, `wjn_console_utf8` (it switches a Windows console to
UTF-8), which `src/wjson_glue.wave` (import `wui::wjson_glue`) provides.
