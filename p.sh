#!/bin/sh

set -e

mkdir -p build

cd payload

echo -------------------- COMPILING PAYLOAD --------------------

# clang -fPIE -nostdlib -ffreestanding -fshort-wchar --target=i686-pc-linux-gnu -fuse-ld=lld -mno-sse -Oz -DCOMPILING -o ../build/payload "-Wl,-Tlink.x" *.c
clang -fPIE -nostdlib -ffreestanding -fshort-wchar -fno-rtti -fno-exceptions --target=i686-pc-linux-msvc -std=c++17 -fuse-ld=lld -mno-sse -Oz -DCOMPILING -o ../build/payload "-Wl,-Tlink.x" *.cpp

echo -------------------- COPYING PAYLOAD --------------------
objcopy -O binary ../build/payload ../build/payload.bin

cd ..
