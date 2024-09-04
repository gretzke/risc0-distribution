// Copyright 2024 RISC Zero, Inc.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

pragma solidity ^0.8.20;

import {RiscZeroCheats} from "risc0/test/RiscZeroCheats.sol";
import {console2} from "forge-std/console2.sol";
import {Test} from "forge-std/Test.sol";
import {IRiscZeroVerifier} from "risc0/IRiscZeroVerifier.sol";
import {RewardDistribution} from "../contracts/RewardDistribution.sol";
import {Elf} from "./Elf.sol"; // auto-generated contract after running `cargo build`.

struct Leaf {
    address account;
    uint256 earned;
}

contract RewardDistributionTest is RiscZeroCheats, Test {
    RewardDistribution public rewardDistribution;

    function setUp() public {
        IRiscZeroVerifier verifier = deployRiscZeroVerifier();
        rewardDistribution =
            new RewardDistribution(verifier, 0x855a7c002948bb381cca3c500d252862d922ff2d7919ae419e2c46df42db6a54);
    }

    function test_SetNewRoot() public {
        uint256 reward = 1000;
        bytes32 oldRoot = 0x855a7c002948bb381cca3c500d252862d922ff2d7919ae419e2c46df42db6a54;
        bytes32 newRoot = 0xebeb411fa248bdaf034ea0687a4429d209a99268bb59021abe3e0938cf1f9fb1;
        address[] memory attesters = new address[](2);
        attesters[0] = 0x4444444444444444444444444444444444444444;
        attesters[1] = 0x5555555555555555555555555555555555555555;
        Leaf[] memory leaves = new Leaf[](2);
        leaves[0] = Leaf(0x6666666666666666666666666666666666666666, 4660);
        leaves[1] = Leaf(0x7777777777777777777777777777777777777777, 22136);

        (, bytes memory seal) =
            prove(Elf.REWARD_DISTRIBUTION_PATH, abi.encode(reward, oldRoot, newRoot, attesters, leaves));

        vm.deal(address(this), reward);
        rewardDistribution.setRoot{value: reward}(newRoot, seal);
        assertEq(rewardDistribution.root(), newRoot);
    }
}
