// SPDX-License-Identifier: UNLICENSED
pragma solidity >=0.8.25 <0.9.0;

import { PRBTest } from "@prb/test/src/PRBTest.sol";
import { StdCheats } from "forge-std/src/StdCheats.sol";

import { NearDataAvailability, VerifiedBatch } from "../src/NearDataAvailability.sol";

contract NearDataAvailabilityTest is PRBTest, StdCheats {
    NearDataAvailability public nearDataAvailability;
    address public owner;
    address public nonOwner;

    /// @dev A function invoked before each test case is run.
    function setUp() public virtual {
        // Instantiate the contract-under-test.
        nearDataAvailability = new NearDataAvailability();
        owner = address(this);
        nonOwner = address(0x1);
        nearDataAvailability.switchBypass();
        // nearDataAvailability.initialize(owner);
    }

    function testVerifyMessageExistingBatch() public {
        VerifiedBatch memory batch = VerifiedBatch(bytes32(uint256(1)), bytes32(uint256(2)), bytes32(uint256(3)));
        nearDataAvailability.notifyAvailable(batch);

        bytes memory encodedBatch = abi.encode(batch.id);
        nearDataAvailability.verifyMessage(bytes32(0), encodedBatch);
    }

    function testVerifyMessageNonExistingBatch() public {
        VerifiedBatch memory batch = VerifiedBatch(bytes32(uint256(1)), bytes32(uint256(2)), bytes32(uint256(3)));

        bytes memory encodedBatch = abi.encode(batch.id);
        vm.expectRevert();
        nearDataAvailability.verifyMessage(bytes32(0), encodedBatch);
    }

    function testNotifyAvailable() public {
        VerifiedBatch memory batch = VerifiedBatch(bytes32(uint256(1)), bytes32(uint256(2)), bytes32(uint256(3)));
        vm.expectEmit(true, true, true, true);
        emit NearDataAvailability.IsAvailable(0, batch);
        nearDataAvailability.notifyAvailable(batch);
    }

    function testNotifyAvailableOverwritesBatch() public {
        VerifiedBatch memory batch1 = VerifiedBatch(bytes32(uint256(1)), bytes32(uint256(2)), bytes32(uint256(3)));
        nearDataAvailability.notifyAvailable(batch1);

        vm.expectEmit(true, true, true, true);
        VerifiedBatch memory batch2 = VerifiedBatch(bytes32(uint256(4)), bytes32(uint256(5)), bytes32(uint256(6)));
        emit NearDataAvailability.IsAvailable(1, batch2);
        nearDataAvailability.notifyAvailable(batch2);
    }

    function testGetProcotolName() public {
        assertEq(nearDataAvailability.getProcotolName(), "NearProtocol");
    }

    function testNotifyAvailableLIFO() public {
        uint256 numBatches = 100;

        VerifiedBatch[] memory batches = new VerifiedBatch[](numBatches);

        for (uint256 i = 0; i < numBatches; i++) {
            VerifiedBatch memory batch =
                VerifiedBatch(bytes32(uint256(i + 1)), bytes32(uint256(i + 2)), bytes32(uint256(i + 3)));
            batches[i] = batch;
            vm.expectEmit(true, true, true, true);
            emit NearDataAvailability.IsAvailable((i % nearDataAvailability._STORED_BATCH_AMT()), batch);
            nearDataAvailability.notifyAvailable(batches[i]);
        }
    }
}
