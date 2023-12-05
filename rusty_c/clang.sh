#!/bin/bash

# clang for cargo bindgen
# clang from ubuntu14 package manager is outdated for cargo bindgen (3.x < 5.0)
# sudo apt-get install llvm-dev libclang-dev clang
# clang 7.0 and clang-extra 
# (see https://stackoverflow.com/questions/46414475/how-to-install-clang-5-after-downloading-tar-xz-file-from-llvm-org)
wget https://releases.llvm.org/7.0.0/clang+llvm-7.0.0-x86_64-linux-gnu-ubuntu-14.04.tar.xz
tar -xvf clang+llvm-7.0.0-x86_64-linux-gnu-ubuntu-14.04.tar.xz
rm -rf clang+llvm-7.0.0-x86_64-linux-gnu-ubuntu-14.04.tar.xz
export PATH=$(pwd)/clang+llvm-7.0.0-x86_64-linux-gnu-ubuntu-14.04/bin:$PATH
export LD_LIBRARY_PATH="$(pwd)/clang+llvm-7.0.0-x86_64-linux-gnu-ubuntu-14.04/lib:$LD_LIBRARY_PATH"