package plasma

import (
	"context"
	"encoding/json"
	"fmt"
	"os"
	"testing"

	"github.com/ethereum/go-ethereum/log"

	sidecar "github.com/nuffle-labs/data-availability/gopkg/sidecar"
	"github.com/ory/dockertest/v3"

	oplog "github.com/ethereum-optimism/optimism/op-service/log"
	"github.com/stretchr/testify/require"
)

const (
	privateKey = "SIGNER_PRIVATE_KEY"
	ethRPC     = "ETHEREUM_RPC"
	transport  = "http"
	svcName    = "eigenda_proxy"
	host       = "127.0.0.1"
	holeskyDA  = "disperser-holesky.eigenda.xyz:443"
)

var (
	IsE2e bool
)

func ParseEnv() {
	IsE2e = len(os.Getenv("NEAR_PLASMA_E2E")) > 0
}

type TestSuite struct {
	Ctx     context.Context
	Log     log.Logger
	Sidecar *sidecar.Client
}

func CreateTestSuite(t *testing.T, useMemory bool) (TestSuite, func()) {

	ctx := context.Background()

	// load signer key from environment
	pk := os.Getenv(privateKey)
	if pk == "" && !useMemory {
		t.Fatal("SIGNER_PRIVATE_KEY environment variable not set")
	}

	// load node url from environment
	ethRPC := os.Getenv(ethRPC)
	if ethRPC == "" && !useMemory {
		t.Fatal("ETHEREUM_RPC environment variable is not set")
	}

	log := oplog.NewLogger(os.Stdout, oplog.CLIConfig{
		Level:  log.LevelDebug,
		Format: oplog.FormatLogFmt,
		Color:  true,
	}).New("role", svcName)

	// TODO: instantiate sidecar docker and expose localhost:5008 for this go test
	configPath := selectPath(t)

	kill := DockerImages(t, configPath)
	client := initClient(t, configPath)
	require.NotNil(t, client)

	return TestSuite{
		Ctx:     ctx,
		Log:     log,
		Sidecar: client,
	}, kill
}

func selectPath(t *testing.T) string {
	// absolute path of project configPath, then relative to that
	var configPath = os.Getenv("GOPATH")
	if len(configPath) == 0 {
		t.Fatal("GOPATH environment variable is not set")
	}

	// Append the right directory
	if IsE2e {
		configPath += "/http-config.json"
	} else {
		configPath += "/test/http-config.json"
	}
	return configPath
}

func initClient(t *testing.T, path string) *sidecar.Client {
	configData, err := os.ReadFile(path)

	log.Debug("initClient configData ", string(configData))
	if err != nil {
		log.Warn("failed to read config file, using default config: ", err)
	}

	// Unmarshal the JSON data into a ConfigureClientRequest struct
	var conf sidecar.ConfigureClientRequest
	err = json.Unmarshal(configData, &conf)
	require.Nil(t, err)

	client, err := sidecar.NewClient("http://localhost:5888", &conf)
	require.Nil(t, err)

	err = client.ConfigureClient(&conf)
	require.Nil(t, err)

	return client
}

func DockerImages(t *testing.T, configPath string) func() {
	// uses a sensible default on windows (tcp/http) and linux/osx (socket)
	pool, err := dockertest.NewPool("near-da-plasma")
	require.Nil(t, err)

	err = pool.Client.Ping()
	require.Nil(t, err)

	// pulls an image, creates a container based on it and runs it
	// Build and run the given Dockerfile

	t.Log("Starting sidecar")
	sidecar, err := pool.BuildAndRunWithOptions("../../bin/http-api/Dockerfile", &dockertest.RunOptions{
		Name:         "near-da-sidecar",
		Env:          []string{},
		Cmd:          []string{"-c", "/app/config.json"},
		Mounts:       []string{fmt.Sprintf("%s:%s", configPath, "/app/config.json")},
		ExposedPorts: []string{"5888/tcp"},
	})
	require.Nil(t, err)
	sidecar.Expire(240) // Tell docker to hard kill the container in 60 seconds

	var localnet *dockertest.Resource = nil
	if !IsE2e {
		t.Log("Starting localnet")
		resource, err := pool.BuildAndRunWithOptions("../sandbox.Dockerfile", &dockertest.RunOptions{
			Name:         "near-localnet",
			Env:          []string{},
			Mounts:       []string{"near-sandbox-data:/root/.near"},
			ExposedPorts: []string{"3030/tcp"},
		})
		require.Nil(t, err)
		localnet = resource
		resource.Expire(240) // Tell docker to hard kill the container in 60 seconds
	}

	// as of go1.15 testing.M returns the exit code of m.Run(), so it is safe to use defer here
	kill := func() {
		if err := pool.Purge(sidecar); err != nil {
			t.Fatalf("Could not purge resource: %s", err)
		}
		if !IsE2e {
			if err := pool.Purge(localnet); err != nil {
				t.Fatalf("Could not purge resource: %s", err)
			}
		}

	}

	return kill
}
