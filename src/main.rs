use std::fs;

fn main() {
    let filename = std::env::args_os().nth(1).expect("missing file argument");
    println!("attempting to read file: {:?}", filename);

    match fs::read_to_string(filename) {
        Ok(file) => {
            println!("read file contents: {}", file)
        }
        Err(error) => {
            // in the "thousands of concurrent connections" scenario
            // this should not panic, but instead write an error to a logfile
            panic!("error reading file: {}", error)
        }
    }

}
