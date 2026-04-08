use crate::cli::{UsersArgs, UsersCommand};
use crate::graphql::client::GraphqlClient;
use crate::graphql::queries;
use crate::models::{UsersResponse, ViewerResponse};
use crate::utils::error::CliError;
use crate::utils::fields::{self, filter_json_nodes};
use crate::utils::output;

const USER_MANDATORY_FIELDS: &[&str] = &["id"];

pub async fn execute(
    client: &GraphqlClient,
    args: UsersArgs,
    fields_filter: Option<&str>,
) -> Result<(), CliError> {
    match args.command {
        UsersCommand::List { me: true } => me(client, fields_filter).await,
        UsersCommand::List { me: false } => list(client, fields_filter).await,
    }
}

async fn me(client: &GraphqlClient, fields_filter: Option<&str>) -> Result<(), CliError> {
    let raw_response: serde_json::Value = client
        .request_raw(queries::VIEWER, serde_json::json!({}))
        .await?;

    let viewer_value = if let Some(filter_str) = fields_filter {
        let parsed_fields = fields::parse_fields(filter_str);
        filter_json_nodes(
            &raw_response["viewer"],
            &parsed_fields,
            USER_MANDATORY_FIELDS,
        )
    } else {
        raw_response["viewer"].clone()
    };

    let response: ViewerResponse =
        serde_json::from_value(serde_json::json!({ "viewer": viewer_value }))?;
    output::print_json(&response.viewer);
    Ok(())
}

async fn list(client: &GraphqlClient, fields_filter: Option<&str>) -> Result<(), CliError> {
    let raw_response: serde_json::Value = client
        .request_raw(queries::USERS_LIST, serde_json::json!({}))
        .await?;

    let users_value = if let Some(filter_str) = fields_filter {
        let parsed_fields = fields::parse_fields(filter_str);
        filter_json_nodes(
            &raw_response["users"],
            &parsed_fields,
            USER_MANDATORY_FIELDS,
        )
    } else {
        raw_response["users"].clone()
    };

    let response: UsersResponse =
        serde_json::from_value(serde_json::json!({ "users": users_value }))?;
    output::print_json(&response.users.nodes);
    Ok(())
}
