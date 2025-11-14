use serde::Deserialize;

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
    client: u16,
    tx: u32,
    amount: f32, // TODO replace this with a proper currency type
}

fn main() {
    let filename = std::env::args_os().nth(1).expect("missing file argument");
    println!("attempting to read file: {:?}", filename);

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

    for result in reader.deserialize() {
        let transaction: Transaction = result.expect("could not parse line");
        println!("{:?}", transaction);
    }
}