# Data Availability

<!-- [![Tests](https://github.com/near/data-availability/actions/workflows/on_pull_request.yml/badge.svg)](https://github.com/near/data-availability/actions/workflows/on_pull_request.yml) -->
<!-- [![Deploy](https://github.com/near/data-availability/actions/workflows/on_main.yml/badge.svg)](https://github.com/near/data-availability/actions/workflows/on_main.yml) -->

Utilising NEAR as storage data availability with a focus on lowering rollup DA fees.

## Components

Herein outlines the components of the project and their purposes.

### Blob store contract

This contract provides the store for arbitrary DA blobs. In practice, these "blobs" are sequencing data from rollups, but they can be any data.

NEAR blockchain state storage is pretty cheap. To limit the costs of NEAR storage even more, we don't store the blob data in the blockchain state.

It works by taking advantage of NEAR consensus around receipts.
When a chunk producer processes a receipt, there is consensus around the receipt.
However, once the chunk has been processed and included in the block, the receipt is no longer required for consensus and can be pruned. The pruning time is at least 3 NEAR epochs, where each epoch is 12 Hours; in practice, this is around five epochs.
Once the receipt has been pruned, archival nodes are responsible for retaining the transaction data, and we can even get the data from indexers.

A blob commitment is a set of NEAR transaction IDs, depending on the blob size. To verify the submission of a blob on NEAR, you can verify with your received commitment via a Merkle inclusion proof with the [near light client](https://github.com/near/near-light-client/tree/master) or use the rough-ish ZK Light Clients. For dirty work, you can also call the Aurora Rainbow Bridge via a view call on Ethereum, with some data transformation. There was some WIP work done to work with the experimental Merkle pollarding from near-light-client [here](https://github.com/dndll/rainbow-bridge/commit/3b8e7b92aac0ca873260b85ac7c9bf8a62856c9f).

What this means:

- consensus is provided around the submission of a blob by NEAR validators
- full nodes store the function input data for at least three days
- archival nodes can store the data for longer
- we don't occupy consensus with more data than needs to be
- indexers can also be used, and this Data is currently indexed by all significant explorers in NEAR
- the commitment is available for a long time, and the commitment is straightforward to create

### Light client

A trustless off-chain light client for NEAR with DA-enabled features.

The light client provides easy access to transaction and receipt inclusion proofs within a block or chunk.
This is useful for checking any dubious blobs which may not have been submitted or validating that a blob has been submitted to NEAR.

A blob submission can be verified by:

- taking the NEAR transaction ID from Ethereum for the blob commitment.
If you're feeling specific, Ask the light client for inclusion proof for the transaction ID or the receipt ID; this will give you a Merkle inclusion proof for the transaction/receipt.
- once you have the inclusion proof, you can ask the light client to verify the proof for you, or advanced users can manually verify it themselves.
- armed with this knowledge, rollup providers can have advanced integration with light clients and build proving systems around it.

In the future, we will provide extensions to light clients to supply non-interactive proofs for blob commitments and other data availability features.

It's also possible that the light client may be on-chain for the header syncing and inclusion proof verification, but this is a low priority right now.

TODO: write and draw up extensions to the light client and draw an architecture diagram

### HTTP Sidecar

This sidecar facilitates all of the NEAR and Rust interactions over a network. With this approach, we can let anyone build a rollup in any language and reduce the maintenance effort of a client SDK for every rollup SDK. To use it, you can make use of the [go library](./gopkg/sidecar) or any other HTTP client. 

- JsonRPC coming soon.
- Shareable configs coming soon.

Deploying it is simple, it all uses the config via the `http-config.json` or can be configured via a PUT to the `/configure` endpoint.

Endpoints can be viewed [here](https://github.com/Nuffle-Labs/data-availability/blob/adb04fd2ead936948d3fce42caf911c7fa268437/bin/sidecar/src/main.rs#L214). We're in the process of creating a `bruno` collection, so only the Plasma endpoints are on there, but feel free to add the other ones if you're adding them - we'd welcome the PR.

It is OP Plasma-ready.

Further deployment info can be seen in the [compose file at the root of the repo](./docker-compose.yml)

### DA RPC Client

This client has been usurped by the sidecar approach for most rollup SDKs; as such, we recommend using the sidecar from now on, unless you use Rust, you can natively use this crate. If there are any dependency incompatibilities, feel free to raise an issue or submit a PR. We strive to make our crates as permissive as we can.

These crates allow a client to interact with the blob store.
It can be treated as a "black box", where blobs go in, and `[transaction_id]` emerges.

The `da-rpc` crate is the rust client, which anyone can use if they prefer rust in their application.
The responsibility of this client is to provide a simple interface for interacting with NEAR DA.

## Integrations

We have some proof of concept works for integrating with other rollups.
We are working to prove the system's capabilities and provide a reference implementation for others to follow.
They are being actively developed, so they are in flux.

Each rollup has different features and capabilities, even if built on the same SDK. The reference implementations are not necessarily "production grade". 
They serve as inspiration to help integrators make use of NEAR DA in their system. Our ultimate goal is to make NEAR DA as pluggable as any other tool
you might use. This means our heavy focus is on proving, submission and making storage as fair as possible.

Architecture Diagrams can be viewed at [this directory](./docs/)

### OP Stack

https://github.com/near/optimism

We have integrated it with the Optimism OP stack. Utilising the `Batcher` for submissions to NEAR and the `proposer` for submitting NEAR commitment data to Ethereum.

We also have created endpoints for plasma in the sidecar.

### CDK Stack

https://github.com/0xPolygon/cdk-validium-node/pull/129

We have natively integrated with the Polygon CDK stack and implemented all their E2E suite. 

### Arbitrum Nitro

https://github.com/near/nitro

We have integrated a small plugin into the DAC `daserver`. This is much like our http sidecar and provides a very modular integration into NEAR DA whilst supporting arbitrum 
DACs. In the future, this will likely be the easiest way to support NEAR DA as it acts as an independent sidecar which can be scaled as needed. This also means that the DAC
can opt in and out of NEAR DA, lowering their infrastructure burden. With this approach, the DAC committee members need a "dumb" signing service, with the store backed
by NEAR.

### 👷🚧 Intregrating your own rollup 🚧👷

NEAR DA aims to be as modular as possible. Most rollups now support some form of DAserver, such as `daserver` on Arbitrum Nitro, `plasma` on OP, and the submission interface on CDK. 

Implementing your rollup should be straightforward, assuming you can utilise `da-rpc` or `da-rpc-go`(with some complexity here).
All the implementations so far have been different, but the general rules have been:

- find where the sequencer normally posts batch data, for optimism it was the `batcher`, for CDK it's the `Sequence Sender` and plug the client in.
- find where the sequencer needs commitments posted, for optimism it was the `proposer`, and CDK the `synchronizer`. Hook the blob reads from the commitment there.

The complexity arises depending on how pluggable the contract commitment data is. If you can add a field, that would be great! But these waters are mostly unchartered.

If your rollup does anything additional, feel free to hack, and we can try to reach the goal of NEAR DA being as modular as possible.

## Getting started

NIX/Devenv, Makefiles, Justfiles and [scripts](./scripts) are floating around, but here's a rundown of how to start with NEAR DA. The main objectives are:
- create near [account](https://docs.near.org/concepts/protocol/account-id) 
- fund near account (testnet faucet or otherwise)
- deploy contract (this document/Makefile)
- sidecar: update http-config.json using the info from keystore & contract. You can use what we do in our tests if you like:
```bash
HTTP_API_TEST_SECRET_KEY=YOUR_SECRET_KEY(is the "private_key" field) \
HTTP_API_TEST_ACCOUNT_ID=YOUR_ACCOUNT_ID \
HTTP_API_TEST_NAMESPACE=null
scripts/enrich.sh
```
- deploy sidecar ([docker-compose file](./docker-compose.yml); if stuck take a look at our [e2e tests on CI](./.github/workflows/on_pull_request.yml))



**Prerequisites**

Rust, go, cmake, and friends should be installed. For a list of required installation items, please look at `flake.nix#nativeBuildInputs`.
If you use Nix, you're in luck! Just do `direnv allow`, and you're good to go.

[Ensure you have setup](https://docs.near.org/tools/near-cli-rs) `near-cli`.
For the Makefiles to work correctly, you need to have the `near-cli-rs` version of NEAR-CLI.
Make sure you setup some keys for your contract, the documentation above should help.
You can write these down, or query these from `~/.near-credentials/**` later.

If you didn't clone with submodules, sync them:
`make submodules`

Note that there are some semantic differences between `near-cli-rs` and `near-cli-js`. Notably, the keys generated with `near-cli-js` used to have an `account_id` key in the JSON object. But this is omitted in `near-cli-rs` because it's already in the filename, but some applications require this object. So you may need to add it back in.

### If using your contract

If you're using your own contract, you must build it yourself and make sure you set the keys.

To build the contract:

`make build-contracts`

The contract will now be in `./target/wasm32-unknown-unknown/release/near_da_blob_store.wasm`.

Now, to deploy, once you've decided where you want to deploy and have permission to do so.
Set `$NEAR_CONTRACT` to the address you want to deploy and sign with.
Advanced users should look at the command and adjust it as needed.

Next up:
`make deploy-contracts`

Remember to update your `.env` file for `DA_KEY`, `DA_CONTRACT`, and `DA_ACCOUNT` for later use.

### If deploying optimism

First, clone the [repository](https://github.com/near/optimism)

Configure `./ops-bedrock/.env.example`.
This needs copying without the `.example` suffix and adding the keys, contract address, and signer from your NEAR wallet. It should work out of the box.

#### If deploying optimism on arm64

You can use a docker image to standardize the builds for da-rpc-sys and genesis.

`da-rpc-sys-unix`
This will copy the contents of `da-rpc-sys-docker` generated libraries to the `gopkg/da-rpc` folder.

`op-devnet-genesis-docker`
This will create a docker image to generate the genesis files.

`op-devnet-genesis`

This will generate the genesis files in a docker container and push the files to the `.devnet` folder.

`make op-devnet-up`
This should build the docker images and deploy a local devnet for you

Once up, observe the logs.

`make op-devnet-da-logs`

You should see `got data from NEAR` and `submitting to NEAR`

Of course, to stop

`make op-devnet-down`

If you just wanna get up and running and have already built the docker images using something like `make bedrock images`, there is a `docker-compose-testnet.yml` in `ops-bedrock` you can play with.

### If deploying polygon CDK

First, clone the [repository](https://github.com/firatNEAR/cdk-validium-node)

Now, we have to pull the docker image containing the contracts.

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

Once the image is downloaded, or advanced users build the image and modify the genesis config for tests, we need to configure an env file again.
The envfile example is at `./cdk-stack/cdk-validium-node/.env.example`, and should be updated with the abovementioned variables.

Now we can do:

`cdk-devnet-up`

This will spawn the devnet and an explorer for each network at `localhost:4000`(L1) and localhost:4001`(L2).

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

### If deploying arbitrum nitro

Build daserver/datool:
`make target/bin/daserver && make target/bin/datool`

Deploy your DA contract as above. 

Update daserver config to introduce new configuration fields:

 "near-aggregator": {
      "enable": true,
      "key": "ed25519:insert_here",
      "account": "helloworld.testnet",
      "contract": "your_deployed_da_contract.testnet",
      "storage": {
        "enable": true,
        "data-dir": "config/near-storage"
      }
    },

`target/bin/datool client rpc store  --url http://localhost:7876 --message "Hello world" --signing-key config/daserverkeys/ecdsa`

Take the hash, check the output:

`target/bin/datool client rest getbyhash --url http://localhost:7877 --data-hash 0xea7c19deb86746af7e65c131e5040dbd5dcce8ecb3ca326ca467752e72915185`
