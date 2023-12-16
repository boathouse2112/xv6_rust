#!/bin/sh

cargo build --all --release
x86_64-elf-objcopy -S -O binary -j .text target/i686-unknown-none/release/bootloader btldr
dd if=/dev/zero of=boot.img count=10000
dd if=btldr of=boot.img conv=notrunc

# Add 0x55AA boot signature
printf "%b" '\x55\xAA' > signature
dd if=signature of=boot.img bs=510 seek=1 conv=notrunc
rm signature

rm btldr
