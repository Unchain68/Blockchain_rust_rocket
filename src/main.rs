mod blockchain;
use std::str::from_boxed_utf8_unchecked;

use blockchain::{Transaction, Blockchain, TxInfo, Block};
use rocket::response::status::NotFound;
use rocket::{routes, Request};
use rocket::tokio::{task};
use uuid::Uuid;
use rocket::serde::json::serde_json::{json, Value, from_str};
use rocket::serde::{json::Json};
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
        new_blockchain.address_transactions_to_pending_transaction(tx);

        Json(&new_blockchain)
    }
}

#[post("/transaction/broadcast", format = "json", data = "<data>")]
async fn transaction_broadcast(data: String) -> Json<&'static Blockchain> {
    unsafe {
        let tx_info: TxInfo = from_str(&data).unwrap();
        //println!("{}","5".to_string().parse::<u32>().unwrap());
        let new_transaction = new_blockchain.create_new_transaction(tx_info.amount, tx_info.sender, tx_info.recipient);
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

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/hello",routes![get_blockchain,transaction,transaction_broadcast,receive_new_block])
}

