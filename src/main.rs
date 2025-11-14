mod transaction;
mod account;

use crate::account::Account;
use crate::transaction::{Transaction, TransactionType};
use std::collections::HashMap;

fn main() {
    let filename = std::env::args_os().nth(1).expect("missing file argument");

    let reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All) // trim all whitespace
        .has_headers(true) // we are told that the file has a header row
        .from_path(filename);

    // "Note that the CSV reader is buffered automatically"
    // https://docs.rs/csv/latest/csv/struct.ReaderBuilder.html#method.from_reader
    // ...and so should be able to handle very large CSV files.

    let mut reader = match reader {
        Ok(reader) => reader,
        Err(error) => {
            // in the "thousands of concurrent connections" scenario...
            // this should not panic, but instead write an error to a logfile and process the next file
            panic!("error reading file: {}", error)
        }
    };

    // TODO replace this with a DB table
    let mut accounts: HashMap<u16, Account> = HashMap::new();

    // TODO replace this with a DB table
    let mut processed_transactions: HashMap<u32, Transaction> = HashMap::new();

    // in the "thousands of concurrent connections" scenario...
    // what I would do here is
    //  - accept one parsed transaction
    //  - hash the client ID
    //  - assign to a specific thread based on the hashed ID
    //  - probably use channels as a unidirectional way of sending Transactions across threads
    //
    // The effect of the above is
    //  - transaction processing is split roughly evenly across N threads
    //      - reduction of "hot spots" from a more naive work distribution
    //  - all transactions for a specific client land on that thread
    //      - this is good for caching, fewer unique clients spend more time in memory / CPU cache
    //      - this is good for DB locking, only one thread is requesting that client's row in the DB

    for result in reader.deserialize() {

        // parse the lines of the CSV as Transactions
        let tx: Transaction = match result {
            Err(error) => {
                // skip invalid lines, do not crash the entire program
                eprintln!("could not parse CSV line as Transaction, skipping: {}", error);
                continue
            }
            Ok(tx) => tx,
        };

        // if we have a duplicate transaction ID, refuse to process and throw an error
        if processed_transactions.contains_key(&tx.tx) {
            eprintln!("transaction {} has already been processed", tx.tx);
            continue
        }

        // if the transaction amount is negative, refuse to process and throw an error
        if tx.amount < 0.0 {
            eprintln!("transaction {} has a negative 'amount' and will not be processed", tx.tx);
            continue
        }

        match accounts.get_mut(&tx.client) {
            Some(account) => {
                if let Err(message) = account.apply(&tx, &processed_transactions) {
                    // there was an error applying the transactions, e.g. insufficient funds
                    eprintln!("{}", message)
                }

                // Regardless of whether the transaction application was successful, we mark the
                // transaction as processed. If the application crashes, we want to know the last
                // transaction we processed and start from there, to avoid double-processing.
                processed_transactions.insert(tx.tx, tx);
            }
            None => {
                match tx.kind {
                    TransactionType::Deposit => {
                        // If no Account exists, but we are making a deposit to it, create a new account
                        // TODO in a DB scenario, everything here must be a single (atomic) transaction
                        let account = Account {
                            id: tx.client,
                            available: tx.amount,
                            held: 0.0,
                            locked: false,
                        };
                        accounts.insert(tx.client, account);
                    }
                    TransactionType::Withdrawal => {

                        // NOTE: in this and all below cases, it's _possible_ that, in a distributed
                        // or multithreaded environment, that this is a valid out-of-order scenario.
                        // This should be handled if and when appropriate by moving "invalid"
                        // transactions into a retry queue, and handling as appropriate.

                        eprintln!("cannot withdraw from a new account (id: {})", tx.client);
                    }
                    TransactionType::Dispute => {
                        eprintln!("cannot dispute a transaction for a new account (id: {})", tx.client);
                    }
                    TransactionType::Resolve => {
                        eprintln!("cannot resolve a disputed transaction for a new account (id: {})", tx.client);
                    }
                    TransactionType::Chargeback => {
                        eprintln!("cannot chargeback a disputed transaction for a new account (id: {})", tx.client);
                    }
                }

                // again, in all cases, we mark these transactions as processed after we've handled them
                processed_transactions.insert(tx.tx, tx);
            }
        }
    }

    // in the "thousands of concurrent connections" scenario...
    // this should not write to stdout; we should instead expose an endpoint which allows for an
    // up-to-date report on specific accounts
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