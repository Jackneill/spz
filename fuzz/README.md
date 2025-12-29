# SPZ Fuzz Testing

This directory contains fuzz tests for the SPZ library.

## Prerequisites

Install cargo-fuzz:

```bash
cargo install cargo-fuzz
```

You'll also need a nightly Rust toolchain:

```bash
rustup install nightly
```

## Running Fuzz Tests

### Build All Targets

Due to a known issue with AddressSanitizer and clang 20, you may need to disable the sanitizer:

```bash
cargo +nightly fuzz build --sanitizer none
```

If your system supports ASAN properly:

```bash
cargo +nightly fuzz build
```

### Run a Specific Target

```bash
# Without sanitizer (works on systems with clang 20+)
cargo +nightly fuzz run <fuzz_target> --sanitizer none

# With ASAN (recommended if it works on your system)
cargo +nightly fuzz run <fuzz_target>
```

### Run with Options

```bash
# Limit input size and run time
cargo +nightly fuzz run <fuzz_target> --sanitizer none -- -max_len=10000 -max_total_time=60

# Run for a specific number of iterations
cargo +nightly fuzz run <fuzz_target> --sanitizer none -- -runs=100000
```

### List Available Targets

```bash
cargo fuzz list
```

## Corpus

The fuzzer maintains a corpus of interesting inputs in `corpus/<fuzz_target>/`. These inputs are reused across runs to improve coverage.

## Crashes

If a crash is found, it will be saved to `artifacts/<fuzz_target>/`. To reproduce:

```bash
cargo +nightly fuzz run <fuzz_target> artifacts/<fuzz_target>/crash-*
```

## Continuous Fuzzing

For CI or extended fuzzing sessions:

```bash
# Run each target for 5 minutes
for target in $(cargo +nightly fuzz list); do
    cargo +nightly fuzz run "$target" --sanitizer none -- -max_total_time=300
done
```
