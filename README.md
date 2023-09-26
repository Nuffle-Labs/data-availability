# Rollup Data Availability

This project enables utilising NEAR as storage data availability for rollups.

## Component Outline

Below outlines the components of the project, and their purpose.

### Blob store

This contract provides the store for arbitrary DA blobs, in practice, these "blobs" are transaction data from rollups.

### Light client

This is an offchain light client for NEAR, with DA enabled features. E.g: KZG, RS & storage

### OP-rpc & \*-sys

These crates provide a client for interacting with the blob store.

The `*-sys` crate is the FFI client bindings used by the `go` node, this calls through to `op-rpc` to interact with the blob store.

`op-rpc-sys` & `optimism` needs testing to ensure we don't have memory leaks.

### OP Stack

`op-stack` contains a few projects, including:

- `optimism` => The op-stack actors as we know it, comprising of sequencer, batcher, proposer, etc.
- `optimism-rs` => This is `a16z/magi`, a rust implementation of op-node. At present it doesn't support sequencing, but we will help them add it later down the line. There is a rough DA implementation, but we should migrate it to using `op-rpc` when we plan to use it.
- `openrpc` => The previously used `go` equivalent of our `op-rpc` crate, this is deprecated and will be removed soon.

## Getting started

There are makefiles floating around the place, but here's a rundown of how to run optimism with DA features.

**Prerequisites**

Rust, go, cmake & friends should be installed. Please take a look at `flake.nix#nativeBuildInputs` for a list of required things to install.

If you use nix, you're in luck! Just do `direnv allow` and you're good to go.

- Sync the git submodules => `git submodule update --init --recursive`
- Build the `op-rpc-sys` ffi lib => `make op-rpc` => this will make sure you installed the prerequisites for local development, and output the header files for the `go` client.
- `make op-rpc-docker` => this will build a docker image for you, which builds a `cdylib` for use by the `optimism` docker images, these automagically require these in the dockerfile when you start the devnet.
- configure `./op-stack/optimism/ops-bedrock/.env.example` => this just needs copying without `.example`, adding a NEAR private key to `DA_KEY`, and should work out of the box for you
- you can configure the contract however you like if you want to use a different signer & contract. (note: you have to build and deploy the contract yourself, see the `Makefile`)
- `make devnet-up` => this should build the docker images and deploy a local devnet for you
- once up, observe the logs, `make devnet-da-logs` => you should see `got data from NEAR` and `submitting to NEAR`
