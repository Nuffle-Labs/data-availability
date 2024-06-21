// Package sidecar provides a client for interacting with the Near Protocol Sidecar service.
//
// The sidecar service is responsible for submitting and retrieving data blobs to and from the Near blockchain.
// It acts as an intermediary between the application and the Near blockchain, abstracting away the complexities
// of interacting with the blockchain directly.
//
// Security Considerations:
// - The sidecar service should be running on a trusted host and port.
// - The host and port should be configurable and not hardcoded.
// - The client should verify the identity of the sidecar service using TLS certificates.
// - The client should validate and sanitize all input parameters to prevent injection attacks.
// - The client should handle errors gracefully and not leak sensitive information in error messages.
// - The client should use secure communication channels (e.g., HTTPS) to prevent eavesdropping and tampering.
// - The client should have proper authentication and authorization mechanisms to prevent unauthorized access.
//
// Usage:
//
// 1. Create a new client instance using the `NewClient` function, providing the host and configuration.
//
//	client, err := sidecar.NewClient("http://localhost:5888", &sidecar.ConfigureClientRequest{...})
//	if err != nil {
//	    // Handle error
//	}
//
// 2. Use the client to interact with the sidecar service.
//
//	// Submit a blob
//	blob := sidecar.Blob{Data: []byte("test_data")}
//	blobRef, err := client.SubmitBlob(blob)
//	if err != nil {
//	    // Handle error
//	}
//
//	// Get a blob
//	retrievedBlob, err := client.GetBlob(*blobRef)
//	if err != nil {
//	    // Handle error
//	}
//
// 3. Close the client when done.
//
//	client.Close()
package sidecar

import (
	"bytes"
	"encoding/hex"
	"encoding/json"
	"errors"
	"fmt"
	"net/http"

	log "github.com/sirupsen/logrus"
)

// Client represents a client for interacting with the Near Protocol Sidecar service.
type Client struct {
	client *http.Client
	host   string
	config *ConfigureClientRequest
}

// NewClient creates a new instance of the Near Protocol Sidecar client.
// It takes the host and configuration as parameters and returns a pointer to the client.
// If the host is empty, it defaults to "http://localhost:5888".
// The configuration can be nil, assuming the sidecar is set up outside of this package.
func NewClient(host string, config *ConfigureClientRequest) (*Client, error) {
	if host == "" {
		host = "http://localhost:5888"
	}
	client := &Client{
		client: &http.Client{},
		host:   host,
		config: config,
	}
	return client, client.Health()
}

func (c *Client) GetHost() string {
	return c.host
}

// ConfigureClient configures the Near Protocol Sidecar client with the provided configuration.
// It sends a PUT request to the "/configure" endpoint with the configuration as JSON payload.
func (c *Client) ConfigureClient(req *ConfigureClientRequest) error {
	if req == nil {
		req = c.config
	}
	jsonData, err := json.Marshal(req)
	if err != nil {
		return fmt.Errorf("failed to marshal configure client request: %v", err)
	}

	httpReq, err := http.NewRequest(http.MethodPut, c.host+"/configure", bytes.NewBuffer(jsonData))
	if err != nil {
		return fmt.Errorf("failed to create configure client request: %v", err)
	}
	httpReq.Header.Set("Content-Type", "application/json")

	resp, err := c.client.Do(httpReq)
	if err != nil {
		return fmt.Errorf("failed to send configure client request: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("failed to configure client, status code: %d", resp.StatusCode)
	}

	return nil
}

// GetBlob retrieves a blob from the Near blockchain using the provided BlobRef.
// It sends a GET request to the "/blob" endpoint with the transaction ID as a query parameter.
func (c *Client) GetBlob(b BlobRef) (*Blob, error) {
	resp, err := c.client.Get(c.host + "/blob?transaction_id=" + b.ID())
	if err != nil {
		return nil, fmt.Errorf("failed to send get blob request: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("failed to get blob, status code: %d", resp.StatusCode)
	}

	var blob Blob
	err = json.NewDecoder(resp.Body).Decode(&blob)
	if err != nil {
		return nil, fmt.Errorf("failed to decode blob response: %v", err)
	}

	return &blob, nil
}

// SubmitBlob submits a blob to the Near blockchain.
// It sends a POST request to the "/blob" endpoint with the blob data as JSON payload.
// The response contains the transaction ID of the submitted blob.
func (c *Client) SubmitBlob(b Blob) (*BlobRef, error) {
	if b.Data == nil {
		return nil, errors.New("blob data cannot be nil")
	}

	jsonData, err := b.MarshalJSON()
	if err != nil {
		return nil, fmt.Errorf("failed to marshal blob: %v", err)
	}
	log.Debug("near-sidecar: SubmitBlob json: ", jsonData)

	resp, err := c.client.Post(c.host+"/blob", "application/json", bytes.NewBuffer(jsonData))
	if err != nil {
		return nil, fmt.Errorf("failed to send submit blob request: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("failed to submit blob, status code: %d", resp.StatusCode)
	}

	var blobRef BlobRef
	err = json.NewDecoder(resp.Body).Decode(&blobRef)
	if err != nil {
		return nil, fmt.Errorf("failed to decode transaction ID: %v", err)
	}

	return &blobRef, nil
}

// Health checks the health of the Near Protocol Sidecar service.
// It sends a GET request to the "/health" endpoint and expects a successful response.
func (c *Client) Health() error {
	resp, err := c.client.Get(c.host + "/health")
	if err != nil {
		return fmt.Errorf("failed to send health check request: %v", err)
	}
	defer resp.Body.Close()

	if resp.StatusCode != http.StatusOK {
		return fmt.Errorf("health check failed, status code: %d", resp.StatusCode)
	}

	return nil
}

// Close closes the Near Protocol Sidecar client.
// It should be called when the client is no longer needed.
func (c *Client) Close() {
	// Perform any necessary cleanup or resource release
}

// BlobRef represents a reference to a blob on the Near blockchain.
type BlobRef struct {
	transactionID [EncodedBlobRefSize]byte
}

// EncodedBlobRefSize is the size of an encoded BlobRef in bytes.
const EncodedBlobRefSize = 32

// NewBlobRef creates a new BlobRef from the provided transaction ID.
// It returns an error if the transaction ID is not exactly 32 bytes.
func NewBlobRef(transactionID []byte) (*BlobRef, error) {
	if len(transactionID) != EncodedBlobRefSize {
		return nil, errors.New("invalid transaction ID length")
	}
	var ref BlobRef
	copy(ref.transactionID[:], transactionID)
	return &ref, nil
}

// Deref returns the transaction ID of the BlobRef.
func (r *BlobRef) Deref() []byte {
	return r.transactionID[:]
}

// ID returns the transaction ID of the BlobRef as a hex-encoded string.
func (r *BlobRef) ID() string {
	return hex.EncodeToString(r.transactionID[:])
}

// MarshalJSON marshals the BlobRef to JSON format.
// It encodes the transaction ID as a hex string.
func (r *BlobRef) MarshalJSON() ([]byte, error) {
	json, err := json.Marshal(struct {
		TransactionID string `json:"transaction_id"`
	}{
		TransactionID: r.ID(),
	})
	return json, err
}

// UnmarshalJSON unmarshals the BlobRef from JSON format.
// It decodes the transaction ID from a hex string.
func (r *BlobRef) UnmarshalJSON(data []byte) error {
	var aux struct {
		TransactionID string `json:"transaction_id"`
	}
	if err := json.Unmarshal(data, &aux); err != nil {
		return err
	}
	transactionID, err := hex.DecodeString(aux.TransactionID)
	if err != nil {
		return err
	}
	copy(r.transactionID[:], transactionID)
	return nil
}

// Blob represents a blob of data stored on the Near blockchain.
type Blob struct {
	Data []byte `json:"data"`
}

// MarshalJSON marshals the Blob to JSON format.
// It encodes the data as a hex string.
func (b *Blob) MarshalJSON() ([]byte, error) {
	return json.Marshal(struct {
		Data string `json:"data"`
	}{
		Data: hex.EncodeToString(b.Data),
	})
}

// UnmarshalJSON unmarshals the Blob from JSON format.
// It decodes the data from a hex string.
func (b *Blob) UnmarshalJSON(data []byte) error {
	var aux struct {
		Data string `json:"data"`
	}
	if err := json.Unmarshal(data, &aux); err != nil {
		return err
	}
	decodedData, err := hex.DecodeString(aux.Data)
	if err != nil {
		return err
	}
	b.Data = decodedData
	return nil
}

// Network represents a Near network.
type Network string

const (
	Mainnet  Network = "mainnet"
	Testnet  Network = "testnet"
	Localnet Network = "localnet"
)

// ConfigureClientRequest represents a request to configure the Near Protocol Sidecar client.
type ConfigureClientRequest struct {
	AccountID  string     `json:"account_id"`
	SecretKey  string     `json:"secret_key"`
	ContractID string     `json:"contract_id"`
	Network    Network    `json:"network"`
	Namespace  *Namespace `json:"namespace"`
}

// Namespace represents a namespace on the Near blockchain.
type Namespace struct {
	ID      int `json:"id"`
	Version int `json:"version"`
}
