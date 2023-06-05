use clap::Parser;
use serde_json::Value;
use std::{fs, process};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of the JSON file to read
    #[arg(short, long)]
    filename: String,

    /// Field in the JSON that contains the nested JSON
    #[arg(long)]
    jsonfield: String,
}

fn main() {
    let args = Args::parse();

    println!("args {:?}", args);

    let data = fs::read(args.filename).unwrap_or_else(|err| {
        eprintln!("Could not read input file: {err}");
        process::exit(1);
    });

    let parsed: Value = serde_json::from_slice(&data).unwrap_or_else(|err| {
        eprintln!("Could not parse file contents as JSON: {err}");
        process::exit(1);
    });

    let Value::Array(values) = parsed else {
        eprintln!("Expected to find array of objects");
        process::exit(1);
    };
    let expected = values.len();
    let nested_jsons: Vec<Value> = values
        .into_iter()
        .map_while(|v| {
            let Some(s) = v.get(&args.jsonfield)?.as_str() else {
                return None;
            };
            match serde_json::from_str(s) {
                Err(err) => {
                    eprintln!("Error while parsing nested field as JSON, {err}");
                    None
                }
                Ok(val) => Some(val),
            }
        })
        .collect();

    let actual = nested_jsons.len();
    if actual != expected {
        eprintln!(
            "Could not apply field to each element in array. Only {actual}, expected {expected}"
        );
        process::exit(1);
    }
}
