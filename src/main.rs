mod blockchain;

use blockchain::{Transaction, Blockchain, TxInfo, BlockInfo, Block, RequestOpitons};
use rocket::http::hyper::request;
use rocket::response::status::NotFound;
use rocket::serde::{Serialize, json ,json::{Json, serde_json::to_string}, Deserialize};

use rocket::{routes, Request};
use rocket::tokio::{task};
use uuid::Uuid;
use reqwest;
use rocket::futures::future::{try_join_all, TryJoinAll};


use rocket::serde::json::serde_json::{json, Value, from_str};

#[macro_use] extern crate rocket;

static mut new_blockchain: Blockchain = blockchain::Blockchain { chain: Vec::<blockchain::Block>::new(), pending_transaction: Vec::<Transaction>::new(), current_node_url: "" , network_nodes: Vec::<String>::new() };


#[get("/blockchain")]
fn get_blockchain() -> Json<&'static Blockchain> {
    unsafe {
        new_blockchain.create_new_block(1000, "ASDFASFAFAFDAF".to_string(), "ASDFAFASDFAFDFSADF".to_string());
        Json(&new_blockchain)
    }
}

#[post("/transaction", format = "json", data = "<transaction>")]
fn transaction(transaction: Json<Transaction>) -> Json<&'static Blockchain> {
    unsafe {
        let tx: Transaction = Transaction { amount: transaction.amount.clone(), sender: transaction.sender.clone(), recipient: transaction.recipient.clone(), transaction_id: transaction.transaction_id.clone() };
        new_blockchain.add_transactions_to_pending_transaction(tx);

        Json(&new_blockchain)
    }
}


#[get("/mine")]
async fn mine() -> Json<Value> {
    unsafe {
        let last_block = new_blockchain.get_last_block();
        let previous_blockhash = last_block.hash;
        let current_blockdata = BlockInfo {
            index: last_block.index.clone()+1,
            transactions: new_blockchain.pending_transaction.clone(),
        };

        let nonce = new_blockchain.proof_of_work(&previous_blockhash, &current_blockdata);
        let blockhash = new_blockchain.hash_block(&previous_blockhash, &current_blockdata, &nonce);
        let new_block = new_blockchain.create_new_block(nonce, previous_blockhash, blockhash);

        let client = reqwest::Client::new();

        for (i, network_node_url) in new_blockchain.network_nodes.iter().enumerate() {
            let request_options_block = RequestOpitons {
                uri: network_node_url.to_owned() + "/receive-new-block",
                method: "POST".to_owned(),
                body: new_block.clone(),
                json: true
            };

            let res = client.post(request_options_block.uri.clone())
                .json(&request_options_block)
                .send()
                .await.unwrap()
                .json::<Block>()
                .await;
        }

        let node_address: String = Uuid::new_v4().to_string();

        let request_options_transactions = RequestOpitons {
            uri: new_blockchain.current_node_url.to_owned() + "/transaction/broadcast",
            method: "POST".to_owned(),
            body: TxInfo {
                amount: 12,
                sender: "00".to_owned(),
                recipient: node_address
            },
            json: true
        };

        let res = client.post(request_options_transactions.uri.clone())
                .json(&request_options_transactions)
                .send()
                .await.unwrap()
                .json::<TxInfo>()
                .await;
        

        // new_blockchain.create_new_transaction(10, "00".to_string(), "recipient".to_string());
        
        // let new_block = new_blockchain.create_new_block(nonce, previous_blockhash, blockhash);
        
        Json(json!({
            "note": "New block mined & broadcast successfully",
            "block": new_block
        }))
    }
}

#[post("/transaction/broadcast", format = "json", data = "<data>")]
async fn transaction_broadcast(data: String) -> Json<&'static Blockchain> {
    unsafe {
        let tx_info: TxInfo = from_str(&data).unwrap();
        //println!("{}","5".to_string().parse::<u32>().unwrap());
        let new_transaction = new_blockchain.create_new_transaction(tx_info.amount, tx_info.sender, tx_info.recipient);
        new_blockchain.add_transactions_to_pending_transaction(new_transaction.clone());
        
        let mut request_promises = vec![];
        for (index, network_node_url) in new_blockchain.network_nodes.iter().enumerate() {
            let request_options_block = RequestOpitons {
                uri: network_node_url.to_string() + "/transaction",
                method: "POST".to_string(),
                body: new_transaction.clone(),
                json: true
            };

            request_promises.push(request_options_block);
        }


        transaction(Json(new_transaction));
        Json(&new_blockchain)
        
    }
}


#[post("/receive-new-block", format = "json", data = "<data>")]
fn receive_new_block(data: String) -> Json<Value> {
    unsafe {
        let new_block: Block = from_str(&data).unwrap();
        let last_block = new_blockchain.get_last_block();
        let correct_hash = last_block.hash == new_block.previous_blockhash;
        let correct_index = last_block.index + 1 == new_block.index;

        if correct_hash && correct_index {
            new_blockchain.chain.push(new_block.clone());
            new_blockchain.pending_transaction = vec![];
            Json(json!({
                "note": "New block received and accepted",
                "newBlock": new_block
            }))
        } else {
            Json(json!({
                "note": "New block rejected",
                "newBlock": new_block
            }))
        }
    }
}

#[post("/register-node", format = "json", data = "<data>")]
fn register_node(data: String) -> Json<Value> {
    unsafe {
        let new_node_url = data;
        let node_not_already_present = new_blockchain.network_nodes.contains(&new_node_url); 
        let not_current_node = new_blockchain.current_node_url != new_node_url;
        if node_not_already_present && not_current_node {
            new_blockchain.network_nodes.push(new_node_url);
        }
        Json(json!({
            "note": "New node registered succesfully."
        }))
    }
}

#[get("/consensus")]
fn consensus() -> Json<&'static Blockchain> {
    unsafe {
        new_blockchain.create_new_block(1000, "ASDFASFAFAFDAF".to_string(), "ASDFAFASDFAFDFSADF".to_string());
        Json(&new_blockchain)
    }
}



#[launch]
fn rocket() -> _ {
    rocket::build().mount("/hello",routes![get_blockchain,transaction,transaction_broadcast,mine])
}



