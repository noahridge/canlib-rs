# Plan: Optional Bindgen for SDK Header Validation

## Problem

With runtime dynamic loading, all FFI types, constants, and function signatures
are defined manually in `canlib-sys/src/lib.rs`. If Kvaser updates their SDK and
changes a function signature, struct layout, or constant value, the manual
declarations silently become wrong — leading to memory corruption, undefined
behavior, or subtle runtime bugs with no compile-time warning.

## Goal

Re-introduce bindgen as an **optional CI-time validation tool** that checks our
manual declarations against the real SDK headers, without requiring it for
normal builds.

## Implementation Steps

### 1. Add `bindgen` as an optional build dependency

In `canlib-sys/Cargo.toml`:

```toml
[features]
default = []
validate-bindings = ["bindgen"]

[build-dependencies]
bindgen = { version = "0.71", optional = true }
```

### 2. Restore `wrapper.h`

```c
#include <canlib.h>
```

### 3. Create a `build.rs` that only runs under the feature flag

When `validate-bindings` is enabled:
- Use bindgen to generate bindings from the installed SDK headers
- Write them to `OUT_DIR/generated_bindings.rs`

When disabled:
- Do nothing (no build script needed, or write an empty file)

### 4. Add a compile-time validation test

Create `canlib-sys/tests/validate_bindings.rs` (only compiled with the feature):

```rust
#[cfg(feature = "validate-bindings")]
mod validate {
    // Include the generated bindings
    mod generated {
        #![allow(non_upper_case_globals, non_camel_case_types, non_snake_case, dead_code)]
        include!(concat!(env!("OUT_DIR"), "/generated_bindings.rs"));
    }

    // Assert that our manual constants match the generated ones
    #[test]
    fn constants_match() {
        assert_eq!(generated::canOK, canlib_sys::canOK);
        assert_eq!(generated::canERR_PARAM, canlib_sys::canERR_PARAM);
        // ... all constants
    }

    // Assert struct sizes/alignment match
    #[test]
    fn struct_layouts_match() {
        assert_eq!(
            std::mem::size_of::<generated::kvBusParamsTq>(),
            std::mem::size_of::<canlib_sys::kvBusParamsTq>(),
        );
        assert_eq!(
            std::mem::size_of::<generated::canBusStatistics>(),
            std::mem::size_of::<canlib_sys::canBusStatistics>(),
        );
    }
}
```

### 5. Add CI job

Add a CI step that runs on a machine with the Kvaser SDK and LLVM installed:

```yaml
- name: Validate FFI bindings against SDK headers
  run: cargo test --features validate-bindings -p canlib-sys
```

This catches SDK drift automatically without affecting normal builds.

## Trade-offs

- **Normal builds**: No bindgen, no LLVM, no SDK required — compiles everywhere
- **CI validation**: Requires SDK + LLVM on one CI runner to verify correctness
- **Developer workflow**: Run `cargo test --features validate-bindings` locally
  after updating manual declarations to verify against installed SDK

## Dependencies

- Kvaser CANLib SDK installed (provides headers)
- LLVM/clang installed (required by bindgen)
- Windows SDK headers available (for `stdlib.h`, `windows.h` — needed by `canlib.h`)
