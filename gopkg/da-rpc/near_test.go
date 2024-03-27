package near_test

import (
	"errors"
	"testing"

	near "github.com/near/rollup-data-availability/gopkg/da-rpc"
)

func TestFrameRefMarshalBinary(t *testing.T) {
	id := make([]byte, 32)
	copy(id, []byte("11111111111111111111111111111111"))
	commitment := make([]byte, 32)
	copy(commitment, []byte("22222222222222222222222222222222"))
	frameRef := near.FrameRef{
		TxId:         id,
		TxCommitment: commitment,
	}
	binary, err := frameRef.MarshalBinary()
	println(binary, id, commitment)
	if err != nil {
		t.Error(err)
	}
	if len(binary) != 64 {
		t.Error("Expected binary length to be 64")
	}
	if string(binary[:32]) != string(id) {
		t.Error("Expected id to be equal")
	}
	if string(binary[32:]) != string(commitment) {
		t.Error("Expected commitment to be equal")
	}
}

func TestFrameRefUnmarshalBinary(t *testing.T) {
	bytes := make([]byte, 64)
	copy(bytes, []byte("1111111111111111111111111111111122222222222222222222222222222222"))
	frameRef := near.FrameRef{}
	err := frameRef.UnmarshalBinary(bytes)
	if err != nil {
		t.Error(err)
	}
	println(frameRef.TxId, frameRef.TxCommitment)
	if string(frameRef.TxId) != "11111111111111111111111111111111" {
		t.Error("Expected id to be equal")
	}
	if string(frameRef.TxCommitment) != "22222222222222222222222222222222" {
		t.Error("Expected commitment to be equal")
	}

}

func TestNewConfig(t *testing.T) {
	config, err := near.NewConfig("account", "contract", "key", "Testnet", 1)
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
	config, err := near.NewConfigFile("keyPath", "contract", "Localnet", 1)
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

func TestNetworkValidation(t *testing.T) {
	config, err := near.NewConfig("account", "contract", "key", "Randomnet", 1)
	if !errors.Is(err, near.ErrInvalidNetwork) {
		t.Error("Expected ErrInvalidNetwork error")
	}

	if config != nil {
		t.Error("Expected config to be nil")
	}
}

func TestLiveSumbit(t *testing.T) {
	t.Skip("TODO")
}

func TestLiveGet(t *testing.T) {
	t.Skip("TODO")
}

func TestGetDAError(t *testing.T) {
	t.Skip("TODO")
}
