TAG_PREFIX := us-docker.pkg.dev/pagoda-solutions-dev/rollup-data-availability
IMAGE_TAG := 0.3.0

#? format: format codes
format:
	taplo format
	cargo fmt --all

#? submodules: update submodules
submodules:
	git submodule update --init --recursive
.PHONY: submodules

make pull-submodules:
	git pull --recurse-submodules
.PHONY: pull-submodules

#? build-contracts: create the blob store contract
build-contracts:
	cargo build --package near-da-blob-store --target wasm32-unknown-unknown --release

#? test-contracts: create the blob store contract and run tests
test-contracts: build-contracts
	cargo test --package near-da-blob-store --test tests -- --nocapture
.PHONY: test-contracts

#? deploy-contracts: deploy the near-da-blob-store contract to the NEAR testnet
deploy-contracts:
	near contract deploy $$NEAR_CONTRACT use-file ./target/wasm32-unknown-unknown/release/near_da_blob_store.wasm with-init-call new json-args {} network-config testnet sign-with-keychain

da-rpc-sys:
	make -C ./crates/da-rpc-sys
.PHONY: da-rpc-sys

#? da-rpc-docker: build docker image
da-rpc-docker:
	make -C ./crates/da-rpc-sys docker TAG_PREFIX=$(TAG_PREFIX) IMAGE_TAG=$(IMAGE_TAG)
.PHONY: da-rpc-docker

#? da-rpc-sys-unix: copy the compiled da-rpc library from the Docker image to the local filesystem
da-rpc-sys-unix:
	docker rm dummy
	docker create --name dummy $(TAG_PREFIX)/da-rpc:$(IMAGE_TAG)
	docker cp dummy:/gopkg/da-rpc/lib ./gopkg/da-rpc/lib
	docker rm -f dummy
.PHONY: da-rpc-sys-unix

#? cdk-images: pull and tag the cdk-validium-contracts and cdk-validium-node Docker images
cdk-images:
	# TODO: when we have public images docker pull "$(TAG_PREFIX)/cdk-validium-contracts:$(IMAGE_TAG)"
	docker pull ghcr.io/dndll/cdk-validium-contracts:latest
	docker tag ghcr.io/dndll/cdk-validium-contracts:latest "$(TAG_PREFIX)/cdk-validium-contracts:$(IMAGE_TAG)"
	$(COMMAND) $(TAG_PREFIX)/cdk-validium-node:latest -f cdk-stack/cdk-validium-node/Dockerfile cdk-stack/cdk-validium-node
	docker tag $(TAG_PREFIX)/cdk-validium-node:latest cdk-validium-node
	
#? cdk-devnet-up: start the cdk-validium-node development network and explorer
cdk-devnet-up:
	make -C ./cdk-stack/cdk-validium-node/test run run-explorer
.PHONY: cdk-devnet-up

#? cdk-devnet-down: stop the cdk-validium-node development network
cdk-devnet-down:
	make -C ./cdk-stack/cdk-validium-node/test stop 
.PHONY: cdk-devnet-up

#? cdk-node: build the cdk-validium-node
cdk-node:
	make -C ./cdk-stack/cdk-validium-node build
.PHONY: cdk-node

#? send-cdk-transfers: run ERC20 transfers script
send-cdk-transfers:
	cd cdk-stack/cdk-validium-node/test/benchmarks/sequencer/scripts/erc20-transfers && go run main.go
.PHONY: send-cdk-transfers

#? cdk-devnet-redeploy-test: build and start the cdk-validium-node development network, then test ERC20 transfers script
cdk-devnet-redeploy-test: cdk-images cdk-devnet-up send-cdk-transfers
.PHONY: cdk-devnet-redeploy-test

#? help: get this help message
help: Makefile
	@echo " Choose a command to run:"
	@sed -n 's/^#?//p' $< | column -t -s ':' |  sort | sed -e 's/^/ /'
.PHONY: help