# Changelog

All notable changes to this project will be documented in this file.

## [0.4.0] - 2024-05-14

### Features

- [**breaking**] Migrate to new http api
- Race archival and base
- Update TGAS for max tx size to be 20
- Introduce settlement mode with pessimistic default for now
- Upgrade cli to use blob structure and mode
- Expose mode to the cli

### Miscellaneous Tasks

- Auto changelog
- Bump version and lints

### Testing

- Migrate to a different test blockhash

## [0.3.0] - 2024-05-08

### Bug Fixes

- Remove test for log
- Localnet url
- Removed hardcoded Testnet from NewConfig
- Clear error on GetDAError
- Import ffi_helpers Nullable trait
- Remove unnecessary unsafe block
- Ffi, ci for everything, mocks, tests

### Co-authored-by

- Don <37594653+dndll@users.noreply.github.com>

### Documentation

- Update readme
- Add notes on nitro

### Features

- Http server
- Added darwin support for gopkg
- Added api for new_client_file
- Add free_client implementation in go
- Localnet listening to an arbitrary loopback address (#91)
- Replace localnet with customnet
- Use urls instead of SocketAddr for custom
- Add clear_error to da-rpc-sys
- Add clear_error to header bindings
- Remove erasure-commit DAS
- [**breaking**] Remove namespace id in blob
- Initial eth DA tracking contract
- [**breaking**] Sidecar go module
- Bump version to 0.3.0

### Miscellaneous Tasks

- Remove bin directory from workspace
- Remove optimism submodule
- Remove cdk submodule
- Remove cdk contracts submodule
- Library linkage
- Remove deserialization & return value
- Use const slice ref for storage keys
- Bump MSRV
- CODEOWNERS
- Create LICENSE
- Move near-da-primitives out of blob store contract
- Slight reuse
- Actions and cleanup

### Refactor

- Remove unnecessary some check on clear_error
- Fix import order

### Testing

- Add error clearing to error handling test
- Remove unnecessary derefs
- Remove unneeded clear since we take the err already
- Add bypass flag for verification until LC is done

### Build

- Lockfile
- Http api docker image

## [0.2.3] - 2023-11-15

### Bug Fixes

- Make sure errors aren't causing segfaults
- Blobs are optional from the contract
- Import math libs in go
- Scripts were moved around
- Make the network lowercase
- Render in github
- Builds for macos
- Cargo build should be locked
- Borsh has been updated and the lockfile wasn't force locked
- This project builds binaries - lockfile committed
- Cdk image should be tagged on rebuild
- All the repos are public now - no need for access token
- Commit the header file for libnear_da_rpc_sys

### Co-authored-by

- Don <37594653+dndll@users.noreply.github.com>
- Don <37594653+dndll@users.noreply.github.com>
- Don <37594653+dndll@users.noreply.github.com>
- Jacob <encody@noreply.users.github.com>

### Documentation

- Readme and scripts
- Update docs for readme
- Arch class diagram for rpc
- Use mermaid code blocks
- Add system context
- Fix render styling
- Add note on fisherman actor
- Add container diagram for optimism
- Add architecture directory to the repository
- Add how-to-integrate comment in the readme
- Fix typo
- Update commitment proposals

### Features

- Update submodules to use DA over NEAR
- Op-rpc with exposed ffi
- Use shared primitives for client & contract
- Ffi client reads
- Generate bindings on build
- Sys crate for go
- Migrate ffi to a sys crate
- Introduce a naive merkleization of commitment blobs
- Use a number instead of unbounded bytes for namespaces
- Allow a user to provide sk instead of a file
- [**breaking**] Remove blobs from state
- Light client failover
- Expose module for near-op-rpc-sys
- Utilise go module for ffi client
- Near DA on polygon CDK
- [**breaking**] Migrate naming to da-rpc
- [**breaking**] Migrate go package to da-rpc-go
- Optimize contract
- Kzg commitments over rs encoded grids
- Kzg codeword proof verification
- Crate
- Commit to columns individually
- Commit to the root
- [**breaking**] Convert witness points to affine

### Miscellaneous Tasks

- Submodules
- Add nix compat
- Switch to near branch for openrpc
- Remove njs for now
- Move contract from near-openrpc to here
- Bookmarks
- Add op node to workspace
- Go workspace
- Submod update
- Combing through magi
- Don't override contracts release profile
- Submodules
- Submodules
- Submodules
- Bump rust version and use optimised resolver
- Update light client submodule
- Update LC
- Update submodules
- Scripts for deploying and building
- Set toolchain to stable
- Update submodule
- Add another node to devnet
- Use private repository from near for optimism
- Remove openrpc
- Remove CDK DA
- Remove optimism-rs for now
- Update submodule
- Add how to get validium contracts image
- Op-stack repository structure
- Cdk stack repository structure
- Submodules track main
- Add tests for rust and go
- Circumvent binstall
- Update submodule for lc
- Submodule
- Fix CDK sequencer spam
- Cdk submodule
- Add time unit to readme for epoch
- Unified dependencies
- Fmt toml and rust
- Remove point compression for now
- Lints and fmt
- Remove light-client submodule
- Use unpublished version until audit
- Publish images
- Mv gopkg so go can read it

### Refactor

- Get_all returns all blobs for a namespace

### Testing

- Compile the contract at test time
- Add kzg from g1 test
- Ignore integration tests

### Build

- Add makefile for building optimised contract
- Dockerfile for op-rpc
- Use light client in devnet docker
- Create makefile entry to push images to the artifact reg
- Update version

### Wip

- Contract flat storage

<!-- generated by git-cliff -->
