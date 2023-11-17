build:
	cargo +nightly --config "build.rustflags='-Clink-arg=-T$(shell pwd)/link.ld'" build --target riscv64gc-unknown-none-elf --release

bin: build
	rust-objcopy --binary-architecture=riscv64 --strip-all -O binary ../../target/riscv64gc-unknown-none-elf/release/ex5_hello ./hello_app.bin

#img: bin
#	dd if=/dev/zero of=./apps.bin bs=1M count=32
#	dd if=./hello_app.bin of=./apps.bin conv=notrunc

app: bin
	mkdir -p ../../payload
	rm -rf ../../payload/*
	mv ./hello_app.bin ../../payload/001hello.app