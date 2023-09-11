build-optimised-contracts:
	/home/common/.cargo/bin/raen build --channel nightly --optimize -w -p near-da-blob-store --release

build-contracts:
	cargo build --package near-da-blob-store -Z=build-std=std,panic_abort -Z=build-std-features=panic_immediate_abort --target wasm32-unknown-unknown --release

build-unoptimised-contracts:
	cargo build --package near-da-blob-store --target wasm32-unknown-unknown --release

deploy-blob-store:
	near contract deploy $$NEAR_CONTRACT use-file ./target/wasm32-unknown-unknown/release/near_da_blob_store.wasm without-init-call network-config testnet sign-with-keychain
