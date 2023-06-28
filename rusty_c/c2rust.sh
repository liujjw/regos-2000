#!/bin/bash

# c2rust
sudo apt-get update
sudo apt-get install curl
sudo apt install build-essential llvm clang libclang-dev cmake libssl-dev pkg-config python3 git python3-pip

sudo rustup component add rustfmt-preview
# sudo find / -name "llvm-config-*" -print 2>/dev/null
# export LLVM_LIB_DIR=/usr/bin/llvm-config-6.0
sudo cargo install c2rust
pip3 install scan-build