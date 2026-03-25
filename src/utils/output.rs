use std::sync::OnceLock;

use serde::Serialize;

use super::error::CliError;
use crate::cli::OutputFormat;

static FORMAT: OnceLock<OutputFormat> = OnceLock::new();

pub fn set_format(fmt: OutputFormat) {
    let _ = FORMAT.set(fmt);
}

fn format() -> OutputFormat {
    FORMAT.get().copied().unwrap_or(OutputFormat::Toon)
}

pub fn print_json<T: Serialize>(data: &T) {
    match format() {
        OutputFormat::Json => match serde_json::to_string_pretty(data) {
            Ok(json) => println!("{json}"),
            Err(e) => eprintln!("{{\"error\": \"Failed to serialize output: {e}\"}}"),
        },
        OutputFormat::Toon => match toon_format::encode_default(data) {
            Ok(toon) => println!("{toon}"),
            Err(e) => eprintln!("error: Failed to serialize output: {e}"),
        },
    }
}

pub fn print_error(err: &CliError) {
    match format() {
        OutputFormat::Json => {
            let error_obj = serde_json::json!({ "error": err.to_string() });
            match serde_json::to_string_pretty(&error_obj) {
                Ok(json) => eprintln!("{json}"),
                Err(_) => eprintln!("{{\"error\": \"{err}\"}}"),
            }
        }
        OutputFormat::Toon => {
            eprintln!("error: {err}");
        }
    }
}
