# BOS (Basic Operating System)

Basic Operating System in Rust, 

I just wanted to pronounce bos instead of dos so, use b for whatever you want, like beta, baegle, ... lol

## Prerequisites

1. **Install Rust nightly and required components:**
	```sh
	rustup default nightly
	rustup component add llvm-tools-preview rust-src
	cargo install bootimage
	```

2. **Add the target for bare metal x86_64:**
	```sh
	rustup target add x86_64-unknown-none
	```

## Building the Bootable Image

1. **Build the kernel and bootable image:**
	```sh
	cargo bootimage
	```
	This will create a bootable disk image at:
	```
	target/x86_64-blog_os/debug/bootimage-bos.bin
	```

## Running

You can run the image in [QEMU](https://www.qemu.org/) with:
```sh
& 'C:\Program Files\qemu\qemu-system-x86_64.exe' -drive format=raw,file=.\target\x86_64-bos\debug\bootimage-bos.bin
```

## Notes

- The project uses `bootloader = "0.9"` for maximum compatibility with minimal kernels.
- The build configuration is set up for a custom target and uses `build-std` to compile core libraries for bare metal.
- For more details, see [https://os.phil-opp.com/minimal-rust-kernel/](https://os.phil-opp.com/minimal-rust-kernel/)