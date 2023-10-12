# Rollup Data Availability

Utilising NEAR as storage data availability with a focus on lowering rollup DA fees.

## Components

Herein outlines the components of the project and their purposes.

### Blob store contract

This contract provides the store for arbitrary DA blobs. In practice, these "blobs" are sequencing data from rollups, but they can be any data.

NEAR blockchain state storage is pretty cheap. At the time of writing, 100KiB is a flat fee of 1NEAR.
To limit the costs of NEAR storage even more, we don't store the blob data in the blockchain state.

It works by taking advantage of NEAR consensus around receipts.
When a chunk producer processes a receipt, there is consensus around the receipt.
However, once the chunk has been processed and included in the block, the receipt is no longer required for consensus and can be pruned. The pruning time is at least 3 NEAR epochs, where each epoch is 12; in practice, this is around five epochs.
Once the receipt has been pruned, it is the responsibility of archival nodes to retain the transaction data, and we can even get the data from indexers.

We can validate that the blob was retrieved from ecosystem actors in the format submitted by checking the blob commitment.
The blob commitment currently needs to be more efficient and will be improved, but it benefits us because anybody can build this with limited expertise and tooling.
It is created by taking a blob, chunking it into 256-byte pieces, and creating a Merkle tree, where each leaf is a Sha-256 hash of the shard.
The root of the Merkle tree is the blob commitment, which is provided as `[transaction_id ++ commitment]` to the L1 contract, which is 64 bytes.

What this means:

- consensus is provided around the submission of a blob by NEAR validators
- the function input data is stored by full nodes for at least three days
- archival nodes can store the data for longer
- we don't occupy consensus with more data than needs to be
- indexers can also be used, and this Data is currently indexed by all significant explorers in NEAR
- the commitment is available for a long time, and the commitment is straightforward to create

### Light client

A trustless off-chain light client for NEAR with DA-enabled features, Such as KZG commitments, Reed-Solomon erasure coding & storage connectors.

The light client provides easy access to transaction and receipt inclusion proofs within a block or chunk.
This is useful for checking any dubious blobs which may not have been submitted or validating that a blob has been submitted to NEAR.

A blob submission can be verified by:

- taking the NEAR transaction ID from Ethereum for the blob commitment.
- Ask the light client for an inclusion proof for the transaction ID or the receipt ID if you're feeling specific; this will give you a Merkle inclusion proof for the transaction/receipt.
- once you have the inclusion proof, you can ask the light client to verify the proof for you, or advanced users can manually verify it themselves.
- armed with this knowledge, rollup providers can have advanced integration with light clients and build proving systems around it.

In the future, we will provide extensions to light clients such that non-interactive proofs can be supplied for blob commitments and other data availability features.

It's also possible that the light client may be on-chain for the header syncing and inclusion proof verification, but this is a low priority right now.

TODO: write and draw up extensions to the light client and draw an architecture diagram

### DA RPC Client

This client is the defacto client for submitting blobs to NEAR.
These crates allow a client to interact with the blob store.
It can be treated as a "black box", where blobs go in, and `[transaction_id ++ commitment]` emerges.

The `da-rpc` crate is the rust client, which anyone can use if they prefer rust in their application.
The responsibility of this client is to provide a simple interface for interacting with NEAR DA.

The `da-rpc-sys` crate is the FFI client binding for use by non-rust applications. This calls through to `da-rpc` to interact with the blob store, with some additional black box functionality for dealing with pointers wrangling and such.

The `da-rpc-go` crate is the go client bindings for use by non-rust applications, and this calls through to `da-rpc-sys`, which provides another application-level layer for easy interaction with the bindings.

## Integrations

We have some proof of concept works for integrating with other rollups.
We are working to prove the system's capabilities and provide a reference implementation for others to follow.
They are being actively developed, so they are in a state of flux.

Architecture Diagrams can be viewed at [this directory](./docs/)

### OP Stack

We have integrated with the Optimism OP stack. Utilising the `Batcher` for submissions to NEAR and the `proposer` for submitting NEAR commitment data to Ethereum.

`./op-stack` contains a few projects:

- `optimism` => Sequencer, Batcher, Proposer, etc. This is the rollup node.
- `da-rpc-go` => Formerly `da-rpc`. This is the go package for integrating with `da-rpc-sys`.

Note that eventually, `optimism` will become its repository, heavily leaning on `da-rpc-go` as a package.
TODO: write a ticket for this

### CDK Stack

`./cdk-stack` contains some projects, too:

- `cdk-validium-contracts` => This contains the contract modifications for removing the CDK DAC signing attestations and adding the Blob commitments.
- `cdk-validium-node` => This contains the modifications for submitting Sequence batches to NEAR, and passing the commitment data through to Ethereum.

Note eventually, `cdk-validium-node` will become its repository, heavily leaning on `da-rpc-go` as a package.
TODO: write a ticket for this

### 👷🚧 Intregrating your own rollup 🚧👷

The aim of NEAR DA is to be as modular as possible.

If implementing your own rollup, it should be fairly straightforward, assuming you can utilise `da-rpc` or `da-rpc-go`(with some complexity here).
All the implementations so far have been different, but the general rules have been:

- find where the sequencer normally posts batch data, for optimism it was the `batcher`, for CDK it's the `Sequence Sender` and plug the client in.
- find where the sequencer needs commitments posted, for optimism it was the `proposer`, and CDK the `synchronizer`. Hook the blob reads from the commitment there.

The complexity arises, depending on how pluggable the commitment data is in the contracts. If you can simply add a field, great! But these waters are unchartered mostly.

If your rollup does anything additional, feel free to hack, and we can try reach the goal of NEAR DA being as modular as possible.

## Getting started

Makefiles are floating around, but here's a rundown of how to start with NEAR DA.

**Prerequisites**

Rust, go, cmake & friends should be installed. Please look at `flake.nix#nativeBuildInputs` for a list of required installation items.
If you use Nix, you're in luck! Just do `direnv allow`, and you're good to go.

[Ensure you have setup](https://docs.near.org/tools/near-cli-rs) `near-cli`.
For the Makefiles to work correctly, you need to have the `near-cli-rs` version of NEAR-CLI.
Make sure you setup some keys for your contract, the documentation above should help.
You can write these down, or query these from `~/.near-credentials/**` later.

If you didn't clone with submodules, sync them:
`make submodules`

Note, there are some semantic differences between `near-cli-rs` and `near-cli-js`. Notably, the keys generated with `near-cli-js` used to have and `account_id` key in the json object. But this is omitted in `near-cli-rs` becuse it's already in the filename, but some applications require this object. So you may need to add it back in.

### If using your own contract

If you're using your own contract, you have to build the contract yourself. And make sure you set the keys.

To build the contract:

`make build-contracts`

The contract will now be in `./target/wasm32-unknown-unknown/release/near_da_blob_store.wasm`.

Now to deploy, once you've decided where you want to deploy to, and have permissions to deploy it.
Set `$NEAR_CONTRACT` to the address you want to deploy to, and sign with.
For advanced users, take a look at the command and adjust as fit.

Next up:
`make deploy-contracts`

Don't forget to update your `.env` file for `DA_KEY`, `DA_CONTRACT` and `DA_ACCOUNT` for use later.

### If the da-rpc-sys image isn't released yet

We use an FFI library for any go applications that need it, until this is release you've gotta build it locally.

`make da-rpc-docker`

This should tag an image which can be used by the integrations, until we eventually publish the package.

Build the `da-rpc-sys` FFI lib:

`make da-rpc`

This will ensure you installed the prerequisites for local development and output the header files for the `go` client.

`make da-rpc-docker`

This will build a docker image for you, which builds a `cdylib` for use by the docker images.
These automagically require these in the dockerfile when you start the local networks.

### If the light client image hasn't been released yet

As part of deploying the devnets, we also deploy the light client.

To build this image, there's a makefile entry for it:

`make light-client-docker`

### If deploying optimism

Configure `./op-stack/optimism/ops-bedrock/.env.example`.
This just needs copying the without `.example` suffix, adding the keys, contract address and signer from your NEAR wallet, and should work out of the box for you.

#### If deploying optimism on arm64

To standardize the builds for da-rpc-sys and genesis, you can use a docker image.

`da-rpc-sys-unix`
This will copy the contents of `da-rpc-sys-docker` generated libraries to the `gopkg/da-rpc` folder.

`op-devnet-genesis-docker`
This will create a docker image to generate the genesis files

`op-devnet-genesis`

This will generate the genesis files in a docker container and push the files in `.devnet` folder.

`make op-devnet-up`
This should build the docker images and deploy a local devnet for you

Once up, observe the logs

`make op-devnet-da-logs`

You should see `got data from NEAR` and `submitting to NEAR`

Of course, to stop

`make op-devnet-down`

If you just wanna get up and running and have already built the docker images using something like `make bedrock images`, there is a `docker-compose-testnet.yml` in `ops-bedrock` you can play with.

### If deploying polygon CDK

First we have to pull the docker image containing the contracts.

**TODO** write docker image to git repo or public artifact registry

`make cdk-images`

**_why is this different to op-stack_**?

When building the contracts in `cdk-validium-contracts`, it does a little bit more than build contracts.
It creates a local eth devnet, deploys the various components (CDKValidiumDeployer & friends).
Then it generates genesis and posts it to L1 at some arbitrary block.
The block number that the L2 genesis gets posted to is **non-deterministic**.
This block is then fed into the `genesis` config in `cdk-validium-node/tests`.
Because of this reason, we want an out of the box deployment, so using a pre-built docker image for this is incredibly convenient.

It's fairly reasonable that, when scanning for the original genesis, we can just query a bunch of blocks between 0..N for the genesis data.
However, this feature doesn't exist yet.

Once the image is downloaded, or advanced users built the image and modified the genesis config for tests, we need to configure an env file again.
The envfile example is at `./cdk-stack/cdk-validium-node/.env.example`, and should be updated with the respective variables as above.

Now we can just do:

`cdk-devnet-up`

This wil spawn the devnet and an explorer for each network at `localhost:4000`(L1) and localhost:4001`(L2).

Run a transaction, and check out your contract on NEAR, verify the commitment with the last 64 bytes of the transaction made to L1.

You'll get some logs that look like:

```
time="2023-10-03T15:16:21Z" level=info msg="Submitting to NEARmaybeFrameData{0x7ff5b804adf0 64}candidate0xfF00000000000000000000000000000000000000namespace{0 99999}txLen1118"
2023-10-03T15:16:21.583Z	WARN	sequencesender/sequencesender.go:129	to 0x0DCd1Bf9A1b36cE34237eEaFef220932846BCD82, data: 438a53990000000000000000000000000000000000000000000000000000000000000060000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb922660000000000000000000000000000000000000000000000000000000000000180000000000000000000000000000000000000000000000000000000000000000233a121c7ad205b875b115c1af3bbbd8948e90afb83011435a7ae746212639654000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000651c2f3400000000000000000000000000000000000000000000000000000000000000005ee177aad2bb1f9862bf8585aafcc34ebe56de8997379cc7aa9dc8b9c68d7359000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000651c303600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040b5614110c679e3d124ca2b7fca6acdd6eb539c1c02899df54667af1ffc7123247f5aa2475d57f8a5b2b3d3368ee8760cffeb72b11783779a86abb83ac09c8d59	{"pid": 7, "version": ""}
github.com/0xPolygon/cdk-validium-node/sequencesender.(*SequenceSender).tryToSendSequence
	/src/sequencesender/sequencesender.go:129
github.com/0xPolygon/cdk-validium-node/sequencesender.(*SequenceSender).Start
	/src/sequencesender/sequencesender.go:69
2023-10-03T15:16:21.584Z	DEBUG	etherman/etherman.go:1136	Estimating gas for tx. From: 0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266, To: 0x0DCd1Bf9A1b36cE34237eEaFef220932846BCD82, Value: <nil>, Data: 438a53990000000000000000000000000000000000000000000000000000000000000060000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb922660000000000000000000000000000000000000000000000000000000000000180000000000000000000000000000000000000000000000000000000000000000233a121c7ad205b875b115c1af3bbbd8948e90afb83011435a7ae746212639654000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000651c2f3400000000000000000000000000000000000000000000000000000000000000005ee177aad2bb1f9862bf8585aafcc34ebe56de8997379cc7aa9dc8b9c68d7359000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000651c303600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040b5614110c679e3d124ca2b7fca6acdd6eb539c1c02899df54667af1ffc7123247f5aa2475d57f8a5b2b3d3368ee8760cffeb72b11783779a86abb83ac09c8d59	{"pid": 7, "version": ""}
2023-10-03T15:16:21.586Z	DEBUG	ethtxmanager/ethtxmanager.go:89	Applying gasOffset: 80000. Final Gas: 246755, Owner: sequencer	{"pid": 7, "version": ""}
2023-10-03T15:16:21.587Z	DEBUG	etherman/etherman.go:1111	gasPrice chose: 8	{"pid": 7, "version": ""}
```

For this transaction, the blob commitment was `7f5aa2475d57f8a5b2b3d3368ee8760cffeb72b11783779a86abb83ac09c8d59`

And if I check the CDKValidium contract `0x0dcd1bf9a1b36ce34237eeafef220932846bcd82`, the root was at the end of the calldata.

`0x438a53990000000000000000000000000000000000000000000000000000000000000060000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb922660000000000000000000000000000000000000000000000000000000000000180000000000000000000000000000000000000000000000000000000000000000233a121c7ad205b875b115c1af3bbbd8948e90afb83011435a7ae746212639654000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000651c2f3400000000000000000000000000000000000000000000000000000000000000005ee177aad2bb1f9862bf8585aafcc34ebe56de8997379cc7aa9dc8b9c68d7359000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000651c303600000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000040b5614110c679e3d124ca2b7fca6acdd6eb539c1c02899df54667af1ffc7123247f5aa2475d57f8a5b2b3d3368ee8760cffeb72b11783779a86abb83ac09c8d59`
