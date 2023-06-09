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
# yunhao qemu, and openocd