package sidecar

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"os"
	"testing"

	log "github.com/sirupsen/logrus"
	"github.com/stretchr/testify/assert"
)

func initClient(t *testing.T) *Client {
	return InitLocalClient(t, "../../test/http-sidecar.json")
}

func InitLocalClient(t *testing.T, path string) *Client {
	configData, err := os.ReadFile(path)
	assert.NoError(t, err)
	log.Debug("initClient configData ", string(configData))

	// Unmarshal the JSON data into a ConfigureClientRequest struct
	var conf ConfigureClientRequest
	err = json.Unmarshal(configData, &conf)
	assert.NoError(t, err)

	client, err := NewClient("http://localhost:5888", &conf)
	assert.NoError(t, err)

	err = client.ConfigureClient(&conf)
	assert.NoError(t, err)

	return client
}

// func TestGetInvalidBlob(t *testing.T) {
// 	client := initClient(t)
// 	defer client.Close()

// 	invalidTransactionID := []byte("invalid_transaction_id")
// 	log.Info("TestGetInvalidBlob invalidTransactionID ", invalidTransactionID)

// 	invalidBlobRef := &BlobRef{}
// 	log.Info("TestGetInvalidBlob invalidBlobRef ", invalidBlobRef)

// 	copy(invalidBlobRef.transactionID[:], invalidTransactionID)
// 	blob, err := client.GetBlob(*invalidBlobRef)
// 	log.Info("TestGetInvalidBlob invalidBlob ", blob)

// 	assert.Error(t, err, "failed to get blob, status code: 500")
// 	assert.Nil(t, blob)
// }

func TestSubmitGetBlob(t *testing.T) {
	testName := "TestSubmitGetBlob "
	client := initClient(t)
	defer client.Close()

	// Test submitting a blob
	data := []byte("test_data")
	blob := &Blob{Data: data}
	log.Info(testName, "blob ", blob)

	blobRef, err := client.SubmitBlob(*blob)
	log.Info(testName, "blobRef ", blobRef)
	assert.NoError(t, err)
	assert.NotNil(t, blobRef)

	// Test getting the submitted blob
	blob, err = client.GetBlob(*blobRef)
	assert.NoError(t, err)

	log.Info("TestSubmitGetBlob blob ", blob)
	if !bytes.Equal(blob.Data, data) {
		t.Fatalf("expected blob data %v but got %v", data, blob.Data)
	}

	// Test submitting an empty blob
	emptyBlob := Blob{}
	blobRef, err = client.SubmitBlob(emptyBlob)
	log.Info("TestSubmitBlob emptyBlob ", emptyBlob)
	assert.Errorf(t, err, "blob data cannot be nil")
	assert.Nil(t, blobRef)
}

// func TestHealth(t *testing.T) {
// 	client := initClient(t)
// 	defer client.Close()

// 	// Test checking the health of the service
// 	err := client.Health()
// 	assert.NoError(t, err)
// }

// func TestBlobMarshalUnmarshal(t *testing.T) {
// 	data := []byte("test_data")
// 	blob := Blob{Data: data}

// 	// Test marshaling the blob
// 	jsonData, err := blob.MarshalJSON()
// 	assert.NoError(t, err)

// 	// Test unmarshaling the blob
// 	var unmarshaled Blob
// 	err = unmarshaled.UnmarshalJSON(jsonData)
// 	assert.NoError(t, err)

// 	if !bytes.Equal(unmarshaled.Data, data) {
// 		t.Fatalf("unmarshaled blob data does not match original data")
// 	}
// }

// func TestNewBlobRefInvalidTransactionID(t *testing.T) {
// 	invalidTransactionID := []byte("invalid_transaction_id")
// 	_, err := NewBlobRef(invalidTransactionID)
// 	assert.Error(t, err, "invalid transaction ID length")
// }

func generateTransactionID(t *testing.T) []byte {

	hex, err := hex.DecodeString("5d0472abe8eef76f9a44a3695d584af4de6e2ddde82dabfa5c8f29e5aec1270d")
	log.Info("generateTransactionID hex ", hex)
	assert.NoError(t, err)

	blobRef, err := NewBlobRef(hex)
	log.Info("generateTransactionID blobRef", blobRef)
	assert.NoError(t, err)

	return blobRef.transactionID[:]
}
