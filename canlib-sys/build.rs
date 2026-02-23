use std::env;
use std::path::PathBuf;

fn main() {
    let sdk_dir = find_sdk_dir();

    // Tell cargo to link against canlib
    if let Some(ref sdk) = sdk_dir {
        let lib_dir = if cfg!(target_os = "windows") {
            if cfg!(target_pointer_width = "64") {
                sdk.join("Lib").join("x64")
            } else {
                sdk.join("Lib").join("MS")
            }
        } else {
            sdk.join("lib")
        };
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
    }

    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=canlib32");
    } else {
        println!("cargo:rustc-link-lib=canlib");
    }

    // Determine include path
    let include_dir = sdk_dir.as_ref().map(|sdk| {
        if cfg!(target_os = "windows") {
            sdk.join("INC")
        } else {
            sdk.join("include")
        }
    });

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    let bindings_path = out_path.join("bindings.rs");

    // Try to generate bindings with bindgen if SDK is found and libclang is available.
    // bindgen panics if libclang is missing, so we catch that.
    if let Some(ref inc) = include_dir {
        let inc_path = inc.display().to_string();
        let result = std::panic::catch_unwind(|| {
            bindgen::Builder::default()
                .header("wrapper.h")
                .clang_arg(format!("-I{}", inc_path))
                .allowlist_function("can.*")
                .allowlist_function("kv.*")
                .allowlist_type("can.*")
                .allowlist_type("kv.*")
                .allowlist_var("can.*")
                .allowlist_var("kv.*")
                .prepend_enum_name(false)
                .derive_debug(true)
                .derive_default(true)
                .generate()
        });

        match result {
            Ok(Ok(bindings)) => {
                bindings
                    .write_to_file(&bindings_path)
                    .expect("Failed to write bindings");
                return;
            }
            Ok(Err(e)) => {
                println!(
                    "cargo:warning=bindgen generation failed: {}. Using manual declarations.",
                    e
                );
            }
            Err(_) => {
                println!(
                    "cargo:warning=bindgen panicked (libclang not found?). Using manual declarations. \
                     Install LLVM/Clang or set LIBCLANG_PATH to enable auto-generated bindings."
                );
            }
        }
    } else {
        println!(
            "cargo:warning=Kvaser CANLib SDK not found. Set CANLIB_SDK_DIR or install the SDK. Using manual declarations only."
        );
    }

    // Write an empty bindings file — the manual extern blocks in lib.rs provide all needed symbols.
    std::fs::write(
        &bindings_path,
        "// Auto-generated: bindgen unavailable, using manual declarations.\n",
    )
    .expect("Failed to write empty bindings file");
}

fn find_sdk_dir() -> Option<PathBuf> {
    // 1. Check environment variable
    if let Ok(dir) = env::var("CANLIB_SDK_DIR") {
        let path = PathBuf::from(dir);
        if path.exists() {
            return Some(path);
        }
    }

    // 2. Windows default install location
    if cfg!(target_os = "windows") {
        let candidates = [
            r"C:\Program Files (x86)\Kvaser\Canlib",
            r"C:\Program Files\Kvaser\Canlib",
            r"C:\Program Files (x86)\Kvaser\CANlib SDK",
            r"C:\Program Files\Kvaser\CANlib SDK",
        ];
        for candidate in &candidates {
            let path = PathBuf::from(candidate);
            if path.exists() {
                return Some(path);
            }
        }
    }

    // 3. Linux: headers typically in /usr/include, libs in /usr/lib
    if cfg!(target_os = "linux") {
        let path = PathBuf::from("/usr");
        if path.join("include").join("canlib.h").exists() {
            return Some(path);
        }
    }

    None
}
