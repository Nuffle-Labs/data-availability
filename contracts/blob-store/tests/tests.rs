use borsh::BorshSerialize;
use near_da_primitives::{Blob, Namespace};

#[tokio::test]
async fn test() -> anyhow::Result<()> {
    eprintln!("Initializing sandbox...");
    let worker = near_workspaces::sandbox().await?;

    eprintln!("Setting up accounts...");
    let wasm = near_workspaces::compile_project(".").await?;

    let contract = worker.dev_deploy(&wasm).await?;
    let alice = worker.dev_create_account().await?;

    eprintln!("Calling contract::new()...");

    alice
        .call(contract.id(), "new")
        .transact()
        .await?
        .into_result()?;

    eprintln!("Viewing contract::own_get_owner()...");

    // alice is implicitly set as owner

    let owner = contract.view("own_get_owner").await?.json::<String>()?;

    assert_eq!(owner, alice.id().as_str(), "alice should be the owner");

    let mut blobs = vec![];
    for _ in 0..100 {
        blobs.push(Blob::new_v0(vec![3u8; 256]));
    }
    let blob_ser = blobs.try_to_vec().unwrap();

    eprintln!("Submitting {} blobs...", blobs.len());

    let result = alice
        .call(contract.id(), "submit")
        .args(blob_ser)
        .transact()
        .await?
        .into_result()?;

    eprintln!("Gas burned: {}", result.total_gas_burnt);

    // test switching ownership
    eprintln!("Creating bob...");

    let bob = worker.dev_create_account().await?;

    eprintln!("Proposing bob as new owner...");

    alice
        .call(contract.id(), "own_propose_owner")
        .args_json(near_sdk::serde_json::json!({
            "account_id": bob.id(),
        }))
        .deposit(1)
        .transact()
        .await?
        .unwrap();

    eprintln!("Ownership acceptance by bob...");

    bob.call(contract.id(), "own_accept_owner")
        .deposit(1)
        .transact()
        .await?
        .unwrap();

    let owner = contract.view("own_get_owner").await?.json::<String>()?;

    assert_eq!(owner, bob.id().as_str(), "bob should be the owner");

    Ok(())
}
