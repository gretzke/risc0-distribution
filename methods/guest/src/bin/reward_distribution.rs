//! Example showing how to use the `sol!` macro to generate Rust bindings for Solidity structs and
//! enums.
//!
use std::collections::HashMap;
use std::{io::Read, ops::Div, usize};

use alloy_primitives::{/*hex,*/ keccak256, Address, B256, U256};
use alloy_sol_types::sol;

use risc0_zkvm::guest::env;
// use std::env::args;

// Generates Rust bindings for Solidity structs, enums and type aliases.
sol! {
    #[allow(missing_docs)]
    #[derive(Debug)]
    struct Leaf {
        address account;
        uint256 earned;
    }
}

fn main() {
    // // Read the input data from command-line arguments
    // let args: Vec<String> = args().collect();
    // if args.len() < 2 {
    //     println!("Please provide a hexadecimal input as an argument.");
    //     return;
    // }
    // let input_str = args[1].trim_start_matches("0x");
    // let input_bytes = hex::decode(input_str).expect("Failed to decode hex input");

    // Read the input data for this application.
    let mut input_bytes = Vec::<u8>::new();
    env::stdin().read_to_end(&mut input_bytes).unwrap();

    // Decode and parse the input
    let reward = U256::from_be_slice(&input_bytes[0..32]);
    let old_root = B256::from_slice(&input_bytes[32..64]);
    let new_root = B256::from_slice(&input_bytes[64..96]);
    let attesters_offset = (U256::from_be_slice(&input_bytes[96..128])).to::<usize>();
    let leaf_offset = (U256::from_be_slice(&input_bytes[128..160])).to::<usize>();
    let attestation_count =
        U256::from_be_slice(&input_bytes[attesters_offset..attesters_offset + 32]);
    let leaf_count = U256::from_be_slice(&input_bytes[leaf_offset..leaf_offset + 32]);

    let mut attesters: Vec<Address> = Vec::new();
    let attester_map: HashMap<Address, bool> = HashMap::new();
    for i in 1..attestation_count.to::<usize>() + 1 {
        let address = Address::from_slice(
            &input_bytes[(attesters_offset + 12 + i * 32)..(attesters_offset + (i + 1) * 32)],
        );
        attesters.push(address);
    }

    let mut leaves: Vec<Leaf> = Vec::new();
    let mut leaf_map: HashMap<Address, bool> = HashMap::new();
    for i in 0..leaf_count.to::<usize>() {
        let start = leaf_offset + i * 32 * 2 + 32;
        let attester = Address::from_slice(&input_bytes[start + 12..start + 32]);
        let earned = U256::from_be_slice(&input_bytes[start + 32..start + 64]);
        leaf_map.insert(attester, true);
        leaves.push(Leaf {
            account: attester,
            earned: earned,
        });
    }

    // construct and verify old root
    let expected_old_root = merkle_tree_root(leaves.clone());
    assert_eq!(expected_old_root, old_root);

    let reward_per_attester = reward.div(U256::from(attesters.len()));

    let mut new_leaves: Vec<Leaf> = Vec::new();
    // iterate over attesters and add to leaves
    for leaf in leaves.clone() {
        let mut reward = leaf.earned;
        if attester_map.contains_key(&leaf.account) {
            reward = reward + reward_per_attester;
        }
        new_leaves.push(Leaf {
            account: leaf.account,
            earned: reward,
        });
    }

    for attester in attesters.clone() {
        if !leaf_map.contains_key(&attester) {
            new_leaves.push(Leaf {
                account: attester,
                earned: reward_per_attester,
            });
        }
    }

    // construct and verify new root
    let expected_new_root = merkle_tree_root(new_leaves);
    assert_eq!(expected_new_root, new_root);

    // Commit the journal that will be received by the application contract.
    // Journal is encoded using Solidity ABI for easy decoding in the app contract.
    // concat old root and new root
    let mut concat_data = Vec::new();
    concat_data.extend_from_slice(old_root.as_slice());
    concat_data.extend_from_slice(new_root.as_slice());
    concat_data.extend_from_slice(&reward.to_be_bytes_vec());
    env::commit_slice(concat_data.as_slice());
}

fn merkle_tree_root(leaves: Vec<Leaf>) -> B256 {
    if leaves.len() == 0 {
        return B256::ZERO;
    }
    let mut leaf_hashes: Vec<B256> = Vec::new();
    for leaf in leaves {
        let mut concat_data = Vec::new();
        concat_data.extend_from_slice(leaf.account.as_slice());
        concat_data.extend_from_slice(&leaf.earned.to_be_bytes_vec());
        leaf_hashes.push(keccak256(&concat_data));
    }
    let mut count = leaf_hashes.len();
    while count > 1 {
        let mut new_level = Vec::new();
        if count % 2 == 1 {
            new_level.push(leaf_hashes[count - 1]);
        }
        for i in 0..count / 2 {
            let mut concat_data = Vec::new();
            concat_data.extend_from_slice(leaf_hashes[i * 2].as_slice());
            concat_data.extend_from_slice(leaf_hashes[i * 2 + 1].as_slice());
            new_level.push(keccak256(&concat_data));
        }
        leaf_hashes = new_level;
        count = leaf_hashes.len();
    }
    leaf_hashes[0]
}
