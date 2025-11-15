# payments-engine

This is a simple payments engine CLI, written in Rust.

It
 - reads in a buffered fashion from a single CSV file using the `csv` crate
 - parses lines as "Transactions" using `serde`, immediately rejecting improperly-formatted lines
 - ignores transactions with negative `amounts`, as this is one of the key assumptions made when processing transactions
 - ignores duplicate transactions, as indicated by their unique `tx` ID numbers
 - processes `Deposit`s, `Withdrawal`s, `Dispute`s, `Resolve`s (resolutions), and `Chargeback`s
 - creates user `Account`s on `Deposit` transactions, if they do not yet exist
 - prints a nicely-formatted report of the state of user accounts after processing

There are lots of in-line comments which explain the inner workings of the code, as well as alternative approaches.

Possible improvements include
 - storing `Account` and processed `Transaction` information in a database, rather than in-memory
 - adding multithreading, distributed by hashed `Account` IDs
 - using a `rust_decimal`-like crate for monetary amounts
 - adding a crate like `typed_floats` to get a `Positive<f32>` type, adding compile-time assurances of positive `amount`s
   - (this could also be done with a built-for-purpose macro in this crate)

The correctness of the code in this project relies heavily on Rust's type system and runtime checks, rather than tests. Regression tests should be added, though, to ensure correct behaviour in scenarios like
 - depositing funds into an existing account
 - withdrawing funds from an existing account with sufficient balance 
 - attempting to deposit into an account which doesn't exist (the account should be created)
 - attempting to overdraw on an account (the withdrawal should be refused)
 - applying a `Dispute`, `Resolve`, or `Chargeback` to an unknown `Transaction`
 - performing a `Dispute`, `Resolve`, or `Chargeback` in cases of insufficient funds

These would all be unit tests of the `Account.apply()` method.