package near_test

import (
	"testing"

	near "github.com/near/rollup-data-availability/gopkg/da-rpc"
	"github.com/stretchr/testify/assert"
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
	if len(binary) != 64 {
		t.Error("Expected binary length to be 64")
	}
	if string(binary[:32]) != string(id) {
		t.Error("Expected id to be equal")
	}
}

func TestFrameRefUnmarshalBinary(t *testing.T) {
	bytes := make([]byte, 64)
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
	assert.Equal(t, Namespace{Version: 0, Id: ns}, config.Namespace)
	assert.NotNil(t, config.Client)

	// Test error cases
	_, err = NewConfig(accountN, contractN, keyN, "InvalidNetwork", ns)
	assert.Error(t, err)
	assert.Equal(t, ErrInvalidNetwork, err)

	if err != nil {
		t.Error(err)
	}
	println(config)
	if config.Namespace.Id != 1 {
		t.Error("Expected namespace id to be equal")
	}
	if config.Namespace.Version != 0 {
		t.Error("Expected namespace version to be equal")
	}
}

func TestNewConfigFile(t *testing.T) {
	keyPathN := "testkey.json"
	contractN := "testcontract"
	networkN := "http://127.0.0.1:3030"
	ns := uint32(1)

	config, err := near.NewConfigFile(keyPathN, contractN, networkN, ns)
	assert.NoError(t, err)
	assert.NotNil(t, config)
	assert.Equal(t, Namespace{Version: 0, Id: ns}, config.Namespace)
	assert.NotNil(t, config.Client)

	// Test error cases
	_, err = NewConfigFile(keyPathN, contractN, "InvalidNetwork", ns)
	assert.Error(t, err)
	assert.Equal(t, ErrInvalidNetwork, err)

	println(config)
	if config.Namespace.Id != 1 {
		t.Error("Expected namespace id to be equal")
	}
	if config.Namespace.Version != 0 {
		t.Error("Expected namespace version to be equal")
	}
}

func TestFreeClient(t *testing.T) {
	config, _ := near.NewConfig("account", "contract", "key", "Testnet", 1)
	config.FreeClient()
	assert.Nil(t, config.Client)
}

func TestLiveSubmit(t *testing.T) {
	candidateHex := "0xfF00000000000000000000000000000000000000"
	data := []byte("test data")

	config := &Config{
		Namespace: Namespace{Version: 0, Id: 123},
		Client:    &C.Client{}, // Use a mock client for testing
	}

	frameData, err := config.Submit(candidateHex, data)
	assert.NoError(t, err)
	assert.NotEmpty(t, frameData)

	// Test error cases
	// TODO: Add test cases for error scenarios
}

func TestLiveForceSubmit(t *testing.T) {
	data := []byte("test data")

	config := &Config{
		Namespace: Namespace{Version: 0, Id: 123},
		Client:    &C.Client{}, // Use a mock client for testing
	}

	frameData, err := config.ForceSubmit(data)
	assert.NoError(t, err)
	assert.NotEmpty(t, frameData)

	// Test error cases
	// TODO: Add test cases for error scenarios
}

func TestLiveGet(t *testing.T) {
	frameRefBytes := []byte("test frame ref")
	txIndex := uint32(0)

	config := &Config{
		Namespace: Namespace{Version: 0, Id: 123},
		Client:    &C.Client{}, // Use a mock client for testing
	}

	data, err := config.Get(frameRefBytes, txIndex)
	assert.NoError(t, err)
	assert.NotEmpty(t, data)

	// Test error cases
	// TODO: Add test cases for error scenarios
}

func TestToBytes(t *testing.T) {
	blob := &C.BlobSafe{
		data: unsafe.Pointer(&[]byte{1, 2, 3}[0]),
		len:  C.size_t(3),
	}

	bytes := ToBytes(blob)
	assert.Equal(t, []byte{1, 2, 3}, bytes)
}

func TestTo32Bytes(t *testing.T) {
	data := []byte{1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32}
	ptr := unsafe.Pointer(&data[0])

	bytes := To32Bytes(ptr)
	assert.Equal(t, data, bytes)
}

//TODO:  Test the conversion into near pkey, contract, key, network

func TestGetDAError(t *testing.T) {
	// Test error case
	// TODO: Mock the C.get_error function to return an error
	err := GetDAError()
	assert.Error(t, err)
	assert.Contains(t, err.Error(), "NEAR DA client")

	// Test no error case
	// TODO: Mock the C.get_error function to return nil
	err = GetDAError()
	assert.NoError(t, err)
}
