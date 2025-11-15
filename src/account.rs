use crate::transaction::{Transaction, TransactionType};
use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct Account {
    pub(crate) id: u16,
    pub(crate) available: f32, // TODO consider using the 'rust_decimal' crate for money
    pub(crate) held: f32,  // TODO consider using the 'rust_decimal' crate for money
    pub(crate) locked: bool,
}

impl Account {

    // Rather than storing this in the Account as well, compute it on the fly. If this becomes a
    // bottleneck (if we're computing `total` a lot), consider keeping `total` in memory and using
    // it to calculate `available` or `held` instead.
    pub(crate) fn total(&self) -> f32 {
        self.available + self.held
    }

    // returns an error if there was a failure to apply
    pub(crate) fn apply(
        &mut self,
        tx: &Transaction,
        processed_transactions: &HashMap<u32, Transaction>,
    ) -> Result<(), &str> {
        match tx.kind {
            TransactionType::Deposit => {
                self.available += tx.amount.unwrap_or(0.0); // Deposits _should_ always have amounts
                Ok(())
            }
            TransactionType::Withdrawal => {
                let tx_amount = tx.amount.unwrap_or(0.0); // Withdrawals _should_ always have amounts
                if self.available >= tx_amount {
                    self.available -= tx_amount;
                    Ok(())
                } else {
                    Err("insufficient funds")
                }
            }

            // NOTE: for all TransactionTypes below, in the case where processed_transactions.get()
            // returns None, it's possible that a valid out-of-order scenario has occurred. These
            // kinds of scenarios should be handled if / when appropriate.
            //
            // However, the instructions say: "If the tx specified by the dispute doesn't exist you
            // can ignore it and assume this is an error on our partners side." So that's what I'll
            // do.

            TransactionType::Dispute => {

                // 1. find disputed transaction
                // 2. drop available balance by that amount (what to do if insufficient funds?)
                // 3. increase held funds by that amount (what to do if insufficient funds?)

                match processed_transactions.get(&tx.tx) {
                    None => {
                        Err("attempt to dispute unknown transaction")
                    }
                    Some(disputed) => {

                        // disputed transactions _should_ only ever be Deposits / Withdrawals, which _should_ have amounts
                        let disputed_amount = disputed.amount.unwrap_or(0.0);

                        // TODO log a warning if there was an attempt to dispute anything other than a Deposit / Withdrawal

                        if self.available >= disputed_amount {
                            self.available -= disputed_amount;
                            self.held += disputed_amount;
                            Ok(())
                        } else {
                            // if disputed amount is greater than user's available balance,
                            // hold their entire available balance -- it's better to lose
                            // _some_ money than lose all of it

                            // TODO fire an alert that this user disputed a transaction which
                            //   ought to have put their account in the red

                            // NOTE this assumes all monetary amounts are positive
                            self.held += self.available;
                            self.available = 0.0;
                            Ok(())
                        }
                    }
                }
            }
            TransactionType::Resolve => {

                // 1. find resolved transaction
                // 2. increase available balance by that amount
                // 3. decrease held funds by that amount (what to do if insufficient funds?)

                match processed_transactions.get(&tx.tx) {
                    None => {
                        Err("attempt to resolve unknown disputed transaction")
                    }
                    Some(disputed) => {

                        // disputed transactions _should_ only ever be Deposits / Withdrawals, which _should_ have amounts
                        let disputed_amount = disputed.amount.unwrap_or(0.0);

                        // TODO log a warning if there was an attempt to dispute anything other than a Deposit / Withdrawal

                        if self.held >= disputed_amount {
                            self.available += disputed_amount;
                            self.held -= disputed_amount;
                            Ok(())
                        } else {
                            // if resolved amount is greater than user's held balance,
                            // transfer their entire held balance and then fire an alert
                            // so customer service can look into the issue further

                            // TODO fire an alert that this user had a disputed transaction
                            //   resolved, but didn't have enough 'held' funds to refund to
                            //   their available balance

                            // NOTE this assumes all monetary amounts are positive
                            self.available += self.held;
                            self.held = 0.0;
                            Ok(())
                        }
                    }
                }
            }
            TransactionType::Chargeback => {

                // 1. find chargeback transaction
                // 2. decrease held balance by that amount
                // 3. freeze the client's account

                match processed_transactions.get(&tx.tx) {
                    None => {
                        Err("attempt to resolve unknown disputed transaction")
                    }
                    Some(disputed) => {

                        // FIXME the docs are not clear here if the transaction ID that a Chargeback
                        //   holds is the ID of the Dispute or of the original Withdrawal / Deposit.
                        //   (i.e. do we need to do "one hop" or "two hops" to get the amount back)
                        //   Resolve specifically says "a transaction that was under dispute by ID"
                        //   I'm assuming that it's the original Withdrawal / Deposit, containing
                        //   the original amount.

                        // disputed transactions _should_ only ever be Deposits / Withdrawals, which _should_ have amounts
                        let disputed_amount = disputed.amount.unwrap_or(0.0);

                        // TODO log a warning if there was an attempt to dispute anything other than a Deposit / Withdrawal

                        if self.held >= disputed_amount {
                            self.held -= disputed_amount;
                            self.locked = true;
                            Ok(())
                        } else {
                            // if chargeback amount is greater than user's held balance,
                            // reduce held balance to zero and then fire an alert
                            // so customer service can look into the issue further

                            // TODO fire an alert that this user had a chargeback
                            //   resolved, but didn't have enough 'held' funds

                            // NOTE this assumes all monetary amounts are positive
                            self.held = 0.0;
                            self.locked = true;
                            Ok(())
                        }
                    }
                }
            }
        }
    }
}