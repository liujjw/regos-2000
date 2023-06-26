#!/bin/bash

set -e

# > 1.59.0 for risc-v
VER=1.70.0

# install rustup
if ! [ -x "$(command -v rustup)" ]; then
    echo "[!] Installing rustup"

    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

    export PATH=$HOME/.cargo/bin:$PATH
fi
rustup default $VER
rustup component add clippy

# install precompiled core crate
# tier 2 target, with single and double precision floating point
rustup target add riscv32i-unknown-none-elf

# install binutils
rustup component add llvm-tools-preview 
cargo install cargo-binutils

# cargo template 
echo "[!] Installing cargo-generate"
echo "[!] libssl-dev and pkg-config may be required on ubuntu"
cargo install cargo-generate

# risc-v toolchain (gdb), qemu, etc. 
echo "[!] Please ensure appropriate risc-v toolchain used by egos-2000 is installed already"
echo "[!] Assuming https://github.com/sifive/freedom-tools/releases/tag/v2020.04.0-Toolchain.Only installed"
echo "[!] Please ensure qemu-system-riscv used by egos-2000 is installed already"
echo "[!] Assuming https://github.com/yhzhang0128/freedom-tools/releases/tag/v2023.5.1 installed"
echo "[!] Restart terminal if Rust not found"
echo "[!] Run []_setup_2.sh for your machine"

source "$HOME/.cargo/env"