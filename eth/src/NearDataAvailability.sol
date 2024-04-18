// SPDX-License-Identifier: AGPL-3.0
pragma solidity >=0.8.25;

import { IDataAvailabilityProtocol } from "@polygon/zkevm-contracts/interfaces/IDataAvailabilityProtocol.sol";
import { OwnableRoles } from "@solady/auth/OwnableRoles.sol";
// import { Initializable } from "@solady/utils/Initializable.sol";

/**
 * @dev Struct to store the data availability batch, transaction verification on ethereum and transaction submission on
 * NEAR
 *
 */
struct VerifiedBatch {
    bytes32 id;
    bytes32 verifyTxHash;
    bytes32 submitTxId;
}

/*
 * Contract responsible for storing the lookup information for the status of each NEARDA batch
 * It is heavily modeled after the requirements from polygon CDK
 */
contract NearDataAvailability is IDataAvailabilityProtocol, OwnableRoles {
    // Name of the data availability protocol
    string internal constant _PROTOCOL_NAME = "NearProtocol";

    // The amount of batches that we track is available
    // note, they are still available via archival and indexers, just not actively tracked
    // in the contract.
    uint256 public constant _STORED_BATCH_AMT = 32;

    // The number of transactions we actively track awaiting verification
    // this is useful for users who want some immediate notfification on-chain.
    uint256 public constant _SUBMITTED_BATCH_AMT = 128;

    // The role allows a client to notify the contract that a batch has been submitted for
    // verification.
    uint256 public constant _NOTIFIER = _ROLE_10;

    // The role enables providing verified batches to the contract, this would normally
    // be the light client.
    uint256 public constant _VERIFIER = _ROLE_11;

    // @dev The batches that have been made available, keyed by bucket id
    // @notice this dusts the earliest batch
    VerifiedBatch[_STORED_BATCH_AMT] public batchInfo;
    uint256 private _verifyBucketIdx;

    /**
     * @dev Batches that have been submitted and are awaiting proofs
     * @notice this dusts the earliest batch
     * @notice this is very inefficient, we are going to modify the way the light client proves batches
     * to better utilise the generators in the light client
     */
    bytes32[_SUBMITTED_BATCH_AMT] public submittedBatches;
    uint256 private _submitBucketIdx;

    /**
     * @dev Emitted when the DA batch is made available, used to determine if the batch has been proven
     * @param batch Batch of data that has been verified
     * @param bucketIdx current index of the batch in the store
     */
    event IsAvailable(uint256 bucketIdx, VerifiedBatch batch);

    /**
     * @dev Emitted when the batch has been submitted for verification
     * @param bucketIdx current index of the tx in the store
     * @param submitTxId transaction id of the submission on NEAR
     */
    event Submitted(uint256 bucketIdx, bytes32 submitTxId);

    /// @custom:oz-upgrades-unsafe-allow constructor
    constructor() {
        // _disableInitializers();
        _initializeOwner(msg.sender);
    }

    // function initialize(address initialOwner) public initializer {
    // }

    /**
     * @notice Verifies that the given signedHash has been signed by requiredAmountOfSignatures committee members
     * @param dataAvailabilityBatch blarg
     *
     */
    function verifyMessage(bytes32, /*hash*/ bytes calldata dataAvailabilityBatch) external view {
        VerifiedBatch storage item;
        // TODO: will fail decoding since not chunked
        bytes32 batchId = abi.decode(dataAvailabilityBatch, (bytes32));
        for (uint256 i = 0; i < batchInfo.length; i++) {
            item = batchInfo[i];
            if (item.id == batchId) {
                return;
            }
        }
        // TODO: when optimise storage layout for NEAR LC, we reenable checking
        // ifsubmitted && sender is sequencer, return;
        revert();
    }

    function notifySubmitted(bytes calldata batches) external onlyRolesOrOwner(_NOTIFIER) {
        // chunk the batches into blobRefSizes
        uint256 blobRefSize = 32;
        uint256 numBatches = batches.length / blobRefSize;
        for (uint256 i = 0; i < numBatches; i++) {
            bytes32 txId = abi.decode(batches[i * blobRefSize:(i + 1) * blobRefSize], (bytes32));
            uint256 bucketIdx = setSubmitted(txId);
            emit Submitted(bucketIdx, txId);
        }
    }

    function setSubmitted(bytes32 txId) private returns (uint256) {
        uint256 bucket = _submitBucketIdx;
        submittedBatches[bucket] = txId;

        // TODO[Optimisation]: replace with bitmask & assembly
        _submitBucketIdx = (bucket + 1) % _SUBMITTED_BATCH_AMT;
        return bucket;
    }

    function notifyAvailable(VerifiedBatch memory verifiedBatch) external onlyRolesOrOwner(_VERIFIER) {
        uint256 bucketIdx = setBatch(verifiedBatch);
        emit IsAvailable(bucketIdx, verifiedBatch);
    }

    function setBatch(VerifiedBatch memory verifiedBatch) private returns (uint256) {
        uint256 bucket = _verifyBucketIdx;
        batchInfo[bucket] = verifiedBatch;

        // TODO[Optimisation]: replace with bitmask & assembly
        _verifyBucketIdx = (bucket + 1) % _STORED_BATCH_AMT;
        return bucket;
    }

    /**
     * @notice Return the protocol name
     */
    function getProcotolName() external pure override returns (string memory) {
        return _PROTOCOL_NAME;
    }
}
