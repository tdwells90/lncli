use crate::cli::{LabelsArgs, LabelsCommand};
use crate::graphql::client::GraphqlClient;
use crate::graphql::queries;
use crate::models::{
    LabelCreateResponse, LabelDeleteResponse, LabelOutput, LabelUpdateResponse, LabelsResponse,
};
use crate::utils::error::CliError;
use crate::utils::identifiers::is_uuid;
use crate::utils::output;

pub async fn execute(client: &GraphqlClient, args: LabelsArgs) -> Result<(), CliError> {
    match args.command {
        LabelsCommand::List { team } => list(client, team).await,
        LabelsCommand::Create {
            name,
            color,
            team,
            parent,
        } => create(client, &name, color, team, parent).await,
        LabelsCommand::Update {
            label_id,
            name,
            color,
        } => update(client, &label_id, name, color).await,
        LabelsCommand::Delete { label_id } => delete(client, &label_id).await,
    }
}

async fn list(client: &GraphqlClient, team: Option<String>) -> Result<(), CliError> {
    let response: LabelsResponse = if let Some(ref team_filter) = team {
        let team_id = crate::commands::resolve_team_id(client, team_filter).await?;
        client
            .request(
                queries::LABELS_LIST_BY_TEAM,
                serde_json::json!({ "teamId": team_id }),
            )
            .await?
    } else {
        client
            .request(queries::LABELS_LIST, serde_json::json!({}))
            .await?
    };

    // Filter out group containers, transform to output format
    let labels: Vec<LabelOutput> = response
        .issue_labels
        .nodes
        .into_iter()
        .filter(|l| !l.is_group.unwrap_or(false))
        .map(|l| {
            let scope = if l.team.is_some() {
                "team".to_string()
            } else {
                "workspace".to_string()
            };
            let group = l.parent.map(|p| p.name);
            LabelOutput {
                id: l.id,
                name: l.name,
                color: l.color,
                scope,
                team: l.team,
                group,
            }
        })
        .collect();

    output::print_json(&labels);
    Ok(())
}

async fn create(
    client: &GraphqlClient,
    name: &str,
    color: Option<String>,
    team: Option<String>,
    parent: Option<String>,
) -> Result<(), CliError> {
    let mut input = serde_json::json!({ "name": name });

    if let Some(c) = color {
        input["color"] = serde_json::json!(c);
    }
    if let Some(ref team_val) = team {
        let team_id = crate::commands::resolve_team_id(client, team_val).await?;
        input["teamId"] = serde_json::json!(team_id);
    }
    if let Some(ref parent_val) = parent {
        let parent_id = resolve_label_id(client, parent_val).await?;
        input["parentId"] = serde_json::json!(parent_id);
    }

    let response: LabelCreateResponse = client
        .request(
            queries::LABEL_CREATE,
            serde_json::json!({ "input": input }),
        )
        .await?;

    if response.issue_label_create.success {
        output::print_json(&response.issue_label_create.issue_label);
    } else {
        return Err(CliError::Other("Failed to create label".to_string()));
    }
    Ok(())
}

async fn update(
    client: &GraphqlClient,
    label_id: &str,
    name: Option<String>,
    color: Option<String>,
) -> Result<(), CliError> {
    let mut input = serde_json::Map::new();

    if let Some(n) = name {
        input.insert("name".to_string(), serde_json::json!(n));
    }
    if let Some(c) = color {
        input.insert("color".to_string(), serde_json::json!(c));
    }

    let response: LabelUpdateResponse = client
        .request(
            queries::LABEL_UPDATE,
            serde_json::json!({ "id": label_id, "input": input }),
        )
        .await?;

    if response.issue_label_update.success {
        output::print_json(&response.issue_label_update.issue_label);
    } else {
        return Err(CliError::Other("Failed to update label".to_string()));
    }
    Ok(())
}

async fn delete(client: &GraphqlClient, label_id: &str) -> Result<(), CliError> {
    let response: LabelDeleteResponse = client
        .request(
            queries::LABEL_DELETE,
            serde_json::json!({ "id": label_id }),
        )
        .await?;

    if response.issue_label_delete.success {
        output::print_json(&serde_json::json!({ "success": true, "id": label_id }));
    } else {
        return Err(CliError::Other("Failed to delete label".to_string()));
    }
    Ok(())
}

/// Resolve a label name or UUID to an ID (for parent label lookup).
async fn resolve_label_id(
    client: &GraphqlClient,
    label: &str,
) -> Result<String, CliError> {
    if is_uuid(label) {
        return Ok(label.to_string());
    }

    let response: LabelsResponse = client
        .request(queries::LABELS_LIST, serde_json::json!({}))
        .await?;

    response
        .issue_labels
        .nodes
        .iter()
        .find(|l| l.name.eq_ignore_ascii_case(label))
        .map(|l| l.id.clone())
        .ok_or_else(|| CliError::NotFound {
            entity: "Label".to_string(),
            identifier: label.to_string(),
        })
}
