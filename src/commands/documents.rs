use crate::cli::{DocumentsArgs, DocumentsCommand};
use crate::commands::{resolve_issue_id, resolve_project_id, resolve_team_id};
use crate::graphql::client::GraphqlClient;
use crate::graphql::queries;
use crate::models::{
    AttachmentCreateResponse, AttachmentsResponse, Document, DocumentCreateResponse,
    DocumentDeleteResponse, DocumentUpdateResponse, DocumentsResponse, SingleDocumentResponse,
};
use crate::utils::error::CliError;
use crate::utils::identifiers::extract_document_id;
use crate::utils::output;
use crate::utils::stdin;

pub async fn execute(client: &GraphqlClient, args: DocumentsArgs) -> Result<(), CliError> {
    match args.command {
        DocumentsCommand::Create {
            title,
            content,
            project,
            team,
            icon,
            color,
            attach_to,
        } => {
            let content = stdin::resolve_optional(content)?;
            create(client, &title, content, project, team, icon, color, attach_to).await
        }
        DocumentsCommand::Update {
            document_id,
            title,
            content,
            project,
            icon,
            color,
        } => {
            let content = stdin::resolve_optional(content)?;
            update(client, &document_id, title, content, project, icon, color).await
        }
        DocumentsCommand::Read { document_id } => read(client, &document_id).await,
        DocumentsCommand::List {
            project,
            issue,
            limit,
        } => list(client, project, issue, limit).await,
        DocumentsCommand::Delete { document_id } => delete(client, &document_id).await,
    }
}

#[allow(clippy::too_many_arguments)]
async fn create(
    client: &GraphqlClient,
    title: &str,
    content: Option<String>,
    project: Option<String>,
    team: Option<String>,
    icon: Option<String>,
    color: Option<String>,
    attach_to: Option<String>,
) -> Result<(), CliError> {
    let mut input = serde_json::json!({ "title": title });

    if let Some(c) = content {
        input["content"] = serde_json::json!(c);
    }
    if let Some(ref p) = project {
        let project_id = resolve_project_id(client, p).await?;
        input["projectId"] = serde_json::json!(project_id);
    }
    if let Some(ref t) = team {
        let team_id = resolve_team_id(client, t).await?;
        input["teamId"] = serde_json::json!(team_id);
    }
    if let Some(i) = icon {
        input["icon"] = serde_json::json!(i);
    }
    if let Some(c) = color {
        input["color"] = serde_json::json!(c);
    }

    let response: DocumentCreateResponse = client
        .request(
            queries::DOCUMENT_CREATE,
            serde_json::json!({ "input": input }),
        )
        .await?;

    if !response.document_create.success {
        return Err(CliError::Other("Failed to create document".to_string()));
    }

    let doc = response.document_create.document.as_ref();

    // Attach to issue if requested
    if let Some(ref issue_val) = attach_to {
        if let Some(doc) = doc {
            let issue_id = resolve_issue_id(client, issue_val).await?;
            let doc_url = format!(
                "https://linear.app/document/{}",
                &doc.id
            );
            let attach_input = serde_json::json!({
                "issueId": issue_id,
                "title": doc.title,
                "url": doc_url,
            });
            let _: AttachmentCreateResponse = client
                .request(
                    queries::ATTACHMENT_CREATE,
                    serde_json::json!({ "input": attach_input }),
                )
                .await?;
        }
    }

    output::print_json(&response.document_create.document);
    Ok(())
}

async fn update(
    client: &GraphqlClient,
    document_id: &str,
    title: Option<String>,
    content: Option<String>,
    project: Option<String>,
    icon: Option<String>,
    color: Option<String>,
) -> Result<(), CliError> {
    let resolved_id = extract_document_id(document_id);

    let mut input = serde_json::Map::new();

    if let Some(t) = title {
        input.insert("title".to_string(), serde_json::json!(t));
    }
    if let Some(c) = content {
        input.insert("content".to_string(), serde_json::json!(c));
    }
    if let Some(ref p) = project {
        let project_id = resolve_project_id(client, p).await?;
        input.insert("projectId".to_string(), serde_json::json!(project_id));
    }
    if let Some(i) = icon {
        input.insert("icon".to_string(), serde_json::json!(i));
    }
    if let Some(c) = color {
        input.insert("color".to_string(), serde_json::json!(c));
    }

    let response: DocumentUpdateResponse = client
        .request(
            queries::DOCUMENT_UPDATE,
            serde_json::json!({ "id": resolved_id, "input": input }),
        )
        .await?;

    if response.document_update.success {
        output::print_json(&response.document_update.document);
    } else {
        return Err(CliError::Other("Failed to update document".to_string()));
    }
    Ok(())
}

async fn read(client: &GraphqlClient, document_id: &str) -> Result<(), CliError> {
    let resolved_id = extract_document_id(document_id);

    let response: SingleDocumentResponse = client
        .request(
            queries::DOCUMENT_READ,
            serde_json::json!({ "id": resolved_id }),
        )
        .await?;

    output::print_json(&response.document);
    Ok(())
}

async fn list(
    client: &GraphqlClient,
    project: Option<String>,
    issue: Option<String>,
    limit: u32,
) -> Result<(), CliError> {
    if let Some(ref issue_val) = issue {
        // Special handling: get documents attached to an issue
        return list_by_issue(client, issue_val).await;
    }

    let mut filter = serde_json::Map::new();

    if let Some(ref p) = project {
        let project_id = resolve_project_id(client, p).await?;
        filter.insert(
            "project".to_string(),
            serde_json::json!({ "id": { "eq": project_id } }),
        );
    }

    let variables = serde_json::json!({
        "first": limit,
        "filter": if filter.is_empty() { serde_json::Value::Null } else { serde_json::Value::Object(filter) }
    });

    let response: DocumentsResponse = client.request(queries::DOCUMENTS_LIST, variables).await?;
    output::print_json(&response.documents.nodes);
    Ok(())
}

async fn list_by_issue(client: &GraphqlClient, issue_val: &str) -> Result<(), CliError> {
    let issue_id = resolve_issue_id(client, issue_val).await?;

    // Get attachments for the issue
    let response: AttachmentsResponse = client
        .request(
            queries::ATTACHMENTS_FOR_ISSUE,
            serde_json::json!({ "issueId": issue_id }),
        )
        .await?;

    // Extract document IDs from attachment URLs
    let mut documents: Vec<Document> = Vec::new();
    for attachment in &response.attachments.nodes {
        if let Some(ref url) = attachment.url {
            if url.contains("linear.app") && url.contains("/document/") {
                let doc_id = extract_document_id(url);
                if let Ok(doc_resp) = client
                    .request::<SingleDocumentResponse>(
                        queries::DOCUMENT_READ,
                        serde_json::json!({ "id": doc_id }),
                    )
                    .await
                {
                    documents.push(doc_resp.document);
                }
            }
        }
    }

    output::print_json(&documents);
    Ok(())
}

async fn delete(client: &GraphqlClient, document_id: &str) -> Result<(), CliError> {
    let resolved_id = extract_document_id(document_id);

    let response: DocumentDeleteResponse = client
        .request(
            queries::DOCUMENT_DELETE,
            serde_json::json!({ "id": resolved_id }),
        )
        .await?;

    if response.document_delete.success {
        output::print_json(&serde_json::json!({ "success": true, "id": resolved_id }));
    } else {
        return Err(CliError::Other("Failed to delete document".to_string()));
    }
    Ok(())
}
