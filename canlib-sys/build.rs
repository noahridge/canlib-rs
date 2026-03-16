use std::env;
use std::path::PathBuf;

fn main() {
    let sdk_dir = find_sdk_dir();

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
            let mut builder = bindgen::Builder::default()
                .header("wrapper.h")
                .clang_arg(format!("-I{}", inc_path));

            // On Windows, clang may not find MSVC/UCRT system headers automatically.
            // Add the system include paths if available.
            if cfg!(target_os = "windows") {
                for inc in find_windows_sdk_includes() {
                    builder = builder.clang_arg(format!("-isystem{}", inc.display()));
                }
                if let Some(msvc_inc) = find_msvc_include() {
                    builder = builder.clang_arg(format!("-isystem{}", msvc_inc.display()));
                }
            }

            // With dynamic loading, all types/constants/functions are defined
            // manually in lib.rs. Bindgen is used only to validate that our
            // wrapper.h parses correctly against the installed SDK headers.
            // We blocklist everything to avoid duplicate definitions.
            builder
                .blocklist_function(".*")
                .blocklist_type(".*")
                .blocklist_var(".*")
                .blocklist_item(".*")
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

    // Write an empty bindings file — the manual declarations in lib.rs provide all needed symbols.
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

/// Find the Windows SDK include directories (ucrt, um, shared).
/// Returns paths under `C:\Program Files (x86)\Windows Kits\10\Include\<version>\`.
#[cfg(target_os = "windows")]
fn find_windows_sdk_includes() -> Vec<PathBuf> {
    let kits_root = PathBuf::from(r"C:\Program Files (x86)\Windows Kits\10\Include");
    if !kits_root.is_dir() {
        return Vec::new();
    }
    // Pick the newest SDK version that contains a ucrt directory with stdlib.h.
    let mut versions: Vec<_> = std::fs::read_dir(&kits_root)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().join("ucrt").join("stdlib.h").exists())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect();
    versions.sort();
    match versions.last() {
        Some(v) => {
            let base = kits_root.join(v);
            // ucrt: C runtime (stdlib.h), um: Windows API (windows.h), shared: common headers
            ["ucrt", "um", "shared"]
                .iter()
                .map(|sub| base.join(sub))
                .filter(|p| p.is_dir())
                .collect()
        }
        None => Vec::new(),
    }
}

/// Find the MSVC include directory (contains vcruntime.h etc.).
/// Searches VS 2022 and 2019 install locations with both Community and BuildTools editions.
#[cfg(target_os = "windows")]
fn find_msvc_include() -> Option<PathBuf> {
    let base_dirs = [
        r"C:\Program Files\Microsoft Visual Studio",
        r"C:\Program Files (x86)\Microsoft Visual Studio",
    ];
    let years = ["2022", "2019"];
    let editions = ["Community", "Professional", "Enterprise", "BuildTools"];

    for base in &base_dirs {
        for year in &years {
            for edition in &editions {
                let tools_dir =
                    PathBuf::from(base).join(year).join(edition).join("VC").join("Tools").join("MSVC");
                if !tools_dir.is_dir() {
                    continue;
                }
                // Pick the newest MSVC toolset version.
                let mut versions: Vec<_> = std::fs::read_dir(&tools_dir)
                    .ok()
                    .into_iter()
                    .flatten()
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().join("include").join("vcruntime.h").exists())
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect();
                versions.sort();
                if let Some(v) = versions.last() {
                    return Some(tools_dir.join(v).join("include"));
                }
            }
        }
    }
    None
}

#[cfg(not(target_os = "windows"))]
fn find_windows_sdk_includes() -> Vec<PathBuf> {
    Vec::new()
}

#[cfg(not(target_os = "windows"))]
fn find_msvc_include() -> Option<PathBuf> {
    None
}
