use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::time::Duration;

use crate::utils::error::CliError;

const LINEAR_API_URL: &str = "https://api.linear.app/graphql";

pub struct GraphqlClient {
    client: reqwest::Client,
    token: String,
}

impl GraphqlClient {
    pub fn new(token: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            client,
            token: token.to_string(),
        }
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn http_client(&self) -> &reqwest::Client {
        &self.client
    }

    pub async fn request<T: DeserializeOwned>(
        &self,
        query: &str,
        variables: Value,
    ) -> Result<T, CliError> {
        let data = self.request_raw(query, variables).await?;
        serde_json::from_value(data)
            .map_err(|e| CliError::GraphqlError(format!("Failed to deserialize data: {e}")))
    }

    pub async fn request_raw(&self, query: &str, variables: Value) -> Result<Value, CliError> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&self.token)
                .map_err(|e| CliError::AuthError(format!("Invalid token format: {e}")))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let body = serde_json::json!({
            "query": query,
            "variables": variables,
        });

        let response = self
            .client
            .post(LINEAR_API_URL)
            .headers(headers)
            .json(&body)
            .send()
            .await?;

        let status = response.status();
        let response_text = response.text().await?;

        if !status.is_success() {
            return Err(CliError::GraphqlError(format!(
                "HTTP {status}: {response_text}"
            )));
        }

        let response_json: Value = serde_json::from_str(&response_text)
            .map_err(|e| CliError::GraphqlError(format!("Failed to parse response: {e}")))?;

        if let Some(errors) = response_json.get("errors") {
            if let Some(arr) = errors.as_array() {
                if let Some(first) = arr.first() {
                    let msg = first
                        .get("message")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown GraphQL error");
                    return Err(CliError::GraphqlError(msg.to_string()));
                }
            }
        }

        response_json
            .get("data")
            .ok_or_else(|| CliError::GraphqlError("No data in response".to_string()))
            .cloned()
    }
}
