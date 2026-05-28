# opencore-amrnb-src

Vendored [opencore-amr-nb](https://sourceforge.net/projects/opencore-amr/)
(Apache 2.0, by Martin Storsjö / PacketVideo) compiled to a standalone
WebAssembly artifact via [Emscripten](https://emscripten.org/). Used by
[Voicetastic](https://git.cha-sam.re/voicetastic)'s browser client for
AMR-NB encode + decode.

The source under [`vendor/`](./vendor) is **unmodified** upstream — see
[`vendor/LICENSE`](./vendor/LICENSE). This crate adds only the build script
([`build.rs`](./build.rs)) that drives `emcc` over the C source, and a
minimal [`src/lib.rs`](./src/lib.rs) that hands the resulting wasm bytes to
consumers via `wasm_module_bytes()`.

## Build

Only does work when the consuming crate targets `wasm32` — on native
(desktop / Android) the build script returns early. The native AMR-NB
path stays the existing `#[link(name = "opencore-amrnb")]` system link.

Requirements for the wasm build:

- Emscripten (`emcc` in `PATH`). On Arch:
  ```
  sudo pacman -S emscripten
  source /etc/profile.d/emscripten.sh
  ```

Then any consumer's `cargo build --target wasm32-unknown-unknown` will run
this build script automatically; the resulting `opencore_amrnb.wasm`
(~145 KB) lands under `OUT_DIR` and is read by `wasm_module_bytes()` via
`include_bytes!`.

## Wire-compat note

The wasm artifact and the native shared library compile from the same
opencore-amr source, so the bytes on the wire are identical regardless of
which path encoded them. A Voicetastic browser client sending AMR-NB and a
desktop / Android receiver decoding it interoperate without ceremony.

## Why a separate repo

Voicetastic's main repo doesn't need to carry ~3.6 MB of vendored opencore
source on every clone; the crate has its own life cycle (opencore-amr last
released in 2017) and might be useful to other Rust + wasm projects.
