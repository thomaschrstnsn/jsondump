use clap::Parser;
use serde_json::Value;
use std::io::Write;
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

    /// Do not actually write files, only print what would be done
    #[arg(long)]
    dry_run: bool,

    /// How to name output files
    #[command(subcommand)]
    output: OutputNaming,
}

#[derive(clap::Subcommand, Debug)]
enum OutputNaming {
    /// Use array index
    ArrayIndex {
        #[arg(long)]
        padding: Option<bool>,
    },

    /// Field in nested object to use
    NestedField { field: String },

    /// Field in original object to use
    OriginalField { field: String },
}

fn get_field_as_string(value: &Value, field: &str) -> Option<String> {
    value.get(field)?.as_str().map(|s| s.to_owned())
}

fn filename(
    naming: &OutputNaming,
    index: usize,
    padding_size: usize,
    original: &Value,
    nested: &Value,
) -> Option<String> {
    let base = match &naming {
        OutputNaming::ArrayIndex { padding } => {
            let padding = if padding.unwrap_or(true) {
                padding_size
            } else {
                0
            };
            Some(format!("{:0padding$}", index))
        }
        OutputNaming::NestedField { field } => get_field_as_string(&nested, field),
        OutputNaming::OriginalField { field } => get_field_as_string(&original, field),
    };
    base.map(|base| format!("{}.json", base))
}

fn write_content(filename: &String, content: &String, dry_run: bool) -> () {
    let bytes = content.as_bytes();
    if dry_run {
        println!(
            "Would have written {} bytes to file: {}",
            bytes.len(),
            filename
        );
        return;
    }

    let path = std::path::Path::new(filename);
    if let Ok(true) = path.try_exists() {
        eprintln!("aborting, after finding existing file: {filename}");
        process::exit(1);
    }

    let mut file = std::fs::File::create(filename).unwrap();
    file.write_all(content.as_bytes()).unwrap();
    println!("Wrote {} bytes to file: {}", bytes.len(), filename);
}

fn main() {
    let args = Args::parse();

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
        .iter()
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

    let padding_size = format!("{}", actual).len();
    let outputs: Vec<_> = values
        .iter()
        .zip(nested_jsons.iter())
        .enumerate()
        .map_while(|(index, (original, nested))| {
            let filename = filename(&args.output, index, padding_size, &original, &nested)?;
            let content = nested.to_string();
            Some((filename, content))
        })
        .collect();

    {
        let outputs_len = outputs.len();
        if outputs_len != expected {
            eprintln!(
                "Could not map one or more object to get filename, got: {}, but expected: {}",
                outputs_len, expected
            );
            process::exit(1);
        }
    }

    if args.dry_run {
        println!("Simulating an actual run, without writing to files:");
    }
    for (filename, content) in outputs {
        write_content(&filename, &content, args.dry_run);
    }
}
