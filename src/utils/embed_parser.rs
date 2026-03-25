use regex::Regex;
use serde::Serialize;
use std::sync::LazyLock;

static CODE_BLOCK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?s)```.*?```|`[^`]+`").unwrap());

static EMBED_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"!?\[([^\]]*)\]\((https://uploads\.linear\.app/[^)]+)\)").unwrap()
});

#[derive(Debug, Clone, Serialize)]
pub struct EmbedInfo {
    pub label: String,
    pub url: String,
}

/// Extract Linear upload URLs from markdown content, ignoring code blocks.
pub fn extract_embeds(content: &str) -> Vec<EmbedInfo> {
    // Strip code blocks to avoid false positives
    let stripped = CODE_BLOCK_RE.replace_all(content, "");

    EMBED_RE
        .captures_iter(&stripped)
        .map(|cap| EmbedInfo {
            label: cap[1].to_string(),
            url: cap[2].to_string(),
        })
        .collect()
}
