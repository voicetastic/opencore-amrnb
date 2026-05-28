//! Builds the vendored opencore-amr-nb to a standalone wasm artifact via
//! emscripten. Only runs when the consuming crate targets wasm32 — on
//! native (the desktop/Android path) we don't compile anything here, the
//! existing `#[link(name = "opencore-amrnb")]` directive in
//! `voicetastic-core::codec::imp` continues to link the system library.

use std::path::PathBuf;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=vendor");
    println!("cargo:rerun-if-changed=build.rs");

    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() != Ok("wasm32") {
        return;
    }

    if Command::new("emcc").arg("--version").output().is_err() {
        panic!(
            "opencore-amrnb: building for wasm32 but `emcc` (emscripten) \
             is not in PATH. Install it (`pacman -S emscripten` on Arch) and \
             source its profile (`source /etc/profile.d/emscripten.sh`)."
        );
    }

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let vendor = manifest.join("vendor");
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let obj_dir = out_dir.join("amr_obj");
    std::fs::create_dir_all(&obj_dir).unwrap();

    // Include paths mirror the upstream Makefile.am exactly.
    let inc = [
        "amrnb",
        "amr_nb/common/include",
        "amr_nb/enc/include",
        "amr_nb/enc/src",
        "amr_nb/dec/include",
        "amr_nb/dec/src",
        "common_shared",
        "oscl",
    ];
    let inc_args: Vec<String> = inc.iter().map(|p| format!("-I{}", vendor.join(p).display())).collect();

    // Per Makefile.am: these are excluded (duplicated elsewhere or unused).
    let exclude = |rel: &str| -> bool {
        let stem = std::path::Path::new(rel).file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if rel.starts_with("amr_nb/common/src/") {
            matches!(
                stem,
                "bits2prm" | "copy" | "div_32" | "l_abs" | "r_fft" | "vad1" | "vad2"
            )
        } else if rel.starts_with("amr_nb/dec/src/") {
            matches!(stem, "decoder_gsm_amr" | "pvgsmamrdecoder")
        } else if rel.starts_with("amr_nb/enc/src/") {
            stem == "gsmamr_encoder_wrapper"
        } else {
            false
        }
    };

    // Collect source files: the C wrapper + each amr_nb subdir's *.cpp.
    let mut sources = vec![vendor.join("wrapper.cpp")];
    for sub in ["amr_nb/dec/src", "amr_nb/enc/src", "amr_nb/common/src"] {
        let dir = vendor.join(sub);
        for entry in std::fs::read_dir(&dir).expect("read source dir") {
            let p = entry.unwrap().path();
            if p.extension().and_then(|e| e.to_str()) != Some("cpp") {
                continue;
            }
            let rel = format!("{}/{}", sub, p.file_name().unwrap().to_string_lossy());
            if exclude(&rel) {
                continue;
            }
            sources.push(p);
        }
    }

    // Compile each .cpp as C99 (matches upstream's COMPILE_AS_C mode — avoids
    // the OSCL C++ headers that aren't present in the public source tree).
    let mut objects: Vec<PathBuf> = Vec::with_capacity(sources.len());
    for src in &sources {
        let stem = src.file_stem().unwrap().to_string_lossy().to_string();
        let obj = obj_dir.join(format!("{stem}.o"));
        let status = Command::new("emcc")
            .args(&inc_args)
            .args(["-x", "c", "-std=c99", "-O2", "-c"])
            .arg(src)
            .arg("-o")
            .arg(&obj)
            .status()
            .expect("emcc invocation failed");
        if !status.success() {
            panic!("emcc failed compiling {}", src.display());
        }
        objects.push(obj);
    }

    // Link a standalone wasm artifact with the AMR-NB C API exported.
    let wasm_out = out_dir.join("opencore_amrnb.wasm");
    let exports = "_Encoder_Interface_init,_Encoder_Interface_exit,_Encoder_Interface_Encode,\
                   _Decoder_Interface_init,_Decoder_Interface_exit,_Decoder_Interface_Decode,\
                   _malloc,_free";
    let mut link = Command::new("emcc");
    link.arg("-O2")
        .arg("-sSTANDALONE_WASM")
        .arg(format!("-sEXPORTED_FUNCTIONS={exports}"))
        .arg("-sEXPORTED_RUNTIME_METHODS=")
        .arg("-sALLOW_MEMORY_GROWTH=1")
        .arg("--no-entry")
        .args(&objects)
        .arg("-o")
        .arg(&wasm_out);
    let status = link.status().expect("emcc link failed");
    if !status.success() {
        panic!("emcc link failed");
    }
    println!("cargo:rustc-env=OPENCORE_AMRNB_WASM={}", wasm_out.display());
}
