# wget https://github.com/riscv/riscv-openocd/archive/refs/tags/v2018.12.0.tar.gz
# tar -xvf v2018.12.0.tar.gz
# rm -rf v2018.12.0.tar.gz
# export PATH=$(pwd)/riscv-openocd-2018.12.0/:$PATH


# udev rules for openocd https://docs.rust-embedded.org/book/intro/install/linux.html#udev-rules
# openocd from sifive
echo "[!] Installing openocd and adding to path"
wget https://static.dev.sifive.com/dev-tools/riscv-openocd-0.10.0-2019.02.0-x86_64-linux-ubuntu14.tar.gz
tar -zxvf riscv-openocd-0.10.0-2019.02.0-x86_64-linux-ubuntu14.tar.gz
rm -rf riscv-openocd-0.10.0-2019.02.0-x86_64-linux-ubuntu14.tar.gz
export PATH=$PATH:$(pwd)/riscv-openocd-0.10.0-2019.02.0-x86_64-linux-ubuntu14/bin