pub mod comments;
pub mod cycles;
pub mod documents;
pub mod embeds;
pub mod issues;
pub mod labels;
pub mod project_milestones;
pub mod projects;
pub mod teams;
pub mod usage;
pub mod users;

use crate::graphql::client::GraphqlClient;
use crate::graphql::queries;
use crate::models::{
    CyclesResponse, IssuesResponse, ProjectMilestonesResponse, ProjectsResponse, TeamsResponse,
};
use crate::utils::error::CliError;
use crate::utils::identifiers::{is_uuid, parse_issue_identifier};

/// Resolve a team key, name, or ID to a team UUID.
pub async fn resolve_team_id(client: &GraphqlClient, team: &str) -> Result<String, CliError> {
    if is_uuid(team) {
        return Ok(team.to_string());
    }

    // Try as team key first (short alphanumeric strings)
    let response: TeamsResponse = client
        .request(
            queries::RESOLVE_TEAM_BY_KEY,
            serde_json::json!({ "key": team }),
        )
        .await?;

    if let Some(t) = response.teams.nodes.first() {
        return Ok(t.id.clone());
    }

    // Try as team name
    let response: TeamsResponse = client
        .request(
            queries::RESOLVE_TEAM_BY_NAME,
            serde_json::json!({ "name": team }),
        )
        .await?;

    response
        .teams
        .nodes
        .first()
        .map(|t| t.id.clone())
        .ok_or_else(|| CliError::NotFound {
            entity: "Team".to_string(),
            identifier: team.to_string(),
        })
}

/// Resolve a project name or ID to a project UUID.
pub async fn resolve_project_id(client: &GraphqlClient, project: &str) -> Result<String, CliError> {
    if is_uuid(project) {
        return Ok(project.to_string());
    }

    let response: ProjectsResponse = client
        .request(
            queries::RESOLVE_PROJECT_BY_NAME,
            serde_json::json!({ "name": project }),
        )
        .await?;

    response
        .projects
        .nodes
        .first()
        .map(|p| p.id.clone())
        .ok_or_else(|| CliError::NotFound {
            entity: "Project".to_string(),
            identifier: project.to_string(),
        })
}

/// Resolve an issue identifier (UUID or ABC-123) to an issue UUID.
pub async fn resolve_issue_id(client: &GraphqlClient, issue_id: &str) -> Result<String, CliError> {
    if is_uuid(issue_id) {
        return Ok(issue_id.to_string());
    }

    if let Some((team_key, number)) = parse_issue_identifier(issue_id) {
        let response: IssuesResponse = client
            .request(
                queries::RESOLVE_ISSUE_BY_IDENTIFIER,
                serde_json::json!({
                    "teamKey": team_key,
                    "number": number
                }),
            )
            .await?;

        return response
            .issues
            .nodes
            .first()
            .map(|i| i.id.clone())
            .ok_or_else(|| CliError::NotFound {
                entity: "Issue".to_string(),
                identifier: issue_id.to_string(),
            });
    }

    Err(CliError::InvalidParameter {
        param: "issue_id".to_string(),
        reason: "Must be a UUID or identifier like ABC-123".to_string(),
    })
}

/// Resolve a cycle name or ID to a cycle UUID.
pub async fn resolve_cycle_id(
    client: &GraphqlClient,
    team_id: &str,
    cycle: &str,
) -> Result<String, CliError> {
    if is_uuid(cycle) {
        return Ok(cycle.to_string());
    }

    let response: CyclesResponse = client
        .request(
            queries::RESOLVE_CYCLE_BY_NAME,
            serde_json::json!({
                "teamId": team_id,
                "name": cycle
            }),
        )
        .await?;

    response
        .cycles
        .nodes
        .first()
        .map(|c| c.id.clone())
        .ok_or_else(|| CliError::NotFound {
            entity: "Cycle".to_string(),
            identifier: cycle.to_string(),
        })
}

/// Resolve a milestone name or ID to a milestone UUID.
pub async fn resolve_milestone_id(
    client: &GraphqlClient,
    project_id: &str,
    milestone: &str,
) -> Result<String, CliError> {
    if is_uuid(milestone) {
        return Ok(milestone.to_string());
    }

    let response: ProjectMilestonesResponse = client
        .request(
            queries::RESOLVE_MILESTONE_BY_NAME,
            serde_json::json!({
                "projectId": project_id,
                "name": milestone
            }),
        )
        .await?;

    response
        .project_milestones
        .nodes
        .first()
        .map(|m| m.id.clone())
        .ok_or_else(|| CliError::NotFound {
            entity: "Project milestone".to_string(),
            identifier: milestone.to_string(),
        })
}
