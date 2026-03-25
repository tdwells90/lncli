use crate::cli::{TeamsArgs, TeamsCommand};
use crate::graphql::client::GraphqlClient;
use crate::graphql::queries;
use crate::models::TeamsResponse;
use crate::utils::error::CliError;
use crate::utils::output;

pub async fn execute(client: &GraphqlClient, args: TeamsArgs) -> Result<(), CliError> {
    match args.command {
        TeamsCommand::List => list(client).await,
    }
}

async fn list(client: &GraphqlClient) -> Result<(), CliError> {
    let response: TeamsResponse = client
        .request(queries::TEAMS_LIST, serde_json::json!({}))
        .await?;
    output::print_json(&response.teams.nodes);
    Ok(())
}
