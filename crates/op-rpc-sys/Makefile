
op-rpc-sys:
	cargo build --release
	mkdir -p ../../op-stack/da-rpc/lib
	cp -f ../../target/release/libnear*.h ../../op-stack/da-rpc/lib/
	cp -f ../../target/release/libnear*.a ../../op-stack/da-rpc/lib/
	cat ../../op-stack/da-rpc/lib/libnear-da-op-rpc-sys.h

clean:
	rm -rf ../../op-stack/da-rpc/lib
	mkdir -p ../../op-stack/da-rpc/lib

install: clean op-rpc-sys
	ls -lh ../../op-stack/da-rpc/lib

test-install: install
	cd ../../op-stack/optimism && make op-batcher && op-batcher/bin/op-batcher --help

musl:
	~/.cargo/bin/cross build --target x86_64-unknown-linux-musl --release

docker:
	DOCKER_BUILDKIT=1 docker build --progress=plain -t near/op-rpc:bullseye -f Dockerfile ../../
