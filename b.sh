#!/bin/sh

set -e

./p.sh

cd payload

echo -------------------- ENCODING PAYLOAD --------------------
read -ra arr <<< "$(python ../payloadb64.py)"
echo -------------------- ASSEMBLING BOOTLOADER PRE --------------------
nasm -fbin -o ../build/bootloader_pre.bin \
	-DPAYLOAD_B64_LEN=${arr[0]} -DPAYLOAD_B64_BYTES=${arr[1]} \
	-DPRE \
	bootloader.asm
echo -------------------- ASSEMBLING BOOTLOADER POST GMH --------------------
nasm -fbin -o ../build/bootloader_post_gmh.bin \
	-DPAYLOAD_B64_LEN=${arr[0]} -DPAYLOAD_B64_BYTES=${arr[1]} \
	-DPOST_GMH \
	bootloader.asm
echo -------------------- ASSEMBLING BOOTLOADER POST B64 --------------------
nasm -fbin -o ../build/bootloader_post_b64.bin \
	-DPAYLOAD_B64_LEN=${arr[0]} -DPAYLOAD_B64_BYTES=${arr[1]} \
	-DPOST_B64 \
	bootloader.asm
echo -------------------- CALCULATING BOOTLOADER VARS --------------------
read -ra arr2 <<< "$(python ../solver.py)"
echo -------------------- ASSEMBLING BOOTLOADER --------------------
nasm -fbin -o ../build/bootloader.bin \
	-DPAYLOAD_B64_LEN=${arr[0]} -DPAYLOAD_B64_BYTES=${arr[1]} \
	-DGMH_B4=${arr2[0]} -DGMH_8=${arr2[1]} -DGMH_1=${arr2[2]} \
	-DB64_B4=${arr2[3]} -DB64_8=${arr2[4]} -DB64_1=${arr2[5]} -DNOP=${arr2[6]} \
	bootloader.asm

cd ..

echo -------------------- CLEANING SAVEFILE --------------------
python clean.py

echo -------------------- GENERATING SPWN --------------------
cargo run

echo -------------------- RUNNING SPWN --------------------
spwn b build/main.spwn

echo -------------------- PATCHING LEVEL --------------------
python patch.py
