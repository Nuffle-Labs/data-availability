
op-rpc-sys:
	cargo build --release
	mkdir -p ../../op-stack/da-rpc/lib
	cp -f ../../target/release/libnear* ../../op-stack/da-rpc/lib/
	cat ../../op-stack/da-rpc/lib/libnear-da-op-rpc-sys.h

install: op-rpc-sys
	ls -l ../../op-stack/da-rpc/lib

test-install: install
	cd ../../op-stack/optimism && make op-node

musl:
	~/.cargo/bin/cross build --target x86_64-unknown-linux-musl --release

docker:
	DOCKER_BUILDKIT=1 docker build --progress=plain -t near/op-rpc:bullseye -f Dockerfile ../../
