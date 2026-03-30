use crate::cli::{CommentsArgs, CommentsCommand};
use crate::commands::resolve_issue_id;
use crate::graphql::client::GraphqlClient;
use crate::graphql::queries;
use crate::models::{CommentCreateResponse, CommentDeleteResponse, CommentUpdateResponse};
use crate::utils::error::CliError;
use crate::utils::output;
use crate::utils::stdin;

pub async fn execute(client: &GraphqlClient, args: CommentsArgs) -> Result<(), CliError> {
    match args.command {
        CommentsCommand::Create { issue_id, body } => {
            let body = stdin::resolve_value(body).await?;
            create(client, &issue_id, &body).await
        }
        CommentsCommand::Update { comment_id, body } => {
            let body = stdin::resolve_value(body).await?;
            update(client, &comment_id, &body).await
        }
        CommentsCommand::Delete { comment_id } => delete(client, &comment_id).await,
    }
}

async fn create(client: &GraphqlClient, issue_id: &str, body: &str) -> Result<(), CliError> {
    let resolved_id = resolve_issue_id(client, issue_id).await?;

    let input = serde_json::json!({
        "issueId": resolved_id,
        "body": body,
    });

    let response: CommentCreateResponse = client
        .request(
            queries::COMMENT_CREATE,
            serde_json::json!({ "input": input }),
        )
        .await?;

    if response.comment_create.success {
        output::print_json(&response.comment_create.comment);
    } else {
        return Err(CliError::Other("Failed to create comment".to_string()));
    }
    Ok(())
}

async fn update(client: &GraphqlClient, comment_id: &str, body: &str) -> Result<(), CliError> {
    let input = serde_json::json!({ "body": body });

    let response: CommentUpdateResponse = client
        .request(
            queries::COMMENT_UPDATE,
            serde_json::json!({ "id": comment_id, "input": input }),
        )
        .await?;

    if response.comment_update.success {
        output::print_json(&response.comment_update.comment);
    } else {
        return Err(CliError::Other("Failed to update comment".to_string()));
    }
    Ok(())
}

async fn delete(client: &GraphqlClient, comment_id: &str) -> Result<(), CliError> {
    let response: CommentDeleteResponse = client
        .request(
            queries::COMMENT_DELETE,
            serde_json::json!({ "id": comment_id }),
        )
        .await?;

    if response.comment_delete.success {
        output::print_json(&serde_json::json!({ "success": true, "id": comment_id }));
    } else {
        return Err(CliError::Other("Failed to delete comment".to_string()));
    }
    Ok(())
}
