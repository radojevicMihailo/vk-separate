use std::fs::{self, File};
use std::io::Read;
use clap::Parser;
use nova_snark::provider::{PallasEngine, VestaEngine};
use nova_snark::traits::circuit::{GenericCircuit, StepCircuit};
use nova_snark::traits::evaluation::EvaluationEngineTrait;
use nova_snark::traits::Engine;
use nova_snark::VerifierKey;
use pasta_curves::{Fp, Fq};
use serde_json::{Value, Map, json};
use serde_json::ser::{Serializer, PrettyFormatter};

type EE<E> = nova_snark::provider::ipa_pc::EvaluationEngine<E>;
type S<E, EE> = nova_snark::spartan::ppsnark::RelaxedR1CSSNARK<E, EE>;

/// CLI tool for processing JSON files and removing specific fields.
#[derive(Parser)]
#[command(name = "vk_separate", version = "1.0", about = "Processes JSON files by removing 'ck' fields.")]
struct Cli {
    /// Input JSON file
    input_file: String,
    
    /// Output directory
    output_dir: String,
    
    /// Output JSON file
    output_file: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
                    ck_v.insert("ck".to_string(), json!([]));
                }
            }
        }
        if let Some(vk_secondary) = obj.get_mut("vk_secondary").and_then(Value::as_object_mut) {
            if let Some(vk_ee) = vk_secondary.get_mut("vk_ee").and_then(Value::as_object_mut) {
                if let Some(ck_v) = vk_ee.get_mut("ck_v").and_then(Value::as_object_mut) {
                    ck_v.insert("ck".to_string(), json!([]));
                }
            }
        }
    }

    if let Value::Object(ref mut map) = json_value {
        remove_ck_field(map);
    }
    
    // Ensure the output directory exists
    fs::create_dir_all(&args.output_dir)?;
    let output_path = format!("{}/{}.json", args.output_dir, args.output_file);
    let mut output_file = File::create(&output_path)?;
    
    // Write formatted JSON output
    let formatter = PrettyFormatter::new();
    let mut serializer = Serializer::with_formatter(&mut output_file, formatter);
    serde::Serialize::serialize(&json_value, &mut serializer)?;
    
    println!("Successfully processed JSON file and replaced 'ck' value with empty array.");

    fn create_bin_file<E1, E2, C1, C2, EE1, EE2>(
        path: &str,
        file_name: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>>
    where
        E1: Engine<Base = <E2 as Engine>::Scalar>,
        E2: Engine<Base = <E1 as Engine>::Scalar>,
        C1: StepCircuit<E1::Scalar>,
        C2: StepCircuit<E2::Scalar>,
        EE1: EvaluationEngineTrait<E1>,
        EE2: EvaluationEngineTrait<E2>, 
    {
        let json_path_vk = format!("{}/{}.json", path, file_name);
        // ! Read from JSON to String
        let json_string_vk = fs::read_to_string(json_path_vk)?;

        // ! From string into Vk
        let json_data_vk: VerifierKey<E1, E2, C1, C2, S<E1, EE1>, S<E2, EE2>> =
            serde_json::from_str(&json_string_vk)?;

        // ! Serialize into Bytes
        let bytes_vk = postcard::to_allocvec(&json_data_vk)?;

        // ! Write bytes to BIN file
        let output_bin_path_vk = format!("{}/{}", path, file_name);
        fs::write(output_bin_path_vk.clone(), &bytes_vk)?;

        Ok(bytes_vk)
    }

    let _ = create_bin_file::<
        PallasEngine,
        VestaEngine,
        GenericCircuit<Fq>,
        GenericCircuit<Fp>,
        EE<_>,
        EE<_>
    >(&args.output_dir, &args.output_file)?;

    println!("Successfully created binary file from JSON file after replacement.");
    
    Ok(())
}
