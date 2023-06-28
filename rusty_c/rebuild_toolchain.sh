#!/bin/bash

echo "fedora36?: $(cat /etc/fedora-release)"

# sifive riscv toolchain
git clone https://github.com/sifive/freedom-tools
cd freedom-tools
git fetch --all --tags
git checkout tags/v2020.04.0-Toolchain.Only -b main
git submodule update --init --recursive

sudo yum install cmake libmpc-devel mpfr-devel gmp-devel gawk bison flex texinfo patchutils gcc gcc-c++ zlib-devel expat-devel
sudo yum install autoconf automake libtool pkg-config

make
# todo only a subset of the full toolchain needs to be built, only gcc and binutils (objdump and objcopy) 
# todo export PATH=$PATH:$PWD/riscv64-unknown-elf-gcc-8.3.0-2020.04.1-x86_64-linux-fedora36/bin

# yunhao qemu