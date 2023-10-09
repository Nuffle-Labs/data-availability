DA_RPC_GO_DIR=../../gopkg/da-rpc

da-rpc-sys:
	cargo build --release
	mkdir -p $(DA_RPC_GO_DIR)/lib
	cp -f ../../target/release/libnear*.h $(DA_RPC_GO_DIR)/lib/
	cp -f ../../target/release/libnear*.a $(DA_RPC_GO_DIR)/lib/
	ls -l $(DA_RPC_GO_DIR)/lib
	cat $(DA_RPC_GO_DIR)/lib/*.h

clean:
	rm -rf $(DA_RPC_GO_DIR)/lib

musl:
	~/.cargo/bin/cross build --target x86_64-unknown-linux-musl --release

TAG_PREFIX?=near
IMAGE_TAG?=0.0.1
docker:
	DOCKER_BUILDKIT=1 docker build --progress=plain -t $(TAG_PREFIX)/da-rpc:$(IMAGE_TAG) -f Dockerfile ../../
	docker tag $(TAG_PREFIX)/da-rpc:$(IMAGE_TAG) $(TAG_PREFIX)/da-rpc:latest

push-docker:
	docker push $(TAG_PREFIX)/da-rpc:$(IMAGE_TAG)-bullseye
	docker push $(TAG_PREFIX)/da-rpc:latest
