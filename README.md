# Benchmarks

RNG creation only (10 rerolls)
Local - ~2.2 billion per second CPU, ~1.6 billion per second GPU

CPU benchmarks:
new+get_node[x3] - 690 ns (1.44 Melem/s)
new_only - 72.7 ns (13.7 Melem/s)
get_node[x3] - 102 ns (9.78 Melem/s)

# Running

## Cloud

Running in the cloud is usually the best option. I'd suggest Google Cloud, using a V100 with Spot pricing, so it only costs about 1 buck an hour.

Simply select the Debian with CUDA 11.8 image for your OS in the disk tab when creating the instance.

Run:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
git clone https://github.com/BigBadE/balatro-seed-checker
cd balatro-seed-checker
sudo apt-get update
# Build LLVM 7.1.0
apt-get update && apt-get -qq -y install \
    build-essential \
    curl \
    clang \
    libssl-dev \
    libtinfo-dev \
    pkg-config \
    xz-utils \
    libffi-dev \
    libedit-dev \
    libncurses5-dev \
    libxml2-dev \
    python3 \
    ninja-build \
    nvidia-cuda-toolkit \
    zlib1g-dev
curl -sSf -L -O https://github.com/llvm/llvm-project/releases/download/llvmorg-7.1.0/llvm-7.1.0.src.tar.xz && \
    tar -xf llvm-7.1.0.src.tar.xz && \
    cd llvm-7.1.0.src && \
    mkdir build && cd build && \
    ARCH=$(dpkg --print-architecture) && \
    if [ "$ARCH" = "amd64" ]; then \
        TARGETS="X86;NVPTX"; \
    else \
        TARGETS="AArch64;NVPTX"; \
    fi && \
    cmake -G Ninja \
        -DCMAKE_BUILD_TYPE=Release \
        -DLLVM_TARGETS_TO_BUILD="$TARGETS" \
        -DLLVM_BUILD_LLVM_DYLIB=ON \
        -DLLVM_LINK_LLVM_DYLIB=ON \
        -DLLVM_ENABLE_ASSERTIONS=OFF \
        -DLLVM_ENABLE_BINDINGS=OFF \
        -DLLVM_INCLUDE_EXAMPLES=OFF \
        -DLLVM_INCLUDE_TESTS=OFF \
        -DLLVM_INCLUDE_BENCHMARKS=OFF \
        -DLLVM_ENABLE_ZLIB=ON \
        -DLLVM_ENABLE_TERMINFO=ON \
        -DCMAKE_INSTALL_PREFIX=/usr \
        .. && \
    ninja -j$(nproc) && \
    ninja install && \
    cd ../.. && \
    rm -rf llvm-7.1.0.src* && \
    ln -s /usr/bin/llvm-config /usr/bin/llvm-config-7
echo 'export LD_LIBRARY_PATH=/usr/local/cuda/nvvm/lib64:$LD_LIBRARY_PATH' >> ~/.bashrc
source ~/.bashrc
. "$HOME/.cargo/env"
cargo run --release
```