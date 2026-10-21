# Incremental Rust acceleration

This first Rust component replaces only PIV v5 frame validation during import.
The JavaScript interface, WebGPU renderer, editable document, editor commands,
history and MCP API retain their existing ownership. Browser builds continue
using their existing codec; this component targets the desktop Python service.

## Build and enable

Install Rust/Cargo 1.85+ and your host linker. Windows needs Visual Studio C++
Build Tools and the Windows SDK; use a Developer PowerShell if Cargo cannot find
`link.exe`. Linux needs its normal C linker/toolchain. macOS requires Xcode
Command Line Tools; macOS support has not been verified locally.

```sh
npm run build:frame-scan:rust
npm start
```

The crate has no third-party dependencies. The build uses Cargo's locked,
offline release mode and writes a host-specific library and source/binary hashes
to `native/bin/rust-<platform>-<arch>`. Generated libraries and Cargo targets are
ignored by Git; existing tracked C++ binaries are never overwritten. On Windows
the helper statically links the CRT, so it adds no Visual C++ runtime installer
requirement. Rust and Cargo are needed only to build, not to run the app.
The build retains the toolchain's standard-library license and copyright notices;
the frozen codec bundles them under `THIRD_PARTY/rust`.

On startup, `backend/native_scan.py` prefers a compatible Rust library, then
the existing C++ library, then the Python implementation. `/api/health` reports
the actual implementation as `rust`, `cpp` or `python`. Restart the app after
building or replacing a helper. A DLL already loaded by a running Windows app
cannot be replaced; close that app before rebuilding.

- `STYK_DISABLE_NATIVE_SCAN=1`: force Python.
- `STYK_NATIVE_SCAN_LIBRARY=/absolute/path/to/library`: explicitly select one
  native library; if it cannot load, use Python rather than a different binary.
- `CARGO=/absolute/path/to/cargo`: select the build tool.

For a packaged desktop build, run `npm run build:frame-scan:rust` before
`npm run build:sidecar`. The sidecar builder prefers the Rust build manifest and
checks its source, Cargo files, platform, architecture, ABI and binary hash.
Packaging checks the same source identity and frozen-library hash. Without a
Rust build, the existing C++ release path remains supported. Release workflows
continue using C++ unless explicitly changed to build Rust first.

## Boundary and correctness

`native/rust-frame-scan` implements ABI 1, shared with the established C++ helper.
The bridge lends immutable payload and definition bytes to Rust for one call.
Rust writes only frame durations and a fixed-size status record into bounded,
caller-owned output. There is no JSON round trip, whole-document clone or
Rust-owned allocation returned across the boundary.

The parser uses checked slices. Raw pointer conversion is confined to the C ABI
entry point and follows its documented buffer contract. Scratch vectors are
private to each call, reused between frames, and use fallible capacity growth.
UID uniqueness is checked with an allocation-free sort of a reusable contiguous
buffer, replacing C++'s per-object hash-node allocations. This implementation
choice contributes to the measured gain; Rust alone does not guarantee speed.

The helper preserves frame timing, signed UID bit patterns, type changes,
optional pose fields, media flags, background references, draw order, camera
finite checks, join handles/cycles and the existing estimated pose budget.
Duplicate identities still invoke the portable validator's established recovery
policy. It does not alter the saved file grammar or editing semantics.

The wrapper checks cancellation before the native call and before publishing
its result. A native call finishes before the second checkpoint; it is not
interruptible midway. Cancelled output does not advance the reader or publish
frame durations. Failures never replace the active editor document.

## Measured component results

Measured on Windows 10 x64, AMD Ryzen 5 5600G, Python 3.14.6 and Rust 1.85.1.
The production Rust DLL uses a static CRT. Each backend runs in a fresh process
on identical input bytes; values are medians of six warm runs after one warm-up.
Timings include metadata packing, ctypes and duration-shell materialization.
Every backend must produce the same result hash and consumed-byte position.

| Synthetic workload | Python | Existing C++ | Rust |
| --- | ---: | ---: | ---: |
| 60 frames × 20 figures × 10 joints | 3.280 ms | 0.109 ms | 0.070 ms |
| 120 frames × 1,000 figures × 32 joints | 373.853 ms | 8.351 ms | 4.143 ms |
| 60 frames × 1,500 joined figures × 24 joints | 313.236 ms | 6.698 ms | 3.761 ms |
| 120 frames × 1,000 figures × 32 joints, scrambled UIDs | 368.764 ms | 8.746 ms | 4.747 ms |

Rust took about 44–50% less time than the existing C++ helper on the three large
fixtures. Process peak working set was similar; no memory reduction is claimed.
Python profiling identifies the portable scanner's repeated field reads and
finite checks as its dominant work. These synthetic results exclude file
decompression, asset decoding, transport, rendering and interaction latency.
They establish neither a whole-app speedup nor higher playback FPS. The private
Pivot corpus was unavailable, so its tests remain skipped.

The actual desktop import check also compares the Rust and Python paths on a
valid PIV containing 24 frames and 40 figures. Frame count, FPS, figure poses and
all evaluated joint coordinates match exactly. This is a correctness check,
not an end-to-end performance claim.

A separate single-pair Electron diagnostic on that small fixture measured
backend decode at 3.415 ms with Python versus 1.153 ms with Rust. Total import
was 189.3 ms versus 213.2 ms, respectively. One noisy pair cannot establish a
whole-app gain or regression; it illustrates why a faster validation component
must not be advertised as a faster complete editor. The frozen Rust service
also passed native-wire import and byte-exact PIV export on the same fixture.

## Reproduce verification

```sh
npm run test:rust
cargo fmt --manifest-path native/rust-frame-scan/Cargo.toml --check
cargo clippy --locked --manifest-path native/rust-frame-scan/Cargo.toml -- -D warnings
python -m unittest discover -s backend -p "test_native_scan*.py" -v
python -m unittest discover -s backend -p "test_*.py"
python scripts/benchmark-rust-scan.py --rust-library /absolute/path/to/built/library
node scripts/smoke-rust-import.mjs /absolute/path/to/project.piv
```

The benchmark writes raw timings, result hashes, memory snapshots and Python
profiles under `artifacts/rust-scan`. Differential tests cover 1,500 deterministic
mutations, every truncation of a mixed fixture, media/optional arrays, concurrent
calls, long joins, malformed metadata, ABI mismatch, fallback and cancellation.
The Rust CI workflow builds and runs the differential checks on Windows and
Linux; local results reported here cover Windows x64 only.
