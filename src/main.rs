use blockchain::{Transaction, Blockchain};
use rocket::response::status::NotFound;
use rocket::routes;
use uuid::Uuid;
use rocket::serde::json::serde_json::{json, Value, from_str};
use rocket::serde::{json::Json};
mod blockchain;
#[macro_use] extern crate rocket;

static mut new_blockchain: Blockchain = blockchain::Blockchain { chain: Vec::<blockchain::Block>::new(), pending_transaction: Vec::<Transaction>::new()};

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

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/hello",routes![get_blockchain,transaction])
}

