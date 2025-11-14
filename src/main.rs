mod transaction;
mod account;

use crate::account::Account;
use crate::transaction::{Transaction, TransactionType};
use std::collections::HashMap;

fn main() {
    let filename = std::env::args_os().nth(1).expect("missing file argument");

    let reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All) // trim all whitespace
        .has_headers(true) // file has a header row
        .from_path(filename);

    let mut reader = match reader {
        Ok(reader) => reader,
        Err(error) => {
            // in the "thousands of concurrent connections" scenario
            // this should not panic, but instead write an error to a logfile
            panic!("error reading file: {}", error)
        }
    };

    // TODO replace this with a DB table
    let mut accounts: HashMap<u16, Account> = HashMap::new();

    // TODO replace this with a DB table
    let mut processed_transactions: HashMap<u32, Transaction> = HashMap::new();

    for result in reader.deserialize() {
        let tx: Transaction = result.expect("could not parse line");
        match accounts.get_mut(&tx.client) {
            None => {
                // if there is no account, but we are trying to apply a transaction to it...
                // handle each transaction appropriately
                match tx.kind {
                    TransactionType::Deposit => {
                        // TODO in a DB scenario, everything here must be a single transaction
                        let account = Account {
                            id: tx.client,
                            available: tx.amount,
                            held: 0.0,
                            locked: false,
                        };
                        accounts.insert(tx.client, account);
                        processed_transactions.insert(tx.tx, tx);
                    }
                    TransactionType::Withdrawal => {
                        // TODO this should fail "nicely"
                        processed_transactions.insert(tx.tx, tx);
                        // NOTE: it's _possible_ in a distributed / multithreaded environment, that
                        // this is an out-of-order scenario. Handle this if / when appropriate.
                        eprintln!("cannot withdraw from a new account");
                    }
                    TransactionType::Dispute => {
                        // TODO this should fail "nicely"
                        processed_transactions.insert(tx.tx, tx);

                        // NOTE: it's _possible_ in a distributed / multithreaded environment, that
                        // this is an out-of-order scenario. Handle this if / when appropriate.
                        eprintln!("cannot dispute a transaction for a new account");
                    }
                    TransactionType::Resolve => {
                        // TODO this should fail "nicely"
                        processed_transactions.insert(tx.tx, tx);

                        // NOTE: it's _possible_ in a distributed / multithreaded environment, that
                        // this is an out-of-order scenario. Handle this if / when appropriate.
                        eprintln!("cannot resolve a disputed transaction for a new account");
                    }
                    TransactionType::Chargeback => {
                        // TODO this should fail "nicely"
                        processed_transactions.insert(tx.tx, tx);

                        // NOTE: it's _possible_ in a distributed / multithreaded environment, that
                        // this is an out-of-order scenario. Handle this if / when appropriate.
                        eprintln!("cannot chargeback a disputed transaction for a new account");
                    }
                }
            }
            Some(account) => {
                if let Err(message) = account.apply(&tx, &processed_transactions) {
                    eprintln!("{}", message)
                }
                processed_transactions.insert(tx.tx, tx);
            }
        }
    }

    println!("client,available,held,total,locked");
    for account in accounts.values() {
        println!("{},{},{},{},{}",
                 account.id,
                 account.available,
                 account.held,
                 account.total(),
                 account.locked,
        )
    }
}