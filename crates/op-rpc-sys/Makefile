
op-rpc-sys:
	cargo build --release
	mkdir -p ../../lib
	cp -f ../../target/release/libnear* ../../lib/
	ls -l ../../lib	
	cat ../../lib/libnear-da-op-rpc-sys.h
	ldd ../../lib/libnear_da_op_rpc_sys.so

install: op-rpc-sys
	mkdir -p ../../op-stack/optimism/lib
	cp -f ../../lib/libnear_da_op_rpc_sys.a ../../op-stack/optimism/lib/
	cp -f ../../lib/libnear-da-op-rpc-sys.h ../../op-stack/optimism/lib/libnear-da-op-rpc.h
	ls -l ../../op-stack/optimism/lib

test-install: install
	cd ../../op-stack/optimism && make op-node

musl:
	~/.cargo/bin/cross build --target x86_64-unknown-linux-musl --release

docker:
	DOCKER_BUILDKIT=1 docker build --progress=plain -t near/op-rpc:bullseye -f Dockerfile ../../