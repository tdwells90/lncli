use std::io::{self, Read};

use crate::utils::error::CliError;

const STDIN_SENTINEL: &str = "-";

/// If value is "-", read from stdin. Otherwise return as-is.
pub fn resolve_value(value: String) -> Result<String, CliError> {
    if value == STDIN_SENTINEL {
        read_stdin()
    } else {
        Ok(value)
    }
}

/// If value is Some("-"), read from stdin. None passes through.
pub fn resolve_optional(value: Option<String>) -> Result<Option<String>, CliError> {
    match value {
        Some(v) if v == STDIN_SENTINEL => Ok(Some(read_stdin()?)),
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

fn read_stdin() -> Result<String, CliError> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    let trimmed = buf.trim_end_matches('\n').trim_end_matches('\r');
    Ok(trimmed.to_string())
}
