TAG_PREFIX := us-docker.pkg.dev/pagoda-solutions-dev/rollup-data-availability
IMAGE_TAG := 0.3.0

format:
	taplo format
	cargo fmt --all

submodules:
	git submodule update --init --recursive
.PHONY: submodules

make pull-submodules:
	git pull --recurse-submodules
.PHONY: pull-submodules

raen-contracts:
	/home/common/.cargo/bin/raen build --channel nightly --optimize -w -p near-da-blob-store --release

# Near contract building
#
# TODO: fix this
build-optimised-contracts:
	cargo build --package near-da-blob-store -Z=build-std=std,panic_abort -Z=build-std-features=panic_immediate_abort --target wasm32-unknown-unknown --release

# Create the blob store contract
build-contracts:
	cargo build --package near-da-blob-store --target wasm32-unknown-unknown --release

test-contracts: build-contracts
	cargo test --package near-da-blob-store --test tests -- --nocapture
.PHONY: test-contracts

# TODO: note to set this
deploy-contracts:
	near contract deploy $$NEAR_CONTRACT use-file ./target/wasm32-unknown-unknown/release/near_da_blob_store.wasm without-init-call network-config testnet sign-with-keychain

da-rpc-sys:
	make -C ./crates/da-rpc-sys
.PHONY: da-rpc-sys

da-rpc-docker:
	make -C ./crates/da-rpc-sys docker TAG_PREFIX=$(TAG_PREFIX) IMAGE_TAG=$(IMAGE_TAG)
.PHONY: da-rpc-docker

da-rpc-sys-unix:
	docker rm dummy
	docker create --name dummy $(TAG_PREFIX)/da-rpc:$(IMAGE_TAG)
	docker cp dummy:/gopkg/da-rpc/lib ./gopkg/da-rpc/lib
	docker rm -f dummy
.PHONY: da-rpc-sys-unix

cdk-images:
	# TODO: when we have public images docker pull "$(TAG_PREFIX)/cdk-validium-contracts:$(IMAGE_TAG)"
	docker pull ghcr.io/dndll/cdk-validium-contracts:latest
	docker tag ghcr.io/dndll/cdk-validium-contracts:latest "$(TAG_PREFIX)/cdk-validium-contracts:$(IMAGE_TAG)"
	$(COMMAND) $(TAG_PREFIX)/cdk-validium-node:latest -f cdk-stack/cdk-validium-node/Dockerfile cdk-stack/cdk-validium-node
	docker tag $(TAG_PREFIX)/cdk-validium-node:latest cdk-validium-node
	
cdk-devnet-up:
	make -C ./cdk-stack/cdk-validium-node/test run run-explorer
.PHONY: cdk-devnet-up

cdk-devnet-down:
	make -C ./cdk-stack/cdk-validium-node/test stop 
.PHONY: cdk-devnet-up

cdk-node:
	make -C ./cdk-stack/cdk-validium-node build
.PHONY: cdk-node

send-cdk-transfers:
	cd cdk-stack/cdk-validium-node/test/benchmarks/sequencer/scripts/erc20-transfers && go run main.go
.PHONY: send-cdk-transfers

cdk-devnet-redeploy-test: cdk-images cdk-devnet-up send-cdk-transfers
.PHONY: cdk-devnet-redeploy-test

