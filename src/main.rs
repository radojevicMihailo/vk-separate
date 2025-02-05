use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::env;
use clap::Parser;
use serde_json::{Value, Map};
use serde_json::ser::{Serializer, PrettyFormatter};

/// CLI tool for processing JSON files and removing specific fields.
#[derive(Parser)]
#[command(name = "json_processor", version = "1.0", about = "Processes JSON files by removing 'ck' fields.")]
struct Cli {
    /// Input JSON file
    input_file: String,
    
    /// Output directory
    output_dir: String,
    
    /// Output JSON file
    output_file: String,
}

fn main() -> io::Result<()> {
    let args = Cli::parse();
    
    let mut file = File::open(&args.input_file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let mut json_value: Value = serde_json::from_str(&contents).expect("Invalid JSON format");
    
    // Function to remove "ck" field from JSON
    fn remove_ck_field(obj: &mut Map<String, Value>) {
        if let Some(vk_primary) = obj.get_mut("vk_primary").and_then(Value::as_object_mut) {
            if let Some(vk_ee) = vk_primary.get_mut("vk_ee").and_then(Value::as_object_mut) {
                if let Some(ck_v) = vk_ee.get_mut("ck_v").and_then(Value::as_object_mut) {
                    ck_v.remove("ck");
                }
            }
        }
        if let Some(vk_secondary) = obj.get_mut("vk_secondary").and_then(Value::as_object_mut) {
            if let Some(vk_ee) = vk_secondary.get_mut("vk_ee").and_then(Value::as_object_mut) {
                if let Some(ck_v) = vk_ee.get_mut("ck_v").and_then(Value::as_object_mut) {
                    ck_v.remove("ck");
                }
            }
        }
    }

    if let Value::Object(ref mut map) = json_value {
        remove_ck_field(map);
    }
    
    // Ensure the output directory exists
    fs::create_dir_all(&args.output_dir)?;
    let output_path = format!("{}/{}", args.output_dir, args.output_file);
    let mut output_file = File::create(&output_path)?;
    
    // Write formatted JSON output
    let formatter = PrettyFormatter::new();
    let mut serializer = Serializer::with_formatter(&mut output_file, formatter);
    serde::Serialize::serialize(&json_value, &mut serializer)?;
    
    println!("Successfully processed JSON file and removed 'ck' fields while preserving field order.");
    
    Ok(())
}
