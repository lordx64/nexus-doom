# DOOM on Nexus zkVM

This project aims to run the classic game DOOM on the Nexus Zero-Knowledge Virtual Machine (zkVM). It's an ongoing work that demonstrates the capabilities of zkVM technology in running complex applications.

## Project Structure

- `methods/guest`: Contains the main DOOM implementation for the zkVM.
- `host`: Handles host-side operations and proof generation.
- `puredoom-rs`: Rust bindings for PureDOOM, a portable DOOM implementation.
- `zkdoom-common`: Shared structures and types used across the project.

## Key Features

1. Cross-compilation for RISC-V 32-bit architecture
2. Custom Rust toolchain configuration
3. Integration with Nexus zkVM
4. Frame capturing and output
5. Demo playback support

## Building and Running

1. Ensure you have the Rust toolchain installed as specified in `rust-toolchain.toml`.

2. Build the project:
   ```
   cargo build --release
   ```

3. Run DOOM with a demo file:
   ```
   cargo run --release -- -e path/to/elf_file -d path/to/demo_file.lmp
   ```