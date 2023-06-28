#!/bin/bash
export PATH=$(pwd)/riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14/bin:$PATH
export PATH=$(pwd)/riscv-qemu-5.2.0-2020.12.0-preview1-x86_64-linux-ubuntu14/bin:$PATH
export PATH=$(pwd)/clang+llvm-7.0.0-x86_64-linux-gnu-ubuntu-14.04/bin:$PATH
export LD_LIBRARY_PATH="$(pwd)/clang+llvm-7.0.0-x86_64-linux-gnu-ubuntu-14.04/lib:$LD_LIBRARY_PATH:$PATH"