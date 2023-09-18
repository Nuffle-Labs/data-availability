raen-contracts:
	/home/common/.cargo/bin/raen build --channel nightly --optimize -w -p near-da-blob-store --release

build-optimised-contracts:
	cargo build --package near-da-blob-store -Z=build-std=std,panic_abort -Z=build-std-features=panic_immediate_abort --target wasm32-unknown-unknown --release

build-contracts:
	cargo build --package near-da-blob-store --target wasm32-unknown-unknown --release

deploy-blob-store:
	near contract deploy $$NEAR_CONTRACT use-file ./target/wasm32-unknown-unknown/release/near_da_blob_store.wasm without-init-call network-config testnet sign-with-keychain

op-rpc-sys:
	make -C ./crates/op-rpc-sys
.PHONY: op-rpc-sys

op-rpc-docker:
	make -C ./crates/op-rpc-sys docker
.PHONY: op-rpc-docker

devnet-up:
	make -C ./op-stack/optimism devnet-up
.PHONY: devnet-up

devnet-down:
	make -C ./op-stack/optimism devnet-down
.PHONY: devnet-down

devnet-da-logs:
	docker compose -f op-stack/optimism/ops-bedrock/docker-compose-devnet.yml logs op-batcher | grep NEAR
	docker compose -f op-stack/optimism/ops-bedrock/docker-compose-devnet.yml logs op-node | grep NEAR
	
clear-namespace:
	near contract call-function as-transaction $$NEAR_CONTRACT clear json-args '{"ns":[[0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], [0, 0, 8, 229, 246, 121, 191, 113, 22, 203, 216, 166, 155, 132, 9, 0, 73, 156, 212, 167, 93, 119, 8, 0, 81, 0, 0, 0, 0, 0, 0, 0]]}' \
		prepaid-gas '100.000 TeraGas' \
		attached-deposit '0 NEAR' \
		sign-as $$NEAR_CONTRACT network-config testnet sign-with-keychain send
