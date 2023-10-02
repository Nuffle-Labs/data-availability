submodules:
	git submodule update --init --recursive

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

# TODO: note to set this
deploy-contracts:
	near contract deploy $$NEAR_CONTRACT use-file ./target/wasm32-unknown-unknown/release/near_da_blob_store.wasm without-init-call network-config testnet sign-with-keychain

op-rpc:
	make -C ./crates/op-rpc-sys
.PHONY: op-rpc

op-rpc-docker:
	make -C ./crates/op-rpc-sys docker
.PHONY: op-rpc-docker

op-devnet-up:
	make -C ./op-stack/optimism devnet-up
.PHONY: devnet-up

op-devnet-down:
	make -C ./op-stack/optimism devnet-down
.PHONY: devnet-down

op-devnet-da-logs:
	docker compose -f op-stack/optimism/ops-bedrock/docker-compose-devnet.yml logs op-batcher | grep NEAR
	docker compose -f op-stack/optimism/ops-bedrock/docker-compose-devnet.yml logs op-node | grep NEAR

TAG_PREFIX := us-docker.pkg.dev/pagoda-solutions-dev/rollup-data-availability
IMAGE_TAG := 0.0.1

bedrock-images:
	cd op-stack && DOCKER_BUILDKIT=1 docker build -t "$(TAG_PREFIX)/op-node:$(IMAGE_TAG)" -f optimism/op-node/Dockerfile .
	cd op-stack && DOCKER_BUILDKIT=1 docker build -t "$(TAG_PREFIX)/op-batcher:$(IMAGE_TAG)" -f optimism/op-batcher/Dockerfile .
	cd op-stack && DOCKER_BUILDKIT=1 docker build -t "$(TAG_PREFIX)/op-proposer:$(IMAGE_TAG)" -f optimism/op-proposer/Dockerfile .
	cd op-stack/optimism/ops-bedrock && DOCKER_BUILDKIT=1 docker build -t "$(TAG_PREFIX)/op-l1:$(IMAGE_TAG)" -f Dockerfile.l1 .
	cd op-stack/optimism/ops-bedrock && DOCKER_BUILDKIT=1 docker build -t "$(TAG_PREFIX)/op-l2:$(IMAGE_TAG)" -f Dockerfile.l2 .
	cd op-stack/optimism && DOCKER_BUILDKIT=1 docker build -t "$(TAG_PREFIX)/op-stateviz:$(IMAGE_TAG)" -f ./ops-bedrock/Dockerfile.stateviz . 
	cd bin/light-client && DOCKER_BUILDKIT=1 docker build -t "$(TAG_PREFIX)/light-client:$(IMAGE_TAG)" -f Dockerfile .
.PHONY: bedrock-images

push-bedrock-images:
	docker push "$(TAG_PREFIX)/op-node:$(IMAGE_TAG)"
	docker push "$(TAG_PREFIX)/op-batcher:$(IMAGE_TAG)"
	docker push "$(TAG_PREFIX)/op-proposer:$(IMAGE_TAG)"
	docker push "$(TAG_PREFIX)/op-l1:$(IMAGE_TAG)"
	docker push "$(TAG_PREFIX)/op-l2:$(IMAGE_TAG)"
	docker push "$(TAG_PREFIX)/op-stateviz:$(IMAGE_TAG)"
	docker push "$(TAG_PREFIX)/light-client:$(IMAGE_TAG)"
.PHONY: push-bedrock-images

cdk-images:
	docker pull us-docker.pkg.dev/pagoda-solutions-dev/rollup-data-availability/cdk-validium-contracts:latest

cdk-devnet-up:
	make -C ./cdk-stack/cdk-validium-node/test run run-explorer
.PHONY: cdk-devnet-up

da-rpc-go:
	make -C ./crates/op-rpc-sys test-install
	cd op-stack/da-rpc && go test -v

light-client-docker:
		make -C ./bin/light-client docker
.PHONY: docker-lightclient

