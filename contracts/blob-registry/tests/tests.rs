#![cfg(test)]

use near_sdk::NearToken;
use serde_json::json;

#[tokio::test]
async fn new() -> anyhow::Result<()> {
    // Create a new sandbox for testing.
    let worker = near_workspaces::sandbox().await?;
    // Compile the contract.
    let wasm = near_workspaces::compile_project(".").await?;
    // Deploy the (wasm) contract into the sandbox.
    let contract = worker.dev_deploy(&wasm).await?;
    // Create a dev account for testing.
    let alice = worker.dev_create_account().await?;

    // Calling contract's `new()`
    contract
        .call("new")
        .args_json(json!({ "owner_id": alice.id() }))
        .transact()
        .await?
        .into_result()?;

    // `alice` is implicitly set as owner
    let owner = contract.view("own_get_owner").await?.json::<String>()?;
    assert_eq!(owner, alice.id().as_str(), "alice should be the owner");

    Ok(())
}

#[tokio::test]
async fn register_consumer_not_enough_funds_errs() -> anyhow::Result<()> {
    // Create a new sandbox for testing.
    let worker = near_workspaces::sandbox().await?;
    // Compile the contract.
    let wasm = near_workspaces::compile_project(".").await?;
    // Deploy the (wasm) contract into the sandbox.
    let contract = worker.dev_deploy(&wasm).await?;
    // Create a dev account for testing.
    let alice = worker.dev_create_account().await?;

    // Calling contract's `new()`
    contract
        .call("new")
        .args_json(json!({ "owner_id": alice.id() }))
        .transact()
        .await?
        .into_result()?;

    // Register a consumer
    let registration = alice
        .call(contract.id(), "register_consumer")
        .args_json(json!({ "namespace": 0 }))
        .deposit(NearToken::from_millinear(99).as_yoctonear())
        .transact()
        .await?
        .into_result();

    assert!(registration.is_err());

    Ok(())
}

#[tokio::test]
async fn submit() -> anyhow::Result<()> {
    // Create a new sandbox for testing.
    let worker = near_workspaces::sandbox().await?;
    // Compile the contract.
    let wasm = near_workspaces::compile_project(".").await?;
    // Deploy the (wasm) contract into the sandbox.
    let contract = worker.dev_deploy(&wasm).await?;
    // Create a dev account for testing.
    let alice = worker.dev_create_account().await?;

    // Calling contract's `new()`
    contract
        .call("new")
        .args_json(json!({ "owner_id": alice.id() }))
        .transact()
        .await?
        .into_result()?;

    // Register a consumer
    alice
        .call(contract.id(), "register_consumer")
        .args_json(json!({ "namespace": 0 }))
        .deposit(NearToken::from_millinear(100).as_yoctonear())
        .transact()
        .await?
        .into_result()?;

    let mut tx_ids = vec![];
    for _ in 0..100 {
        tx_ids.push(hex::encode([1u8; 32]));
    }

    eprintln!("Submitting {} TX IDs...", tx_ids.len());
    let result = alice
        .call(contract.id(), "submit")
        .args_json(json!({ "namespace": 0, "_transaction_ids": tx_ids }))
        .transact()
        .await?
        .into_result()?;

    println!("Gas burned: {}", result.total_gas_burnt);

    Ok(())
}

#[tokio::test]
async fn owner_change() -> anyhow::Result<()> {
    // Create a new sandbox for testing.
    let worker = near_workspaces::sandbox().await?;
    // Compile the contract.
    let wasm = near_workspaces::compile_project(".").await?;
    // Deploy the (wasm) contract into the sandbox.
    let contract = worker.dev_deploy(&wasm).await?;
    // Create a dev account for testing.
    let alice = worker.dev_create_account().await?;

    // Calling contract's `new()`
    contract
        .call("new")
        .args_json(json!({ "owner_id": alice.id() }))
        .transact()
        .await?
        .into_result()?;

    // `alice` is implicitly set as owner
    let owner = contract.view("own_get_owner").await?.json::<String>()?;
    assert_eq!(owner, alice.id().as_str(), "alice should be the owner");

    // test switching ownership
    let bob = worker.dev_create_account().await?;

    // Alice proposes Bob as the new owner
    alice
        .call(contract.id(), "own_propose_owner")
        .args_json(json!({
            "account_id": bob.id(),
        }))
        .deposit(NearToken::from_yoctonear(1).as_yoctonear())
        .transact()
        .await?
        .unwrap();

    // Bob accepts the ownership
    bob.call(contract.id(), "own_accept_owner")
        .deposit(NearToken::from_yoctonear(1).as_yoctonear())
        .transact()
        .await?
        .unwrap();

    // Check if Bob is the new owner
    let owner = contract.view("own_get_owner").await?.json::<String>()?;
    assert_eq!(owner, bob.id().as_str(), "bob should be the owner");

    Ok(())
}
