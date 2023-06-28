#!/bin/bash

wget https://static.dev.sifive.com/dev-tools/riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14.tar.gz
tar -zxvf riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14.tar.gz
rm -rf riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14.tar.gz
export PATH=$(pwd)/riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-ubuntu14/bin:$PATH

wget https://github.com/yhzhang0128/freedom-tools/releases/download/v2023.5.1/riscv-qemu-5.2.0-2020.12.0-preview1-x86_64-linux-ubuntu14.tar.gz
tar -zxvf riscv-qemu-5.2.0-2020.12.0-preview1-x86_64-linux-ubuntu14.tar.gz
rm -rf riscv-qemu-5.2.0-2020.12.0-preview1-x86_64-linux-ubuntu14.tar.gz
export PATH=$(pwd)/riscv-qemu-5.2.0-2020.12.0-preview1-x86_64-linux-ubuntu14/bin:$PATH

# for cargo-generate
sudo apt install -y libssl-dev pkg-config

# gcc version 4.8.4 (Ubuntu 4.8.4-2ubuntu1~14.04.4) needs std=c99 for compiling

# temp glibc version for qemu