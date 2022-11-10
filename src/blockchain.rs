use std::{time::{SystemTime, UNIX_EPOCH}, thread::current};
use rocket::serde::{Serialize, json ,json::{Json, serde_json::to_string}, Deserialize};
use uuid::Uuid;
use std::fmt;
use sha2::{Sha256, Digest, digest::generic_array::GenericArray};
use std::io;
use rocket::futures::future;

#[derive(Serialize)]
#[serde(crate = "rocket::serde")]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transaction: Vec<Transaction>,
    pub current_node_url : &'static str,
    pub network_nodes : Vec<String>
}

#[derive(Serialize,Clone,Deserialize,Debug)]
#[serde(crate = "rocket::serde")]
pub struct Block {
    pub index: usize,
    pub timestamp: u128,
    pub transactions: Vec<Transaction>,
    pub nonce: u128,
    pub hash: String,
    pub previous_blockhash: String
}


impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{} {} {:?} {} {} {}",self.index,self.timestamp,self.transactions,self.nonce,self.hash,self.previous_blockhash);
        Ok(())
    }
}

#[derive(Deserialize,Serialize,Clone,Debug)]
#[serde(crate = "rocket::serde")]
pub struct Transaction {
    pub amount: u128,
    pub sender: String,
    pub recipient: String,
    pub transaction_id: String
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{} {} {} {}",self.amount, self.sender, self.recipient, self.transaction_id);
        Ok(())
    }
}

#[derive(Serialize,Clone,Deserialize,Debug)]
#[serde(crate = "rocket::serde")]
pub struct TxInfo {
    pub amount: u128,
    pub sender: String,
    pub recipient: String
}

#[derive(Serialize,Clone,Deserialize,Debug)]
#[serde(crate = "rocket::serde")]
pub struct BlockInfo {
    pub index: usize,
    pub transactions: Vec<Transaction>,
}

impl fmt::Display for BlockInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{} {:?}",self.index, self.transactions);
        Ok(())
    }
}

#[derive(Serialize,Clone,Deserialize,Debug)]
#[serde(crate = "rocket::serde")]
pub struct RequestOpitons<T> {
    pub uri: String,
    pub method: String,
    pub body: T,
    pub json: bool
}



impl Blockchain {
    pub fn create_new_block(&mut self, nonce: u128, previous_blockhash: String, hash: String) -> Block {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        
        let new_block = Block { 
            index: self.chain.len() + 1, 
            timestamp: time, 
            transactions: self.pending_transaction.clone(), 
            nonce: nonce, 
            hash: hash, 
            previous_blockhash: previous_blockhash
        };
        
        self.pending_transaction = Vec::<Transaction>::new();
        self.chain.push(new_block);

        self.chain[self.chain.len()-1].clone()
    }

    pub fn get_last_block(&self) -> Block {
        self.chain[self.chain.len()-1].clone()
    }

    pub fn create_new_transaction(&self, amount: u128, sender: String, recipient: String) -> Transaction {
        let new_transaction = Transaction {
            amount: amount,
            sender: sender,
            recipient: recipient,
            transaction_id: Uuid::new_v4().to_string()
        };

        new_transaction
    }

    pub fn add_transactions_to_pending_transaction(&mut self, transaction_obj : Transaction) -> usize {
        self.pending_transaction.push(transaction_obj);
        self.get_last_block().index + 1
    }

    pub fn hash_block(&self, previous_blockhash: &String, current_block_data: &BlockInfo, nonce: &u128) -> String {
        let data_as_string = previous_blockhash.clone() + &nonce.to_string() + &current_block_data.to_string();
        let mut hasher = Sha256::new();
        hasher.update(data_as_string);
        format!("{:X}", hasher.finalize())
    }

    pub fn proof_of_work(&self, previous_blockhash: &String , current_block_data: &BlockInfo) -> u128 {
        let mut nonce = 0;
        let mut hash = self.hash_block(previous_blockhash, &current_block_data, &nonce);
        while hash[..4] != "0000".to_string() {
            nonce += 1;
            hash = self.hash_block(previous_blockhash, &current_block_data, &nonce);
        }

        nonce
    }

    pub fn chain_is_valid(&self, blockchain : &Vec<Block>) -> bool {
        let mut valid_chain = true;
        

        for i in 1..blockchain.len(){
            let current_block = blockchain[i].clone();
            let current_blockdata = BlockInfo {
                index: current_block.index.clone(),
                transactions: current_block.transactions.clone()
            };
            let prev_block = blockchain[i-1].clone();
            let block_hash = self.hash_block(&prev_block.hash, &current_blockdata, &current_block.nonce);
            if block_hash[..4] != "0000".to_string() { valid_chain = false };
            if current_block.previous_blockhash != prev_block.hash { valid_chain = false }; 
        };

        let genesis_block = &blockchain[0];
        let correct_nonce = genesis_block.nonce == 100; 
        let correct_previouse_block_hash = genesis_block.previous_blockhash == "0";
        let correct_hash = genesis_block.hash == "0";
        let correct_transaction = genesis_block.transactions.len() == 0;

        if !correct_nonce || !correct_previouse_block_hash || !correct_hash || !correct_transaction {
            valid_chain = false
        };

        valid_chain
    }

    pub fn get_block(&self, block_hash : String) -> Result<&Block, String> {
        for block in self.chain.iter() {
            if block.hash == block_hash {
                let correct_block = block;
                return Ok(correct_block) 
            } 
        };
        return Err("Error".to_string())
    }

    pub fn get_transaction(&self, transaction_id : String) -> Result<(&Transaction, &Block), String> {
        for block  in self.chain.iter() {
            for transaction in block.transactions.iter() {
                let mut correct_transaction;
                let mut correct_block;

                if transaction.transaction_id == transaction_id {
                    correct_transaction = transaction;
                    correct_block = block;
                    return Ok((correct_transaction, correct_block))
                }
            }
        };

        return Err("Error".to_string())
    }

    pub fn get_address_data(&self, address : String) -> (Vec<&Transaction>, u128) {
        let mut address_transactions = vec![];
        for block in self.chain.iter() {
            for transaction in block.transactions.iter() {
                if transaction.sender == address || transaction.recipient == address {
                    address_transactions.push(transaction);
                }
            }
        }

        let mut balance = 0;
        for transaction in address_transactions.iter() {
            if transaction.recipient == address {
                balance += transaction.amount;
                balance -= transaction.amount;
            }
        }

        (address_transactions, balance)
    }
}
