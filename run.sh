#!/bin/sh
qemu-system-x86_64 -drive format=raw,file=target/x86_64-unknown-none/debug/bootimage-xv6_x86_64_rs.bin
