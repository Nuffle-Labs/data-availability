package near_test

import (
	"testing"
	"unsafe"

	near "github.com/nuffle-labs/data-availability/gopkg/da-rpc"
	sidecar "github.com/nuffle-labs/data-availability/gopkg/sidecar"
	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"
)

var (
	stubKey  string = "ed25519:4dagBsEqCv3Ao5wa4KKFa57xNAH4wuBjh9wdTNYeCqDSeA9zE7fCnHSvWpU8t68jUpcCGqgfYwcH68suPaqmdcgm"
	localNet string = "http://127.0.0.1:3030"
	account  string = "test.near"
	contract string = "test.near"
)

func TestFrameRefMarshalBinary(t *testing.T) {
	id := make([]byte, 32)
	copy(id, []byte("11111111111111111111111111111111"))
	frameRef := near.BlobRef{
		TxId: id,
	}
	binary, err := frameRef.MarshalBinary()
	if err != nil {
		t.Error(err)
	}
	if len(binary) != sidecar.EncodedBlobRefSize {
		t.Error("Expected binary length to be 64")
	}
	if string(binary[:sidecar.EncodedBlobRefSize]) != string(id) {
		t.Error("Expected id to be equal")
	}
}

func TestFrameRefUnmarshalBinary(t *testing.T) {
	bytes := make([]byte, sidecar.EncodedBlobRefSize)
	copy(bytes, []byte("1111111111111111111111111111111122222222222222222222222222222222"))
	blobRef := near.BlobRef{}
	err := blobRef.UnmarshalBinary(bytes)
	if err != nil {
		t.Error(err)
	}
	if string(blobRef.TxId) != "11111111111111111111111111111111" {
		t.Error("Expected id to be equal")
	}
}

func TestNewConfig(t *testing.T) {
	accountN := "testaccount"
	contractN := "testcontract"
	keyN := "testkey"
	networkN := "Testnet"
	ns := uint32(123)

	config, err := near.NewConfig(accountN, contractN, keyN, networkN, ns)

	assert.NoError(t, err)
	assert.NotNil(t, config)
	assert.Equal(t, near.Namespace{Version: 0, Id: ns}, config.Namespace)
	assert.NotNil(t, config.Client)

	println(config)
	if config.Namespace.Id != ns {
		t.Error("Expected namespace id to be equal")
	}
	if config.Namespace.Version != 0 {
		t.Error("Expected namespace version to be equal")
	}

	// Test error cases
	_, err = near.NewConfig(accountN, contractN, keyN, "InvalidNetwork", ns)
	assert.Error(t, err)
	assert.Equal(t, near.ErrInvalidNetwork, err)
}

func TestNewConfigFile(t *testing.T) {
	keyPathN := "testkey.json"
	contractN := "testcontract"
	networkN := "http://127.0.0.1:3030"
	ns := uint32(1)

	config, err := near.NewConfigFile(keyPathN, contractN, networkN, ns)
	assert.NoError(t, err)
	assert.NotNil(t, config)
	assert.Equal(t, near.Namespace{Version: 0, Id: ns}, config.Namespace)
	assert.NotNil(t, config.Client)

	// Test error cases
	_, err = near.NewConfigFile(keyPathN, contractN, "InvalidNetwork", ns)
	require.Error(t, err)
	require.Equal(t, near.ErrInvalidNetwork, err)

	println(config)
	if config.Namespace.Id != 1 {
		t.Error("Expected namespace id to be equal")
	}
	if config.Namespace.Version != 0 {
		t.Error("Expected namespace version to be equal")
	}
}

func liveConfig(t *testing.T) *near.Config {
	config, err := near.NewConfig(account, contract, stubKey, localNet, 0)
	require.NotNil(t, config)
	require.NoError(t, err)
	return config
}

func TestFreeClient(t *testing.T) {
	config, _ := near.NewConfig(account, contract, stubKey, "Testnet", 1)
	config.FreeClient()
	assert.Nil(t, config.Client)
}

func TestLiveSubmitRetrieve(t *testing.T) {
	candidateHex := "0xfF00000000000000000000000000000000000000"
	data := []byte("test data")

	config := liveConfig(t)

	blobRef, err := config.Submit(candidateHex, data)
	require.NoError(t, err)
	require.NotEmpty(t, blobRef)

	txIndex := uint32(0)

	data, err = config.Get(blobRef, txIndex)
	assert.NoError(t, err)
	assert.NotEmpty(t, data)
}

func TestLiveForceSubmit(t *testing.T) {
	data := []byte("test data")

	config := liveConfig(t)

	frameData, err := config.ForceSubmit(data)
	assert.NoError(t, err)
	assert.NotEmpty(t, frameData)

	// Test error cases
	// TODO: Add test cases for error scenarios
}

func TestToBytes(t *testing.T) {
	b := []byte{1, 2, 3}
	blob := near.NewBlobSafe(b)
	bytes := near.ToBytes(blob)
	assert.Equal(t, b, bytes)
}

func TestTo32Bytes(t *testing.T) {
	data := []byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32}
	ptr := unsafe.Pointer(&data[0])

	bytes := near.To32Bytes(ptr)
	assert.Equal(t, data, bytes)
}

func TestGetDAError(t *testing.T) {
	// Test error case
	near.TestSetError("test error")
	err := near.GetDAError()
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "test error")

	// // Test no error case
	err = near.GetDAError()
	assert.NoError(t, err)
}
