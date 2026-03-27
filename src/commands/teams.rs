use crate::cli::{TeamsArgs, TeamsCommand};
use crate::graphql::client::GraphqlClient;
use crate::graphql::queries;
use crate::models::TeamsResponse;
use crate::utils::error::CliError;
use crate::utils::fields::{self, filter_json_nodes};
use crate::utils::output;

const TEAM_MANDATORY_FIELDS: &[&str] = &["id"];

pub async fn execute(
    client: &GraphqlClient,
    args: TeamsArgs,
    fields_filter: Option<&str>,
) -> Result<(), CliError> {
    match args.command {
        TeamsCommand::List => list(client, fields_filter).await,
    }
}

async fn list(client: &GraphqlClient, fields_filter: Option<&str>) -> Result<(), CliError> {
    let raw_response: serde_json::Value = client
        .request_raw(queries::TEAMS_LIST, serde_json::json!({}))
        .await?;

    let teams_value = if let Some(filter_str) = fields_filter {
        let parsed_fields = fields::parse_fields(filter_str);
        filter_json_nodes(
            &raw_response["teams"],
            &parsed_fields,
            TEAM_MANDATORY_FIELDS,
        )
    } else {
        raw_response["teams"].clone()
    };

    let response: TeamsResponse =
        serde_json::from_value(serde_json::json!({ "teams": teams_value }))?;
    output::print_json(&response.teams.nodes);
    Ok(())
}
