package sidecar

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"fmt"
	"os"
	"testing"

	log "github.com/sirupsen/logrus"
)

// TODO: setup the sidecar in tests
func initClient(t *testing.T) *Client {
	return InitLocalClient(t, "../../http-config.json")
}

func InitLocalClient(t *testing.T, path string) *Client {
	configData, err := os.ReadFile(path)

	log.Debug("initClient configData ", string(configData))
	if err != nil {
		log.Warn("failed to read config file, using default config: ", err)
	}

	// Unmarshal the JSON data into a ConfigureClientRequest struct
	var conf ConfigureClientRequest
	err = json.Unmarshal(configData, &conf)
	if err != nil {
		panic(fmt.Errorf("failed to unmarshal config: %v", err))
	}

	client, err := NewClient("http://localhost:5888", &conf)
	if err != nil {
		t.Fatalf("failed to create client: %v", err)
	}
	err = client.ConfigureClient(&conf)
	if err != nil {
		log.Warn("failed to configure client, likely already configured: ", err)
	}
	return client
}

func TestGetInvalidBlob(t *testing.T) {
	client := initClient(t)
	defer client.Close()

	invalidTransactionID := []byte("invalid_transaction_id")
	log.Info("TestGetBlob invalidTransactionID ", invalidTransactionID)
	invalidBlobRef := &BlobRef{}
	log.Info("TestGetBlob invalidBlobRef ", invalidBlobRef)
	copy(invalidBlobRef.transactionID[:], invalidTransactionID)
	blob, err := client.GetBlob(*invalidBlobRef)
	log.Info("TestGetBlob invalidBlob ", blob)
	if err == nil {
		t.Fatal("expected an error but got none")
	}
	if blob != nil {
		t.Fatalf("expected nil blob but got %v", blob)
	}
}

func TestSubmitGetBlob(t *testing.T) {
	client := initClient(t)
	defer client.Close()

	// Test submitting a blob
	data := []byte("test_data")
	blob := &Blob{Data: data}
	log.Info("TestSubmitBlob blob ", blob)
	blobRef, err := client.SubmitBlob(*blob)
	log.Info("TestSubmitBlob blobRef ", blobRef)
	if err != nil {
		t.Fatalf("failed to submit blob: %v", err)
	}
	if blobRef == nil {
		t.Fatal("got nil blob reference")
	}

	// Test getting the submitted blob
	blob, err = client.GetBlob(*blobRef)
	if err != nil {
		t.Fatalf("failed to get blob: %v", err)
	}
	log.Info("TestGetBlob blob ", blob)
	if !bytes.Equal(blob.Data, data) {
		t.Fatalf("expected blob data %v but got %v", data, blob.Data)
	}

	// Test submitting an empty blob
	emptyBlob := Blob{}
	blobRef, err = client.SubmitBlob(emptyBlob)
	log.Info("TestSubmitBlob emptyBlob ", emptyBlob)
	if err == nil {
		t.Fatal("expected an error but got none")
	}
	if blobRef != nil {
		t.Fatalf("expected nil blob reference but got %v", blobRef)
	}
}

func TestHealth(t *testing.T) {
	client := initClient(t)
	defer client.Close()

	// Test checking the health of the service
	err := client.Health()
	if err != nil {
		t.Fatalf("health check failed: %v", err)
	}
}

func TestBlobMarshalUnmarshal(t *testing.T) {
	data := []byte("test_data")
	blob := Blob{Data: data}

	// Test marshaling the blob
	jsonData, err := blob.MarshalJSON()
	if err != nil {
		t.Fatalf("failed to marshal blob: %v", err)
	}

	// Test unmarshaling the blob
	var unmarshaled Blob
	err = unmarshaled.UnmarshalJSON(jsonData)
	if err != nil {
		t.Fatalf("failed to unmarshal blob: %v", err)
	}

	if !bytes.Equal(unmarshaled.Data, data) {
		t.Fatalf("unmarshaled blob data does not match original data")
	}
}

func TestNewBlobRefInvalidTransactionID(t *testing.T) {
	invalidTransactionID := []byte("invalid_transaction_id")
	_, err := NewBlobRef(invalidTransactionID)
	if err == nil {
		t.Fatal("expected an error but got none")
	}
}

func generateTransactionID(t *testing.T) []byte {

	hex, err := hex.DecodeString("5d0472abe8eef76f9a44a3695d584af4de6e2ddde82dabfa5c8f29e5aec1270d")
	log.Info("generateTransactionID hex ", hex)
	if err != nil {
		t.Errorf("Failed to decode hex string: %v", err)
	}

	blobRef, err := NewBlobRef(hex)
	log.Info("generateTransactionID blobRef", blobRef)
	if err != nil {
		t.Fatalf("failed to create blob reference: %v", err)
	}
	return blobRef.transactionID[:]
}
