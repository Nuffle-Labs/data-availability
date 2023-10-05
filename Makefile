
TAG_PREFIX := us-docker.pkg.dev/pagoda-solutions-dev/rollup-data-availability
IMAGE_TAG := 0.0.1

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

# TODO: note to set this
deploy-contracts:
	near contract deploy $$NEAR_CONTRACT use-file ./target/wasm32-unknown-unknown/release/near_da_blob_store.wasm without-init-call network-config testnet sign-with-keychain

da-rpc-sys:
	make -C ./crates/da-rpc-sys
.PHONY: da-rpc-sys

da-rpc-docker:
	make -C ./crates/da-rpc-sys docker TAG_PREFIX=$(TAG_PREFIX) IMAGE_TAG=$(IMAGE_TAG)
.PHONY: da-rpc-docker

op-devnet-up:
	make -C ./op-stack/optimism devnet-up
.PHONY: devnet-up

op-devnet-down:
	make -C ./op-stack/optimism devnet-down
.PHONY: devnet-down

op-devnet-da-logs:
	docker compose -f op-stack/optimism/ops-bedrock/docker-compose-devnet.yml logs op-batcher | grep NEAR
	docker compose -f op-stack/optimism/ops-bedrock/docker-compose-devnet.yml logs op-node | grep NEAR

COMMAND = docker buildx build -t 
bedrock-images: # light-client-docker
	$(COMMAND) "$(TAG_PREFIX)/op-node:$(IMAGE_TAG)" -f op-stack/optimism/op-node/Dockerfile op-stack/optimism
	docker tag "$(TAG_PREFIX)/op-node:$(IMAGE_TAG)" "$(TAG_PREFIX)/op-node:latest"
	
	$(COMMAND) "$(TAG_PREFIX)/op-batcher:$(IMAGE_TAG)" -f op-stack/optimism/op-batcher/Dockerfile op-stack/optimism 
	docker tag "$(TAG_PREFIX)/op-batcher:$(IMAGE_TAG)" "$(TAG_PREFIX)/op-batcher:latest"

	$(COMMAND) "$(TAG_PREFIX)/op-proposer:$(IMAGE_TAG)" -f op-stack/optimism/op-proposer/Dockerfile op-stack/optimism 
	docker tag "$(TAG_PREFIX)/op-proposer:$(IMAGE_TAG)" "$(TAG_PREFIX)/op-proposer:latest"

	$(COMMAND) "$(TAG_PREFIX)/op-l1:$(IMAGE_TAG)" -f op-stack/optimism/ops-bedrock/Dockerfile.l1 op-stack/optimism/ops-bedrock 
	docker tag "$(TAG_PREFIX)/op-l1:$(IMAGE_TAG)" "$(TAG_PREFIX)/op-l1:latest"

	$(COMMAND) "$(TAG_PREFIX)/op-l2:$(IMAGE_TAG)" -f op-stack/optimism/ops-bedrock/Dockerfile.l2 op-stack/optimism/ops-bedrock 
	docker tag "$(TAG_PREFIX)/op-l2:$(IMAGE_TAG)" "$(TAG_PREFIX)/op-l2:latest"

	$(COMMAND) "$(TAG_PREFIX)/op-stateviz:$(IMAGE_TAG)" -f op-stack/optimism/ops-bedrock/Dockerfile.stateviz op-stack/optimism 
	docker tag "$(TAG_PREFIX)/op-stateviz:$(IMAGE_TAG)" "$(TAG_PREFIX)/op-stateviz:latest"
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
	# TODO: when we have public images docker pull "$(TAG_PREFIX)/cdk-validium-contracts:$(IMAGE_TAG)"
	docker pull ghcr.io/dndll/cdk-validium-contracts:latest
	docker tag ghcr.io/dndll/cdk-validium-contracts:latest "$(TAG_PREFIX)/cdk-validium-contracts:$(IMAGE_TAG)"
	

cdk-devnet-up:
	make -C ./cdk-stack/cdk-validium-node/test run run-explorer
.PHONY: cdk-devnet-up

da-rpc-go:
	make -C ./crates/da-rpc-sys test-install
	cd op-stack/da-rpc && go test -v

light-client-docker:
		make -C ./bin/light-client docker TAG_PREFIX=$(TAG_PREFIX) IMAGE_TAG=$(IMAGE_TAG)
.PHONY: docker-lightclient

