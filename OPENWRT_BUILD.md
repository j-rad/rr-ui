# Building for OpenWrt

Since OpenWrt uses the `musl` C library and often runs on different architectures (ARM, MIPS, x86_64), cross-compilation is required.

## Method 1: Using `cross` (Recommended)

The easiest way to build for OpenWrt is using [cross](https://github.com/cross-rs/cross), which uses Docker to provide the cross-compilation environment.

1. **Install `cross`**:

    ```bash
    cargo install cross
    ```

2. **Determine your target architecture**:
    * **ARM64** (Raspberry Pi 4, NanoPi R4S, etc.): `aarch64-unknown-linux-musl`
    * **x86_64** (x86 Routers): `x86_64-unknown-linux-musl`
    * **ARMv7** (older ARM): `armv7-unknown-linux-musleabihf`
    * **MIPS** (common routers): `mips-unknown-linux-musl` (Note: MIPS support might be limited by dependencies)

3. **Build**:

    ```bash
    # For ARM64
    cross build --release --features server --target aarch64-unknown-linux-musl

    # For x86_64
    cross build --release --features server --target x86_64-unknown-linux-musl
    ```

    The binary will be in `target/<TARGET>/release/rr-ui`.

## Method 2: Manual Cross-Compilation

If you have the cross-compilation toolchain installed (e.g., via OpenWrt SDK):

1. **Add the target**:

    ```bash
    rustup target add aarch64-unknown-linux-musl
    ```

2. **Configure Linker**:
    Add the following to `.cargo/config.toml`:

    ```toml
    [target.aarch64-unknown-linux-musl]
    linker = "aarch64-openwrt-linux-musl-gcc"
    ```

3. **Build**:

    ```bash
    cargo build --release --features server --target aarch64-unknown-linux-musl
    ```

## Post-Build Steps

1. **Copy to Device**:

    ```bash
    scp target/aarch64-unknown-linux-musl/release/rr-ui root@192.168.1.1:/usr/bin/
    ```

2. **Run Installation Script**:
    Copy `install.sh` to the device and run it to set up the service.
