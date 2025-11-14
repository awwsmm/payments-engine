use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Type {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize)]
struct Transaction {
    // note, there are a few ways around the below, we could also use a raw identifier like r#type
    #[serde(rename = "type")] kind: Type,
    client: u16, // client / account ID
    tx: u32, // transaction ID
    // TODO amount must be positive -- this is very, very important
    amount: f32, // TODO consider using the 'rust_decimal' crate for money
}

#[derive(Debug)]
struct Account {
    id: u16,
    available: f32, // TODO consider using the 'rust_decimal' crate for money
    held: f32,  // TODO consider using the 'rust_decimal' crate for money
    locked: bool,
}

impl Account {
    fn total(&self) -> f32 {
        self.available + self.held
    }
}

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
        let transaction: Transaction = result.expect("could not parse line");

        // handle each transaction appropriately
        match transaction.kind {
            Type::Deposit => {
                match accounts.get_mut(&transaction.client) {
                    None => {
                        // TODO in a DB scenario, everything here must be a single transaction
                        let account = Account {
                            id: transaction.client,
                            available: transaction.amount,
                            held: 0.0,
                            locked: false,
                        };
                        accounts.insert(transaction.client, account);
                        processed_transactions.insert(transaction.tx, transaction);
                    }
                    Some(account) => {
                        // TODO in a DB scenario, everything here must be a single transaction
                        account.available += transaction.amount;
                        processed_transactions.insert(transaction.tx, transaction);
                    }
                }
            }
            Type::Withdrawal => {
                match accounts.get_mut(&transaction.client) {
                    None => {
                        // TODO this should fail "nicely"
                        processed_transactions.insert(transaction.tx, transaction);
                        // NOTE: it's _possible_ in a distributed / multithreaded environment, that
                        // this is an out-of-order scenario. Handle this if / when appropriate.
                        eprintln!("cannot withdraw from a new account");
                    }
                    Some(account) => {
                        if account.available >= transaction.amount {
                            account.available -= transaction.amount;
                            processed_transactions.insert(transaction.tx, transaction);
                        } else {
                            // TODO this should fail "nicely"
                            processed_transactions.insert(transaction.tx, transaction);
                            eprintln!("insufficient funds")
                        }
                    }
                }
            }
            Type::Dispute => {
                match accounts.get_mut(&transaction.client) {
                    None => {
                        // TODO this should fail "nicely"
                        processed_transactions.insert(transaction.tx, transaction);

                        // NOTE: it's _possible_ in a distributed / multithreaded environment, that
                        // this is an out-of-order scenario. Handle this if / when appropriate.
                        eprintln!("cannot dispute a transaction for a new account");
                    }
                    Some(account) => {

                        // 1. find disputed transaction
                        // 2. drop available balance by that amount (what to do if insufficient funds?)
                        // 3. increase held funds by that amount (what to do if insufficient funds?)

                        match processed_transactions.get(&transaction.tx) {
                            None => {

                                // "If the tx specified by the dispute doesn't exist you can ignore it and
                                // assume this is an error on our partners side."

                                // NOTE: it's _possible_ in a distributed / multithreaded environment, that
                                // this is an out-of-order scenario. Handle this if / when appropriate.
                                processed_transactions.insert(transaction.tx, transaction);
                                eprintln!("attempt to dispute unknown transaction");
                            }
                            Some(disputed) => {
                                if account.available >= disputed.amount {
                                    account.available -= disputed.amount;
                                    account.held += disputed.amount;
                                    processed_transactions.insert(transaction.tx, transaction);
                                } else {
                                    // if disputed amount is greater than user's available balance,
                                    // hold their entire available balance -- it's better to lose
                                    // _some_ money than lose all of it

                                    // TODO fire an alert that this user disputed a transaction which
                                    //   ought to have put their account in the red

                                    // NOTE this assumes all monetary amounts are positive
                                    account.held += account.available;
                                    account.available = 0.0;
                                    processed_transactions.insert(transaction.tx, transaction);
                                }
                            }
                        }
                    }
                }
            }
            Type::Resolve => {
                match accounts.get_mut(&transaction.client) {
                    None => {
                        // TODO this should fail "nicely"
                        processed_transactions.insert(transaction.tx, transaction);

                        // NOTE: it's _possible_ in a distributed / multithreaded environment, that
                        // this is an out-of-order scenario. Handle this if / when appropriate.
                        eprintln!("cannot resolve a disputed transaction for a new account");
                    }
                    Some(account) => {

                        // 1. find resolved transaction
                        // 2. increase available balance by that amount
                        // 3. decrease held funds by that amount (what to do if insufficient funds?)

                        match processed_transactions.get(&transaction.tx) {
                            None => {

                                // "If the tx specified by the dispute doesn't exist you can ignore it and
                                // assume this is an error on our partners side."

                                // NOTE: it's _possible_ in a distributed / multithreaded environment, that
                                // this is an out-of-order scenario. Handle this if / when appropriate.
                                processed_transactions.insert(transaction.tx, transaction);
                                eprintln!("attempt to resolve unknown disputed transaction");
                            }
                            Some(disputed) => {
                                if account.held >= disputed.amount {
                                    account.available += disputed.amount;
                                    account.held -= disputed.amount;
                                    processed_transactions.insert(transaction.tx, transaction);
                                } else {
                                    // if resolved amount is greater than user's held balance,
                                    // transfer their entire held balance and then fire an alert
                                    // so customer service can look into the issue further

                                    // TODO fire an alert that this user had a disputed transaction
                                    //   resolved, but didn't have enough 'held' funds to refund to
                                    //   their available balance

                                    // NOTE this assumes all monetary amounts are positive
                                    account.available += account.held;
                                    account.held = 0.0;
                                    processed_transactions.insert(transaction.tx, transaction);
                                }
                            }
                        }
                    }
                }
            }
            Type::Chargeback => {
                match accounts.get_mut(&transaction.client) {
                    None => {
                        // TODO this should fail "nicely"
                        processed_transactions.insert(transaction.tx, transaction);

                        // NOTE: it's _possible_ in a distributed / multithreaded environment, that
                        // this is an out-of-order scenario. Handle this if / when appropriate.
                        eprintln!("cannot chargeback a disputed transaction for a new account");
                    }
                    Some(account) => {

                        // 1. find chargeback transaction
                        // 2. decrease held balance by that amount
                        // 3. freeze the client's account

                        match processed_transactions.get(&transaction.tx) {
                            None => {

                                // "If the tx specified by the dispute doesn't exist you can ignore it and
                                // assume this is an error on our partners side."

                                // NOTE: it's _possible_ in a distributed / multithreaded environment, that
                                // this is an out-of-order scenario. Handle this if / when appropriate.
                                processed_transactions.insert(transaction.tx, transaction);
                                eprintln!("attempt to resolve unknown disputed transaction");
                            }
                            Some(chargeback) => {
                                if account.held >= chargeback.amount {
                                    account.held -= chargeback.amount;
                                    processed_transactions.insert(transaction.tx, transaction);
                                } else {
                                    // if chargeback amount is greater than user's held balance,
                                    // reduce held balance to zero and then fire an alert
                                    // so customer service can look into the issue further

                                    // TODO fire an alert that this user had a chargeback
                                    //   resolved, but didn't have enough 'held' funds

                                    // NOTE this assumes all monetary amounts are positive
                                    account.held = 0.0;
                                    processed_transactions.insert(transaction.tx, transaction);
                                }
                            }
                        }
                    }
                }
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