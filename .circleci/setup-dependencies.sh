#!/bin/bash

set -x

echo "Setting up Rust"
curl https://sh.rustup.rs -sSf | sh
export PATH=$PATH:~/.cargo/bin/

export LLVM_VERSION=5.0.2
echo "Setting up LLVM ${LLVM_VERSION}"
export LLVM=clang+llvm-${LLVM_VERSION}-x86_64-linux-gnu-ubuntu-16.04
wget http://llvm.org/releases/${LLVM_VERSION}/${LLVM}.tar.xz
mkdir llvm
tar -xf ${LLVM}.tar.xz -C llvm --strip-components=1
export LIBCLANG_PATH=`pwd`/llvm/lib/

echo "Setting up ISPC"
wget -O ispc.tar.gz https://downloads.sourceforge.net/project/ispcmirror/v1.10.0/ispc-v1.10.0-${TRAVIS_OS_NAME}.tar.gz
tar -xvf ispc.tar.gz
export PATH=$PATH:`pwd`/ispc-1.10.0-Linux/bin/:`pwd`/ispc-1.10.0-Darwin/bin/
ispc --version

