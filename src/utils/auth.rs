use std::env;
use std::fs;
use std::path::PathBuf;

use super::error::CliError;

pub fn get_api_token(cli_token: Option<&str>) -> Result<String, CliError> {
    // 1. CLI flag
    if let Some(token) = cli_token {
        let token = token.trim();
        if !token.is_empty() {
            return Ok(token.to_string());
        }
    }

    // 2. Environment variable
    if let Ok(token) = env::var("LINEAR_API_TOKEN") {
        let token = token.trim().to_string();
        if !token.is_empty() {
            return Ok(token);
        }
    }

    // 3. Token file
    let home = env::var("HOME")
        .or_else(|_| env::var("USERPROFILE"))
        .map_err(|_| CliError::AuthError("Could not determine home directory".to_string()))?;

    let token_path = PathBuf::from(home).join(".linear_api_token");
    if token_path.exists() {
        let token = fs::read_to_string(&token_path)
            .map_err(|e| {
                CliError::AuthError(format!(
                    "Failed to read token file {}: {e}",
                    token_path.display()
                ))
            })?
            .trim()
            .to_string();
        if !token.is_empty() {
            return Ok(token);
        }
    }

    Err(CliError::AuthError(
        "No API token found. Provide one via --api-token, LINEAR_API_TOKEN env var, or ~/.linear_api_token file".to_string(),
    ))
}
