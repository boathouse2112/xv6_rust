#!/bin/sh

cargo build --all --release
x86_64-elf-objcopy -S -O binary -j .text target/i686-unknown-none/release/bootloader btldr
dd if=/dev/zero of=boot.img bs=1024 count=1440
dd if=btldr of=boot.img conv=notrunc
rm btldr
