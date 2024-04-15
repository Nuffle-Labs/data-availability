NEWCONTRACT=forge script Deploy --fork-url local --broadcast --legacy --json | jq -R 'fromjson?' | jq -r '.returns.da.value'
docker exec zkevm-sequence-sender /app/zkevm-node set-dap --da-addr $NEWCONTRACT --network custom --custom-network-file /app/genesis.json --key-store-path /pk/sequencer.keystore --pw testonly --cfg /app/config.toml
