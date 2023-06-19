#!/bin/bash

# c2rust
sudo apt-get update
sudo apt-get install curl
sudo apt install build-essential llvm clang libclang-dev cmake libssl-dev pkg-config python3 git

# qemu TODO build for ubuntu18
wget https://github.com/yhzhang0128/freedom-tools/releases/download/v2023.5.1/riscv-qemu-5.2.0-2020.12.0-preview1-x86_64-linux-ubuntu14.tar.gz
tar -zxvf riscv-qemu-5.2.0-2020.12.0-preview1-x86_64-linux-ubuntu14.tar.gz
rm -rf riscv-qemu-5.2.0-2020.12.0-preview1-x86_64-linux-ubuntu14.tar.gz
export PATH=$(pwd)/riscv-qemu-5.2.0-2020.12.0-preview1-x86_64-linux-ubuntu14/bin:$PATH