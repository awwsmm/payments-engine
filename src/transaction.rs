use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Debug, Deserialize)]
pub(crate) struct Transaction {
    // "type" is a reserved word in Rust, so we have to rename this field
    // we could also use a raw identifier like r#type
    #[serde(rename = "type")]
    pub(crate) kind: TransactionType,

    // Above, we deserialize directly to a list of known transaction types. The effect of this is
    // that lines with unknown "type"s are simply discarded and not processed at all. An alternative
    // approach would be to deserialize "kind" as a String and simply ignore the transaction,
    // writing to a log that we have an unknown "type". This might be the preferred approach if, for
    // example, auditability is a concern.

    pub(crate) client: u16, // client / account ID
    pub(crate) tx: u32, // transaction ID

    // TODO amount must be positive -- this is very, very important
    // Option because amount will not be present for Dispute, Resolve, Chargeback
    pub(crate) amount: Option<f32>, // TODO consider using the 'rust_decimal' crate for money
}