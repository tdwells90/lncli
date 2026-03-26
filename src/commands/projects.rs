use crate::cli::{ProjectsArgs, ProjectsCommand};
use crate::commands::{resolve_project_id, resolve_team_id};
use crate::graphql::client::GraphqlClient;
use crate::graphql::queries;
use crate::models::{
    ProjectCreateResponse, ProjectDeleteResponse, ProjectUpdateResponse, ProjectsResponse,
    SingleProjectResponse,
};
use crate::utils::error::CliError;
use crate::utils::output;
use crate::utils::stdin;

pub async fn execute(client: &GraphqlClient, args: ProjectsArgs) -> Result<(), CliError> {
    match args.command {
        ProjectsCommand::List { limit } => list(client, limit).await,
        ProjectsCommand::Read {
            project_id_or_name,
        } => read(client, &project_id_or_name).await,
        ProjectsCommand::Create {
            name,
            teams,
            description,
            content,
            lead,
            priority,
            start_date,
            target_date,
            icon,
            color,
        } => {
            stdin::validate_at_most_one_stdin(&[
                ("--description", description.as_deref()),
                ("--content", content.as_deref()),
            ])?;
            let description = stdin::resolve_optional(description)?;
            let content = stdin::resolve_optional(content)?;
            create(
                client,
                &name,
                &teams,
                description,
                content,
                lead,
                priority,
                start_date,
                target_date,
                icon,
                color,
            )
            .await
        }
        ProjectsCommand::Update {
            project_id_or_name,
            name,
            description,
            content,
            lead,
            priority,
            start_date,
            target_date,
            icon,
            color,
            teams,
        } => {
            stdin::validate_at_most_one_stdin(&[
                ("--description", description.as_deref()),
                ("--content", content.as_deref()),
            ])?;
            let description = stdin::resolve_optional(description)?;
            let content = stdin::resolve_optional(content)?;
            update(
                client,
                &project_id_or_name,
                name,
                description,
                content,
                lead,
                priority,
                start_date,
                target_date,
                icon,
                color,
                teams,
            )
            .await
        }
        ProjectsCommand::Delete {
            project_id_or_name,
        } => delete(client, &project_id_or_name).await,
    }
}

async fn list(client: &GraphqlClient, limit: Option<u32>) -> Result<(), CliError> {
    let vars = if let Some(l) = limit {
        serde_json::json!({ "first": l })
    } else {
        serde_json::json!({})
    };
    let response: ProjectsResponse = client.request(queries::PROJECTS_LIST, vars).await?;
    output::print_json(&response.projects.nodes);
    Ok(())
}

async fn read(client: &GraphqlClient, project_id_or_name: &str) -> Result<(), CliError> {
    let project_id = resolve_project_id(client, project_id_or_name).await?;
    let response: SingleProjectResponse = client
        .request(
            queries::PROJECT_READ,
            serde_json::json!({ "id": project_id }),
        )
        .await?;
    output::print_json(&response.project);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn create(
    client: &GraphqlClient,
    name: &str,
    teams: &str,
    description: Option<String>,
    content: Option<String>,
    lead: Option<String>,
    priority: Option<u8>,
    start_date: Option<String>,
    target_date: Option<String>,
    icon: Option<String>,
    color: Option<String>,
) -> Result<(), CliError> {
    let team_ids = resolve_team_ids(client, teams).await?;

    let mut input = serde_json::json!({
        "name": name,
        "teamIds": team_ids,
    });

    if let Some(d) = description {
        input["description"] = serde_json::json!(d);
    }
    if let Some(c) = content {
        input["content"] = serde_json::json!(c);
    }
    if let Some(l) = lead {
        input["leadId"] = serde_json::json!(l);
    }
    if let Some(p) = priority {
        input["priority"] = serde_json::json!(p);
    }
    if let Some(sd) = start_date {
        input["startDate"] = serde_json::json!(sd);
    }
    if let Some(td) = target_date {
        input["targetDate"] = serde_json::json!(td);
    }
    if let Some(i) = icon {
        input["icon"] = serde_json::json!(i);
    }
    if let Some(c) = color {
        input["color"] = serde_json::json!(c);
    }

    let response: ProjectCreateResponse = client
        .request(
            queries::PROJECT_CREATE,
            serde_json::json!({ "input": input }),
        )
        .await?;

    if response.project_create.success {
        output::print_json(&response.project_create.project);
    } else {
        return Err(CliError::Other("Failed to create project".to_string()));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn update(
    client: &GraphqlClient,
    project_id_or_name: &str,
    name: Option<String>,
    description: Option<String>,
    content: Option<String>,
    lead: Option<String>,
    priority: Option<u8>,
    start_date: Option<String>,
    target_date: Option<String>,
    icon: Option<String>,
    color: Option<String>,
    teams: Option<String>,
) -> Result<(), CliError> {
    let project_id = resolve_project_id(client, project_id_or_name).await?;

    let mut input = serde_json::Map::new();

    if let Some(n) = name {
        input.insert("name".to_string(), serde_json::json!(n));
    }
    if let Some(d) = description {
        input.insert("description".to_string(), serde_json::json!(d));
    }
    if let Some(c) = content {
        input.insert("content".to_string(), serde_json::json!(c));
    }
    if let Some(l) = lead {
        input.insert("leadId".to_string(), serde_json::json!(l));
    }
    if let Some(p) = priority {
        input.insert("priority".to_string(), serde_json::json!(p));
    }
    if let Some(sd) = start_date {
        input.insert("startDate".to_string(), serde_json::json!(sd));
    }
    if let Some(td) = target_date {
        input.insert("targetDate".to_string(), serde_json::json!(td));
    }
    if let Some(i) = icon {
        input.insert("icon".to_string(), serde_json::json!(i));
    }
    if let Some(c) = color {
        input.insert("color".to_string(), serde_json::json!(c));
    }
    if let Some(ref t) = teams {
        let team_ids = resolve_team_ids(client, t).await?;
        input.insert("teamIds".to_string(), serde_json::json!(team_ids));
    }

    let response: ProjectUpdateResponse = client
        .request(
            queries::PROJECT_UPDATE,
            serde_json::json!({ "id": project_id, "input": input }),
        )
        .await?;

    if response.project_update.success {
        output::print_json(&response.project_update.project);
    } else {
        return Err(CliError::Other("Failed to update project".to_string()));
    }
    Ok(())
}

async fn delete(client: &GraphqlClient, project_id_or_name: &str) -> Result<(), CliError> {
    let project_id = resolve_project_id(client, project_id_or_name).await?;

    let response: ProjectDeleteResponse = client
        .request(
            queries::PROJECT_DELETE,
            serde_json::json!({ "id": project_id }),
        )
        .await?;

    if response.project_delete.success {
        output::print_json(&serde_json::json!({ "success": true, "id": project_id }));
    } else {
        return Err(CliError::Other("Failed to delete project".to_string()));
    }
    Ok(())
}

/// Resolve comma-separated team keys/names/IDs to a Vec of team UUIDs.
async fn resolve_team_ids(client: &GraphqlClient, teams_csv: &str) -> Result<Vec<String>, CliError> {
    let mut ids = Vec::new();
    for team in teams_csv.split(',').map(|s| s.trim()) {
        let id = resolve_team_id(client, team).await?;
        ids.push(id);
    }
    Ok(ids)
}
