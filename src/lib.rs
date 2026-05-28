//! Vendored opencore-amr-nb compiled to a standalone wasm artifact for the
//! browser client. See `vendor/` for the original source (Apache 2.0).
//!
//! Native targets do nothing here — desktop's path keeps its existing
//! `#[link(name = "opencore-amrnb")]` system link in
//! `voicetastic-core::codec::imp`. On `wasm32` the build script runs
//! `emcc` (Emscripten) to produce a standalone `.wasm` that exports the
//! six `Encoder_Interface_*` / `Decoder_Interface_*` functions; the bytes
//! are baked into this crate via [`wasm_module_bytes`] and the web driver
//! instantiates them at runtime through a small JS shim.

#![allow(rustdoc::broken_intra_doc_links)]

/// The standalone wasm artifact emitted by `build.rs` from the vendored C
/// source. Loaded by the browser driver as a `WebAssembly.Module`. The
/// module exports:
///
/// - `Encoder_Interface_init(dtx: i32) -> *mut`
/// - `Encoder_Interface_Encode(state, mode, speech, out, forceSpeech) -> i32`
/// - `Encoder_Interface_exit(state)`
/// - `Decoder_Interface_init() -> *mut`
/// - `Decoder_Interface_Decode(state, payload, speech, bfi)`
/// - `Decoder_Interface_exit(state)`
/// - `malloc`, `free`, `memory`, `_initialize`
///
/// And takes one import: `env.emscripten_notify_memory_growth(idx: u32)`,
/// which the JS shim provides as a no-op.
#[cfg(target_arch = "wasm32")]
pub fn wasm_module_bytes() -> &'static [u8] {
    include_bytes!(concat!(env!("OUT_DIR"), "/opencore_amrnb.wasm"))
}
