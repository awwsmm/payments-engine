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
    // note, there are a few ways around the below, we could also use a raw identifier like r#type
    #[serde(rename = "type")]
    pub(crate) kind: TransactionType,
    pub(crate) client: u16, // client / account ID
    pub(crate) tx: u32, // transaction ID
    // TODO amount must be positive -- this is very, very important
    pub(crate) amount: f32, // TODO consider using the 'rust_decimal' crate for money
}