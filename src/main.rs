mod model;
mod provider_a;
mod provider_b;
mod provider_c;

use std::{collections::BTreeMap, io::Read, time::Duration};

use crate::model::{NormalizationError, NormalizeData, Provider};

pub fn run_provider(
    data: &str, 
    provider: &mut dyn Provider) -> Result<(Vec<NormalizeData>, NormalizationError), NormalizationError> {
    if let Err(err) = provider.parse(data) {
        return Err(err);
    }
    let valdation_errors = provider.validate();
    Ok((provider.convert(), valdation_errors))
}

fn read_file_contents(file_path: &str) -> String {
    let mut output: String = "".into();
    match std::fs::File::open(file_path) {
        Ok(mut f) => {
            let _ = f.read_to_string(&mut output);
        },
        Err(err) => {
            println!("error reading file: {:?}", err);
        }
    }

    output
}

pub fn handle_data(
    provider_name: &str,
    file_path: &str, 
) -> Result<(Vec<NormalizeData>, NormalizationError), NormalizationError> {
    match provider_name {
        "a" => {
            let mut handler = provider_a::ProviderHandler::new();
            let data = read_file_contents(file_path);
            return run_provider(&data, &mut handler as &mut dyn Provider);
        },
        "b" => {
            let mut handler = provider_b::ProviderHandler::new();
            let data = read_file_contents(file_path);
            return run_provider(&data, &mut handler as &mut dyn Provider);
        },
        "c" => {
            let mut handler = provider_c::ProviderHandler::new();
            let data = read_file_contents(file_path);
            return run_provider(&data, &mut handler as &mut dyn Provider);
        },
        _ => {
            return Err(NormalizationError::Unknown(format!("Provider not found with name: {}", provider_name)));
        }
    }
}

fn main() {
    println!("Handling files of type a, b, c");
    println!("Press Ctrl-C to quit");
    // TODO: add error checking
    let input_path = "./input";
    let normalized_path = "./normalized";
    let _ = std::fs::create_dir_all(input_path);
    let _ = std::fs::create_dir_all(normalized_path);

    loop {
        std::thread::sleep(Duration::from_secs(3));
        if let Ok(entries) = std::fs::read_dir(input_path) {
            println!("Reading input dir: {}", input_path);
            for entry in entries {
                if let Ok(entry) = entry {
                    println!("file found: {:?}", entry);
                    if let Some(maybe_file_type) = entry.path().extension() {
                        let file_type = maybe_file_type.to_str();
                        if let Some(file_type) = file_type {
                            if let Some(file_path) = entry.path().to_str() {
                                match handle_data(&file_type, &file_path) {
                                    Ok(result) => {
                                        let output_path = format!("{}/normalize_{}.n", normalized_path, entry.file_name().to_str().unwrap_or("name_missing"));
                                        let _ = std::fs::write(output_path, serde_json::json!(result.0).to_string());
                                        if result.1 != NormalizationError::None {
                                            println!("Error processing file: {}\n{}", 
                                                file_path, 
                                                serde_json::json!(result.1).to_string());
                                        }

                                        std::fs::remove_file(file_path);
                                    },
                                    Err(err) => {
                                        println!("error processing file data: {:?}", err);
                                    }

                                }
                            }
                        }
                    }
                }
            }

        }
        
    }
}
