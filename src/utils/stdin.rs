use std::io::{self, IsTerminal, Read};

use crate::utils::error::CliError;

const STDIN_SENTINEL: &str = "-";

/// If value is "-", read from stdin. Otherwise return as-is.
pub async fn resolve_value(value: String) -> Result<String, CliError> {
    if value == STDIN_SENTINEL {
        read_stdin().await
    } else {
        Ok(value)
    }
}

/// If value is Some("-"), read from stdin. None passes through.
pub async fn resolve_optional(value: Option<String>) -> Result<Option<String>, CliError> {
    match value {
        Some(v) if v == STDIN_SENTINEL => Ok(Some(read_stdin().await?)),
        other => Ok(other),
    }
}

/// Validate that at most one field in a set is requesting stdin.
pub fn validate_at_most_one_stdin(fields: &[(&str, Option<&str>)]) -> Result<(), CliError> {
    let stdin_fields: Vec<&str> = fields
        .iter()
        .filter(|(_, v)| *v == Some(STDIN_SENTINEL))
        .map(|(name, _)| *name)
        .collect();

    if stdin_fields.len() > 1 {
        return Err(CliError::InvalidParameter {
            param: stdin_fields.join(", "),
            reason: format!(
                "only one field can read from stdin (-) per invocation, but {} were set to \"-\"",
                stdin_fields.len()
            ),
        });
    }
    Ok(())
}

async fn read_stdin() -> Result<String, CliError> {
    if io::stdin().is_terminal() {
        return Err(CliError::Other(
            "stdin is a terminal — pipe or redirect input when using \"-\" (e.g. echo \"text\" | lncli ...)"
                .to_string(),
        ));
    }

    let buf = tokio::task::spawn_blocking(|| {
        let mut buf = String::new();
        io::stdin().read_to_string(&mut buf)?;
        Ok::<String, io::Error>(buf)
    })
    .await
    .map_err(|e| CliError::Other(format!("stdin read task failed: {e}")))??;

    Ok(buf.trim_end_matches(['\n', '\r']).to_string())
}
