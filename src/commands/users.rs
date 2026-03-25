use crate::cli::{UsersArgs, UsersCommand};
use crate::graphql::client::GraphqlClient;
use crate::graphql::queries;
use crate::models::UsersResponse;
use crate::utils::error::CliError;
use crate::utils::output;

pub async fn execute(client: &GraphqlClient, args: UsersArgs) -> Result<(), CliError> {
    match args.command {
        UsersCommand::List => list(client).await,
    }
}

async fn list(client: &GraphqlClient) -> Result<(), CliError> {
    let response: UsersResponse = client
        .request(queries::USERS_LIST, serde_json::json!({}))
        .await?;
    output::print_json(&response.users.nodes);
    Ok(())
}
