# udev rules for openocd https://docs.rust-embedded.org/book/intro/install/linux.html#udev-rules
# openocd from sifive
echo "[!] Installing openocd and adding to path"
wget https://static.dev.sifive.com/dev-tools/riscv-openocd-0.10.0-2019.02.0-x86_64-linux-ubuntu14.tar.gz
tar -zxvf riscv-openocd-0.10.0-2019.02.0-x86_64-linux-ubuntu14.tar.gz
rm -rf riscv-openocd-0.10.0-2019.02.0-x86_64-linux-ubuntu14.tar.gz
export PATH=$PATH:$(pwd)/riscv-openocd-0.10.0-2019.02.0-x86_64-linux-ubuntu14/bin

# clang for cargo bindgen
sudo apt-get install llvm-dev libclang-dev clang