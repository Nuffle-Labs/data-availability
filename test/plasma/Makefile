
e2e-test:
	INTEGRATION=true go test -timeout 1m -v -parallel 4 -deploy-config .devnet/devnetL1.json

holesky-test:
	TESTNET=true go test -timeout 50m -v ./e2e  -parallel 4 -deploy-config ../.devnet/devnetL1.json
