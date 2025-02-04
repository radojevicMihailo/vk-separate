use std::fs::{self, File};
use std::io::{self, Read};
use std::env;
use serde_json::{Value, Map};
use serde_json::ser::Serializer;
use serde_json::ser::PrettyFormatter;

// ! cargo run -- <input_json_file> <output_directory> <output_json_file>
// ! cargo run -- vk_spark_cubic.json ./ vk_compressed.json

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <input_json_file> <output_directory> <output_json_file>", args[0]);
        std::process::exit(1);
    }

    let input_file = &args[1];
    let output_dir = &args[2];
    let output_file = &args[3];

    let mut file = File::open(input_file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let mut json_value: Value = serde_json::from_str(&contents).expect("Invalid JSON format");
    
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
    
    fs::create_dir_all(output_dir)?;
    let output_path = format!("{}/{}", output_dir, output_file);
    let output_file = File::create(&output_path)?;
    let formatter = PrettyFormatter::new();
    let mut serializer = Serializer::with_formatter(output_file, formatter);
    serde::Serialize::serialize(&json_value, &mut serializer)?;
    
    println!("Successfully processed JSON file and removed 'ck' fields while preserving field order.");
    
    Ok(())
}
