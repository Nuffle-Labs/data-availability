# CLI

This is a CLI for interacting with the NEAR DA smart contract. It allows you to configure the client, get blobs, and submit blobs to the contract.

## Usage

### Configure

Provide a `da_config.json` file with the following contents:
```json
{
    "account_id": "throwayaccount.testnet",
    "secret_key": "ed25519:zmF3hHyozS6sEutTSHep1ZS51E8B5pybAJt1yvVaFe9DWNTbXwtRYv4AQ5xAvXJFpqggMPtbdP3MkKViswbYc29",
    "contract_id": "throwayaccount.testnet",
    "network": "Testnet",
    "namespace": {
        "version": 1,
        "id": 1
    }
}
```

Alternatively, you can use the `--account-id`, `--secret-key`, `--contract-id`, `--network`, and `--namespace` flags to configure the client.

## Commands

### Submit

```sh
$ cargo run --bin near-da-cli submit <blob>
```
### Get

```sh
cargo run --bin  near-da-cli get <transaction_id>
```
