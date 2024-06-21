package plasma_test

import (
	"testing"
	"time"

	// sidecar "github.com/nuffle-labs/data-availability"
	// "github.com/nuffle-labs/data-availability/test/plasma"

	"github.com/ethereum-optimism/optimism/op-e2e/e2eutils/wait"
	op_plasma "github.com/ethereum-optimism/optimism/op-plasma"

	"github.com/nuffle-labs/data-availability/gopkg/sidecar"
	plasma "github.com/nuffle-labs/data-availability/test/plasma"
	"github.com/stretchr/testify/require"
)

func TestPlasmaClient(t *testing.T) {
	t.Parallel()

	ts, kill := plasma.CreateTestSuite(t, plasma.IsE2e)
	defer kill()

	daClient := op_plasma.NewDAClient(ts.Sidecar.GetHost(), false, false)
	t.Log("Waiting for client to establish connection with plasma server...")

	// wait for the server to come online after starting
	time.Sleep(5 * time.Second)

	// 1 - write arbitrary data to EigenDA

	var testPreimage = []byte("feel the rain on your skin!")

	t.Log("Setting input data on proxy server...")
	commit, err := daClient.SetInput(ts.Ctx, testPreimage)
	require.NoError(t, err)

	// 2 - fetch data from EigenDA for generated commitment key
	t.Log("Getting input data from proxy server...")
	preimage, err := daClient.GetInput(ts.Ctx, commit)
	require.NoError(t, err)
	require.Equal(t, testPreimage, preimage)
}

func TestProxyClient(t *testing.T) {
	t.Parallel()

	ts, kill := plasma.CreateTestSuite(t, plasma.IsE2e)
	defer kill()

	daClient := ts.Sidecar
	t.Log("Waiting for client to establish connection with plasma server...")
	// wait for server to come online after starting
	wait.For(ts.Ctx, time.Second*1, func() (bool, error) {
		err := daClient.Health()
		if err != nil {
			return false, nil
		}

		return true, nil
	})

	// 1 - write arbitrary data to EigenDA

	var testPreimage = []byte("inter-subjective and not objective!")

	t.Log("Setting input data on proxy server...")
	blobInfo, err := daClient.SubmitBlob(sidecar.Blob{Data: testPreimage})
	require.NoError(t, err)

	// // 2 - fetch data from EigenDA for generated commitment key
	t.Log("Getting input data from proxy server...")
	preimage, err := daClient.GetBlob(*blobInfo)
	require.NoError(t, err)
	require.Equal(t, testPreimage, preimage)
}
