use regex::Regex;
use std::sync::LazyLock;

static UUID_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$")
        .unwrap()
});

static IDENTIFIER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([A-Za-z]+)-(\d+)$").unwrap());

static DOCUMENT_URL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"https://linear\.app/[^/]+/document/[^/]+-([a-f0-9]+)$").unwrap()
});

pub fn is_uuid(value: &str) -> bool {
    UUID_RE.is_match(value)
}

/// Parse a team-prefixed identifier like "ABC-123" into (team_key, number).
pub fn parse_issue_identifier(value: &str) -> Option<(String, u32)> {
    let caps = IDENTIFIER_RE.captures(value)?;
    let team_key = caps.get(1)?.as_str().to_uppercase();
    let number: u32 = caps.get(2)?.as_str().parse().ok()?;
    Some((team_key, number))
}

/// Extract a document slug ID from a Linear document URL, or return the input as-is.
pub fn extract_document_id(value: &str) -> String {
    if let Some(caps) = DOCUMENT_URL_RE.captures(value) {
        if let Some(m) = caps.get(1) {
            return m.as_str().to_string();
        }
    }
    value.to_string()
}
