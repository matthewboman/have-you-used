use std::collections::HashSet;
use super::*;

pub struct Blockchain {
    pub blocks: Vec<Block>,
    unspent_outputs: HashSet<Hash>,
}

impl Blockchain {
    pub fn new() -> Blockchain {
        Blockchain {
            blocks: vec![],
            unspent_outputs: HashSet::new()
        }
    }

    pub fn update_with_block(&mut self, block: Block) -> Result<(), BlockValidationError> {
        let i = self.blocks.len();


        if block.index != i as u32 {
            return Err(BlockValidationError::MismatchedIndex);
        }

        if !block::check_difficulty(&block.hash(), block.difficulty) {
            return Err(BlockValidationError::InvalidHash);
        }

        // Genesis block won't have previous block
        if i == 0 {
            if block.prev_block_hash != vec![0; 32] {
                return Err(BlockValidationError::InvalidGenesisBlockFormat);
            }
        // Not genesis block
        } else {
            let prev_block = &self.blocks[i - 1];

            if block.timestamp <= prev_block.timestamp {
                return Err(BlockValidationError::AcrhonologicalTimestamp);
            }
            if block.prev_block_hash != prev_block.hash {
                return Err(BlockValidationError::MismatchedPreviousHash);
            }
        }

        if let Some((coinbase, transactions)) = block.transactions.split_first() {
            if !coinbase.is_coinbase() {
                return Err(BlockValidationError::InvalidCoinbaseTransaction);
            }

            let mut block_spent: HashSet<Hash> = HashSet::new();
            let mut block_created: HashSet<Hash> = HashSet::new();
            let mut total_fee = 0;

            for transaction in transactions {
                let input_hashes = transaction.input_hashes();

                if !(&input_hashes - &self.unspent_outputs).is_empty() {
                    return Err(BlockValidationError::InvalidInput);
                }

                if !(&input_hashes & &block_spent).is_empty() {
                    return Err(BlockValidationError::InvalidInput);
                }

                let input_value = transaction.input_value();
                let output_value = transaction.output_value();

                if output_value > input_value {
                    return Err(BlockValidationError::InsufficientInputValue);
                }

                // calculate the fee
                let fee = input_value - output_value;
                total_fee += fee;

                block_spent.extend(input_hashes);
                block_created.extend(transaction.output_hashes());
            }

            if coinbase.output_value() < total_fee {
                // TODO: more validation
                return Err(BlockValidationError::InvalidCoinbaseTransaction);
            } else {
                block_created.extend(coinbase.output_hashes());
            }

            // update unspent output's pool
            self.unspent_outputs.retain(|output| !block_spent.contains(output));
            self.unspent_outputs.extend(block_created);
        }

        // add transaction to blockchain
        self.blocks.push(block);

        Ok(())
    }
}

#[derive(Debug)]
pub enum BlockValidationError {
    MismatchedIndex,
    InvalidHash,
    AcrhonologicalTimestamp,
    MismatchedPreviousHash,
    InvalidGenesisBlockFormat,
    InvalidInput,
    InsufficientInputValue,
    InvalidCoinbaseTransaction,
}
