# opencore-amrnb

[![crates.io](https://img.shields.io/crates/v/opencore-amrnb.svg)](https://crates.io/crates/opencore-amrnb)
[![docs.rs](https://docs.rs/opencore-amrnb/badge.svg)](https://docs.rs/opencore-amrnb)

Vendored [opencore-amr-nb](https://sourceforge.net/projects/opencore-amr/)
(Apache 2.0, by Martin Storsjö / PacketVideo, the AMR-NB implementation
shipped in Android AOSP and used by ffmpeg/gstreamer) compiled to a
standalone WebAssembly artifact via [Emscripten](https://emscripten.org/).
Originally written for [Voicetastic](https://github.com/voicetastic)'s
browser client, but useful for any Rust + wasm project that needs AMR-NB
encode/decode.

The source under [`vendor/`](./vendor) is **unmodified** upstream — see
[`vendor/LICENSE`](./vendor/LICENSE). This crate adds only:

- [`build.rs`](./build.rs) — drives `emcc` over the C source on wasm32
  targets (no-op on native), with the same exclusion list the upstream
  Makefile.am uses.
- [`src/lib.rs`](./src/lib.rs) — `wasm_module_bytes()` hands the resulting
  `.wasm` to consumers.

The crate version tracks upstream opencore-amr releases (currently
`0.1.6`, Sept 2017 — the last tagged upstream release). Local changes on
top of the build glue, if any, are tagged with semver build metadata
(e.g. `0.1.6+vt.1`).

## Usage

```toml
[dependencies]
opencore-amrnb = "0.1"
```

```rust
// On wasm32 only: bytes of a standalone `.wasm` module exporting the
// AMR-NB C API (`Encoder_Interface_*`, `Decoder_Interface_*`,
// plus `malloc` / `free`). Instantiate it from JS — see the JS shim
// in voicetastic-core's `codec/amrnb_shim.js` for a working example.
#[cfg(target_arch = "wasm32")]
let bytes: &'static [u8] = opencore_amrnb::wasm_module_bytes();
```

The exported wasm imports one symbol — `env.emscripten_notify_memory_growth(idx: u32)` — which you supply as a no-op in the JS instantiation `importObject`.

## Build requirements

Only does work when the consuming crate targets `wasm32-unknown-unknown`.
On native (desktop / Android) the build script returns early — the native
AMR-NB path stays the existing `#[link(name = "opencore-amrnb")]` system
link, so you keep using your distro's `libopencore-amrnb` package there.

For the wasm build you need Emscripten in `PATH`:

```sh
# Arch:
sudo pacman -S emscripten
source /etc/profile.d/emscripten.sh

# Debian/Ubuntu:
sudo apt install emscripten

# Otherwise:
# https://emscripten.org/docs/getting_started/downloads.html
```

Then any consumer's `cargo build --target wasm32-unknown-unknown` will
run this build script automatically; the resulting `opencore_amrnb.wasm`
(~145 KB) lands under `OUT_DIR` and is read by `wasm_module_bytes()` via
`include_bytes!`.

## Wire-compat note

The wasm artifact and the native shared library compile from the same
opencore-amr source, so the bytes on the wire are identical regardless of
which path encoded them. A browser client sending AMR-NB and a native
(desktop / Android) receiver decoding it interoperate without ceremony.

## License

- This crate's build glue (`build.rs`, `src/lib.rs`, `Cargo.toml`,
  `README.md`): Apache-2.0.
- The vendored opencore-amr source under `vendor/`: Apache-2.0, © 2007
  PacketVideo Corporation, with portions by Martin Storsjö. See
  [`vendor/LICENSE`](./vendor/LICENSE).
- A copy of the Apache-2.0 text also lives at [`LICENSE`](./LICENSE) at
  the crate root for crates.io metadata.

Note that AMR-NB is also covered by patents that have been declared
essential to the 3GPP TS 26.0xx series. As of 2024, the bulk of those
patents have expired (the codec was standardized in 1999 with 20-year
patent terms), but if you ship a product, verify the situation in your
jurisdiction — Apache-2.0 explicitly does not grant patent rights from
third parties.
