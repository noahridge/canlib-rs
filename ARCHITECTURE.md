# canlib-rs Architecture

This document describes the internal architecture of `canlib-rs`, a Rust wrapper for the Kvaser CANLib SDK.

## Crate Structure

The project is organized as a Cargo workspace with two crates:

```
canlib-rs/
├── Cargo.toml              # Workspace root
├── canlib-sys/             # Layer 1: Raw FFI bindings
│   ├── build.rs            # Build script (SDK detection + bindgen)
│   ├── wrapper.h           # C header entry point
│   └── src/lib.rs          # FFI types, constants, extern "C" declarations
└── canlib/                 # Layer 2: Safe idiomatic Rust wrapper
    ├── src/
    │   ├── lib.rs          # Public API, initialization, channel enumeration
    │   ├── channel.rs      # Channel handle (open/close/read/write/bus control)
    │   ├── message.rs      # CAN message types and flags
    │   ├── error.rs        # Error enum and status code mapping
    │   ├── bus_params.rs   # Bitrate, timing, and driver configuration
    │   └── status.rs       # Bus status, error counters, statistics
    └── examples/           # Usage examples
```

## Layer 1: `canlib-sys` (FFI Bindings)

This crate follows the Rust `-sys` crate convention. Its job is to expose the raw C API to Rust code.

### Build Script (`build.rs`)

The build script handles two concerns:

1. **SDK Discovery** — Locates the Kvaser CANLib SDK on the host system by checking:
   - The `CANLIB_SDK_DIR` environment variable (highest priority)
   - Windows default paths: `C:\Program Files (x86)\Kvaser\Canlib`, `C:\Program Files\Kvaser\CANlib SDK`
   - Linux: `/usr/include/canlib.h` (from the linuxcan package)

2. **Binding Generation** — Attempts to run `bindgen` against `canlib.h` to auto-generate FFI declarations. If `libclang` is not available (bindgen panics), it catches the panic and falls back to an empty bindings file. The manual `extern "C"` blocks in `lib.rs` provide all required function declarations regardless.

### Linking

The build script emits Cargo link directives:
- **Windows**: `cargo:rustc-link-lib=canlib32` with search path `Lib/x64` (64-bit) or `Lib/MS` (32-bit)
- **Linux**: `cargo:rustc-link-lib=canlib`

### `src/lib.rs`

Contains three categories of declarations:

- **`include!` of bindgen output** — Auto-generated types and functions (empty if bindgen unavailable)
- **Manual constants** — All `canERR_*`, `canOPEN_*`, `canBITRATE_*`, `canMSG_*`, `canFDMSG_*`, `canDRIVER_*`, `canSTAT_*`, and `canCHANNELDATA_*` constants
- **Manual `extern "C"` blocks** — Complete function declarations for all wrapped CANLib functions, ensuring the crate works even without bindgen

This dual approach (bindgen + manual fallback) maximizes portability: the crate compiles whether or not LLVM/Clang is installed.

## Layer 2: `canlib` (Safe Wrapper)

This crate provides a safe, ergonomic Rust API. It depends on `canlib-sys`, `bitflags`, and `thiserror`.

### Module Dependency Graph

```
lib.rs
├── error.rs        (no internal deps)
├── message.rs      (no internal deps)
├── bus_params.rs   (no internal deps)
├── status.rs       (no internal deps)
└── channel.rs      (depends on all of the above)
```

`lib.rs` re-exports the primary types from all modules at the crate root for convenience.

### Error Handling (`error.rs`)

All CANLib functions return `canStatus` (a signed integer). The error module provides:

- `CanError` — An enum with a variant for each known `canERR_*` code, plus `Unknown(i32)` as a catch-all. Uses `thiserror` for `Display` and `Error` trait implementations.
- `Result<T>` — Type alias for `std::result::Result<T, CanError>`.
- `check_status()` — Converts a raw `canStatus` into `Result<()>` (non-negative = success).
- `check_handle()` — Converts a raw return value into `Result<canHandle>` (used by `canOpenChannel` which returns a handle on success, negative on error).

Every unsafe FFI call in the wrapper is checked through one of these functions.

### Message Types (`message.rs`)

- `CanMessage` — Represents a CAN frame (classic or FD). Fields: `id`, `data` (Vec<u8>), `dlc`, `flags`, `timestamp`. Constructors: `new()` (standard 11-bit), `new_extended()` (29-bit), `new_fd()` (CAN FD with optional BRS).
- `MessageFlags` — A `bitflags` type mapping CANLib's message flag constants (RTR, STD, EXT, FD, BRS, ESI, etc.).

### Bus Parameters (`bus_params.rs`)

- `Bitrate` — Enum of predefined classic CAN bitrates (10K through 1M).
- `FdBitrate` — Enum of predefined CAN FD data-phase bitrates.
- `BusParams` — Custom timing struct (freq, tseg1, tseg2, sjw, noSamp, syncMode).
- `BusParamsTq` — Time-quanta timing struct, with `to_raw()`/`from_raw()` conversions to the FFI `kvBusParamsTq`.
- `DriverType` — Transceiver mode enum (Normal, Silent, SelfReception, Off).

### Channel (`channel.rs`)

`Channel` is the central type. It wraps a `canHandle` and manages its lifecycle.

**Ownership and RAII:**
- `Channel::open()` calls `canOpenChannel` and returns a `Channel`.
- `Drop` implementation calls `canBusOff` (if on-bus) then `canClose`, ensuring resources are always released.

**Thread Safety:**
- `Channel` is `Send` but not `Sync`. CANLib handles are per-thread; each thread that needs CAN access must open its own channel. The `Send` impl allows moving a channel to a different thread (e.g., spawning a reader thread).

**Method Categories:**
| Category | Methods |
|---|---|
| Bus control | `bus_on`, `bus_off`, `reset_bus`, `is_on_bus` |
| Bus params | `set_bitrate`, `set_bus_params`, `set_bus_params_tq`, `set_bus_params_fd`, `set_bus_params_fd_tq`, `get_bus_params`, `get_bus_params_tq` |
| Driver mode | `set_output_control`, `get_output_control` |
| Transmit | `write`, `write_wait`, `write_sync` |
| Receive | `read`, `read_wait`, `read_sync`, `read_specific`, `read_specific_skip` |
| Filters | `set_acceptance_filter` |
| Status | `read_status`, `read_error_counters`, `request_chip_status`, `request_bus_statistics`, `get_bus_statistics` |
| Queue | `flush_rx`, `flush_tx` |
| Escape hatch | `raw_handle` |

**Buffer Management:**
All read/write methods use stack-allocated `[u8; 64]` buffers internally. Data is copied between these fixed buffers and the `Vec<u8>` in `CanMessage`, keeping the unsafe FFI boundary contained.

### Status (`status.rs`)

- `BusStatus` — `bitflags` type for bus state (ERROR_PASSIVE, BUS_OFF, ERROR_WARNING, ERROR_ACTIVE, TX_PENDING, RX_PENDING, OVERRUN).
- `ErrorCounters` — Struct with tx/rx/overrun error counts.
- `BusStatistics` — Struct with frame counts, bus load, and overruns. Includes `bus_load_percent()` helper.

### Library Initialization (`lib.rs`)

`canInitializeLibrary()` must be called once before any other CANLib function. This is handled by `ensure_initialized()` using `std::sync::Once`, and is called automatically by `Channel::open()` and the enumeration functions. Users never need to call it manually.

## Safety Strategy

| Concern | Approach |
|---|---|
| Null/dangling pointers | All raw pointer operations are confined to the sys crate and the `Channel` methods. The safe API uses `Vec<u8>` and stack buffers. |
| Use-after-free | `Channel` owns the handle; `Drop` closes it. No way to use a closed handle through the safe API. |
| Thread safety | `Channel: Send + !Sync` matches CANLib's threading model. `Once` guards initialization. |
| Error propagation | Every FFI call is checked via `check_status`/`check_handle`. No silent failures. |
| Resource cleanup | RAII via `Drop`. Bus is taken off before handle is closed. |
| Buffer overflows | Read/write buffers are `CANFD_MAX_DLC` (64 bytes), matching the maximum CAN FD payload. DLC is bounds-checked. |

## Dependencies

| Crate | Role | Layer |
|---|---|---|
| `bindgen` | Build-time C header parsing | canlib-sys (build-dep) |
| `bitflags` | Type-safe flag sets for MessageFlags, OpenFlags, BusStatus | canlib |
| `thiserror` | Derive `Error` + `Display` for CanError | canlib |
