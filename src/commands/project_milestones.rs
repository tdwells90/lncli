use crate::cli::{ProjectMilestonesArgs, ProjectMilestonesCommand};
use crate::commands::{resolve_milestone_id, resolve_project_id};
use crate::graphql::client::GraphqlClient;
use crate::graphql::queries;
use crate::models::{
    ProjectMilestoneCreateResponse, ProjectMilestoneDeleteResponse,
    ProjectMilestoneUpdateResponse, ProjectMilestonesResponse, SingleProjectMilestoneResponse,
};
use crate::utils::error::CliError;
use crate::utils::identifiers::is_uuid;
use crate::utils::output;
use crate::utils::stdin;

pub async fn execute(
    client: &GraphqlClient,
    args: ProjectMilestonesArgs,
) -> Result<(), CliError> {
    match args.command {
        ProjectMilestonesCommand::List { project, limit } => {
            list(client, &project, limit).await
        }
        ProjectMilestonesCommand::Read {
            milestone_id_or_name,
            project,
            issues_first,
        } => read(client, &milestone_id_or_name, project, issues_first).await,
        ProjectMilestonesCommand::Delete {
            milestone_id_or_name,
            project,
        } => delete(client, &milestone_id_or_name, project).await,
        ProjectMilestonesCommand::Create {
            name,
            project,
            description,
            target_date,
        } => {
            let description = stdin::resolve_optional(description)?;
            create(client, &name, &project, description, target_date).await
        }
        ProjectMilestonesCommand::Update {
            milestone_id_or_name,
            project,
            name,
            description,
            target_date,
            sort_order,
        } => {
            let description = stdin::resolve_optional(description)?;
            update(
                client,
                &milestone_id_or_name,
                project,
                name,
                description,
                target_date,
                sort_order,
            )
            .await
        }
    }
}

async fn list(client: &GraphqlClient, project: &str, limit: u32) -> Result<(), CliError> {
    let project_id = resolve_project_id(client, project).await?;

    let response: ProjectMilestonesResponse = client
        .request(
            queries::PROJECT_MILESTONES_LIST,
            serde_json::json!({
                "projectId": project_id,
                "first": limit
            }),
        )
        .await?;

    output::print_json(&response.project_milestones.nodes);
    Ok(())
}

async fn read(
    client: &GraphqlClient,
    milestone_id_or_name: &str,
    project: Option<String>,
    issues_first: u32,
) -> Result<(), CliError> {
    let milestone_id = if is_uuid(milestone_id_or_name) {
        milestone_id_or_name.to_string()
    } else {
        let project_val = project.as_deref().ok_or_else(|| CliError::RequiresParameter {
            flag: "milestone name".to_string(),
            required: "--project".to_string(),
        })?;
        let project_id = resolve_project_id(client, project_val).await?;
        resolve_milestone_id(client, &project_id, milestone_id_or_name).await?
    };

    let query = queries::project_milestone_read_query();
    let response: SingleProjectMilestoneResponse = client
        .request(
            &query,
            serde_json::json!({
                "id": milestone_id,
                "issuesFirst": issues_first
            }),
        )
        .await?;

    output::print_json(&response.project_milestone);
    Ok(())
}

async fn delete(
    client: &GraphqlClient,
    milestone_id_or_name: &str,
    project: Option<String>,
) -> Result<(), CliError> {
    let milestone_id = if is_uuid(milestone_id_or_name) {
        milestone_id_or_name.to_string()
    } else {
        let project_val = project.as_deref().ok_or_else(|| CliError::RequiresParameter {
            flag: "milestone name".to_string(),
            required: "--project".to_string(),
        })?;
        let project_id = resolve_project_id(client, project_val).await?;
        resolve_milestone_id(client, &project_id, milestone_id_or_name).await?
    };

    let response: ProjectMilestoneDeleteResponse = client
        .request(
            queries::PROJECT_MILESTONE_DELETE,
            serde_json::json!({ "id": milestone_id }),
        )
        .await?;

    if response.project_milestone_delete.success {
        output::print_json(&serde_json::json!({ "success": true, "id": milestone_id }));
    } else {
        return Err(CliError::Other(
            "Failed to delete project milestone".to_string(),
        ));
    }
    Ok(())
}

async fn create(
    client: &GraphqlClient,
    name: &str,
    project: &str,
    description: Option<String>,
    target_date: Option<String>,
) -> Result<(), CliError> {
    let project_id = resolve_project_id(client, project).await?;

    let mut input = serde_json::json!({
        "name": name,
        "projectId": project_id,
    });

    if let Some(desc) = description {
        input["description"] = serde_json::json!(desc);
    }
    if let Some(date) = target_date {
        input["targetDate"] = serde_json::json!(date);
    }

    let response: ProjectMilestoneCreateResponse = client
        .request(
            queries::PROJECT_MILESTONE_CREATE,
            serde_json::json!({ "input": input }),
        )
        .await?;

    if response.project_milestone_create.success {
        output::print_json(&response.project_milestone_create.project_milestone);
    } else {
        return Err(CliError::Other(
            "Failed to create project milestone".to_string(),
        ));
    }
    Ok(())
}

async fn update(
    client: &GraphqlClient,
    milestone_id_or_name: &str,
    project: Option<String>,
    name: Option<String>,
    description: Option<String>,
    target_date: Option<String>,
    sort_order: Option<f64>,
) -> Result<(), CliError> {
    let milestone_id = if is_uuid(milestone_id_or_name) {
        milestone_id_or_name.to_string()
    } else {
        let project_val = project.as_deref().ok_or_else(|| CliError::RequiresParameter {
            flag: "milestone name".to_string(),
            required: "--project".to_string(),
        })?;
        let project_id = resolve_project_id(client, project_val).await?;
        resolve_milestone_id(client, &project_id, milestone_id_or_name).await?
    };

    let mut input = serde_json::Map::new();

    if let Some(n) = name {
        input.insert("name".to_string(), serde_json::json!(n));
    }
    if let Some(d) = description {
        input.insert("description".to_string(), serde_json::json!(d));
    }
    if let Some(td) = target_date {
        input.insert("targetDate".to_string(), serde_json::json!(td));
    }
    if let Some(so) = sort_order {
        input.insert("sortOrder".to_string(), serde_json::json!(so));
    }

    let response: ProjectMilestoneUpdateResponse = client
        .request(
            queries::PROJECT_MILESTONE_UPDATE,
            serde_json::json!({ "id": milestone_id, "input": input }),
        )
        .await?;

    if response.project_milestone_update.success {
        output::print_json(&response.project_milestone_update.project_milestone);
    } else {
        return Err(CliError::Other(
            "Failed to update project milestone".to_string(),
        ));
    }
    Ok(())
}
