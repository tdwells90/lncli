use crate::cli::{IssuesArgs, IssuesCommand, LabelMode};
use crate::commands::{resolve_issue_id, resolve_project_id, resolve_team_id};
use crate::graphql::client::GraphqlClient;
use crate::graphql::queries;
use crate::models::{
    Issue, IssueCreateResponse, IssueDeleteResponse, IssueSearchResponse, IssueUpdateResponse,
    IssuesResponse, SingleIssueResponse,
};
use crate::utils::embed_parser;
use crate::utils::error::CliError;
use crate::utils::fields::{self, filter_json_nodes};
use crate::utils::identifiers::{is_uuid, parse_issue_identifier};
use crate::utils::output;
use crate::utils::stdin;

const ISSUE_MANDATORY_FIELDS: &[&str] = &["id", "identifier"];

pub async fn execute(
    client: &GraphqlClient,
    args: IssuesArgs,
    fields_filter: Option<&str>,
) -> Result<(), CliError> {
    match args.command {
        IssuesCommand::List { limit, project } => {
            list(client, limit, project, fields_filter).await
        }
        IssuesCommand::Read { issue_id, project } => {
            read(client, &issue_id, project, fields_filter).await
        }
        IssuesCommand::Search {
            query,
            team,
            assignee,
            project,
            status,
            limit,
        } => {
            search(
                client,
                &query,
                team,
                assignee,
                project,
                status,
                limit,
                fields_filter,
            )
            .await
        }
        IssuesCommand::Delete { issue_id } => delete(client, &issue_id).await,
        IssuesCommand::Create {
            title,
            description,
            assignee,
            priority,
            project,
            team,
            labels,
            project_milestone,
            cycle,
            status,
            parent_ticket,
        } => {
            let description = stdin::resolve_optional(description).await?;
            create(
                client,
                &title,
                description,
                assignee,
                priority,
                project,
                &team,
                labels,
                project_milestone,
                cycle,
                status,
                parent_ticket,
            )
            .await
        }
        IssuesCommand::Update {
            issue_id,
            title,
            description,
            status,
            priority,
            assignee,
            project,
            labels,
            label_by,
            clear_labels,
            parent_ticket,
            clear_parent_ticket,
            project_milestone,
            clear_project_milestone,
            cycle,
            clear_cycle,
        } => {
            let description = stdin::resolve_optional(description).await?;
            update(
                client,
                &issue_id,
                title,
                description,
                status,
                priority,
                assignee,
                project,
                labels,
                label_by,
                clear_labels,
                parent_ticket,
                clear_parent_ticket,
                project_milestone,
                clear_project_milestone,
                cycle,
                clear_cycle,
            )
            .await
        }
    }
}

async fn delete(client: &GraphqlClient, issue_id: &str) -> Result<(), CliError> {
    let resolved_id = resolve_issue_id(client, issue_id).await?;

    let response: IssueDeleteResponse = client
        .request(
            queries::ISSUE_DELETE,
            serde_json::json!({ "id": resolved_id }),
        )
        .await?;

    if response.issue_delete.success {
        output::print_json(&serde_json::json!({ "success": true, "id": resolved_id }));
    } else {
        return Err(CliError::Other("Failed to delete issue".to_string()));
    }
    Ok(())
}

fn add_embeds(issue: &mut Issue) {
    if let Some(ref desc) = issue.description {
        let embeds = embed_parser::extract_embeds(desc);
        if !embeds.is_empty() {
            issue.embeds = Some(embeds);
        }
    }
}

async fn list(
    client: &GraphqlClient,
    limit: u32,
    project: Option<String>,
    fields_filter: Option<&str>,
) -> Result<(), CliError> {
    let query = queries::issues_list_query();

    let filter = if let Some(ref project_val) = project {
        let project_id = resolve_project_id(client, project_val).await?;
        serde_json::json!({ "project": { "id": { "eq": project_id } } })
    } else {
        serde_json::Value::Null
    };

    let raw_response: serde_json::Value = client
        .request_raw(
            &query,
            serde_json::json!({ "first": limit, "filter": filter }),
        )
        .await?;

    let issues_value = if let Some(filter_str) = fields_filter {
        let fields = fields::parse_fields(filter_str);
        filter_json_nodes(&raw_response["issues"], &fields, ISSUE_MANDATORY_FIELDS)
    } else {
        raw_response["issues"].clone()
    };

    let response: IssuesResponse =
        serde_json::from_value(serde_json::json!({ "issues": issues_value }))?;

    let mut issues = response.issues.nodes;
    for issue in &mut issues {
        add_embeds(issue);
    }

    output::print_json(&issues);
    Ok(())
}

async fn read(
    client: &GraphqlClient,
    issue_id: &str,
    project: Option<String>,
    fields_filter: Option<&str>,
) -> Result<(), CliError> {
    let (raw_response, is_uuid_path) = if is_uuid(issue_id) {
        if project.is_some() {
            return Err(CliError::InvalidParameter {
                param: "project".to_string(),
                reason: "--project is not supported when reading by UUID".to_string(),
            });
        }
        let query = queries::issue_read_by_id_query();
        let resp: serde_json::Value = client
            .request_raw(&query, serde_json::json!({ "id": issue_id }))
            .await?;
        (resp, true)
    } else if let Some((team_key, number)) = parse_issue_identifier(issue_id) {
        let query = queries::issue_read_by_identifier_query();
        let mut filter = serde_json::json!({
            "team": { "key": { "eq": team_key } },
            "number": { "eq": number }
        });
        if let Some(ref project_val) = project {
            let project_id = resolve_project_id(client, project_val).await?;
            filter
                .as_object_mut()
                .expect("filter is a JSON object")
                .insert(
                    "project".to_string(),
                    serde_json::json!({ "id": { "eq": project_id } }),
                );
        }
        let resp: serde_json::Value = client
            .request_raw(&query, serde_json::json!({ "filter": filter }))
            .await?;
        (resp, false)
    } else {
        return Err(CliError::InvalidParameter {
            param: "issue_id".to_string(),
            reason: "Must be a UUID or identifier like ABC-123".to_string(),
        });
    };

    let issue_value = if is_uuid_path {
        if let Some(filter_str) = fields_filter {
            let fields = fields::parse_fields(filter_str);
            filter_json_nodes(&raw_response["issue"], &fields, ISSUE_MANDATORY_FIELDS)
        } else {
            raw_response["issue"].clone()
        }
    } else {
        if let Some(filter_str) = fields_filter {
            let fields = fields::parse_fields(filter_str);
            filter_json_nodes(&raw_response["issues"], &fields, ISSUE_MANDATORY_FIELDS)
        } else {
            raw_response["issues"].clone()
        }
    };

    let mut issue = if is_uuid_path {
        let response: SingleIssueResponse =
            serde_json::from_value(serde_json::json!({ "issue": issue_value }))?;
        response.issue
    } else {
        let response: IssuesResponse =
            serde_json::from_value(serde_json::json!({ "issues": issue_value }))?;
        response
            .issues
            .nodes
            .into_iter()
            .next()
            .ok_or_else(|| CliError::NotFound {
                entity: "Issue".to_string(),
                identifier: issue_id.to_string(),
            })?
    };
    add_embeds(&mut issue);
    output::print_json(&issue);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn search(
    client: &GraphqlClient,
    query: &str,
    team: Option<String>,
    assignee: Option<String>,
    project: Option<String>,
    status: Option<String>,
    limit: u32,
    fields_filter: Option<&str>,
) -> Result<(), CliError> {
    let mut filter = serde_json::Map::new();

    if let Some(ref team_val) = team {
        let team_id = resolve_team_id(client, team_val).await?;
        filter.insert(
            "team".to_string(),
            serde_json::json!({ "id": { "eq": team_id } }),
        );
    }

    if let Some(ref assignee_val) = assignee {
        filter.insert(
            "assignee".to_string(),
            serde_json::json!({ "id": { "eq": assignee_val } }),
        );
    }

    if let Some(ref project_val) = project {
        let project_id = resolve_project_id(client, project_val).await?;
        filter.insert(
            "project".to_string(),
            serde_json::json!({ "id": { "eq": project_id } }),
        );
    }

    if let Some(ref status_val) = status {
        let statuses: Vec<&str> = status_val.split(',').map(|s| s.trim()).collect();
        filter.insert(
            "state".to_string(),
            serde_json::json!({ "name": { "in": statuses } }),
        );
    }

    let variables = serde_json::json!({
        "term": query,
        "first": limit,
        "filter": if filter.is_empty() { serde_json::Value::Null } else { serde_json::Value::Object(filter) }
    });

    let gql_query = queries::issues_search_query();
    let raw_response: serde_json::Value = client.request_raw(&gql_query, variables).await?;

    let issues_value = if let Some(filter_str) = fields_filter {
        let fields = fields::parse_fields(filter_str);
        filter_json_nodes(
            &raw_response["searchIssues"],
            &fields,
            ISSUE_MANDATORY_FIELDS,
        )
    } else {
        raw_response["searchIssues"].clone()
    };

    let response: IssueSearchResponse =
        serde_json::from_value(serde_json::json!({ "searchIssues": issues_value }))?;

    let mut issues = response.search_issues.nodes;
    for issue in &mut issues {
        add_embeds(issue);
    }

    output::print_json(&issues);
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn create(
    client: &GraphqlClient,
    title: &str,
    description: Option<String>,
    assignee: Option<String>,
    priority: Option<u8>,
    project: Option<String>,
    team: &str,
    labels: Option<String>,
    project_milestone: Option<String>,
    cycle: Option<String>,
    status: Option<String>,
    parent_ticket: Option<String>,
) -> Result<(), CliError> {
    // Resolve team
    let team_id = resolve_team_id(client, team).await?;

    let mut input = serde_json::json!({
        "title": title,
        "teamId": team_id,
    });

    if let Some(desc) = description {
        input["description"] = serde_json::json!(desc);
    }
    if let Some(ref assignee_val) = assignee {
        input["assigneeId"] = serde_json::json!(assignee_val);
    }
    if let Some(p) = priority {
        input["priority"] = serde_json::json!(p);
    }
    if let Some(ref project_val) = project {
        let project_id = resolve_project_id(client, project_val).await?;
        input["projectId"] = serde_json::json!(project_id);
    }
    if let Some(ref labels_val) = labels {
        let label_ids = resolve_label_ids(client, labels_val, Some(&team_id)).await?;
        input["labelIds"] = serde_json::json!(label_ids);
    }
    if let Some(ref status_val) = status {
        let state_id = resolve_status_id(client, &team_id, status_val).await?;
        input["stateId"] = serde_json::json!(state_id);
    }
    if let Some(ref milestone_val) = project_milestone {
        if project.is_none() {
            return Err(CliError::RequiresParameter {
                flag: "--project-milestone".to_string(),
                required: "--project".to_string(),
            });
        }
        let project_id = input["projectId"].as_str().unwrap().to_string();
        let milestone_id =
            crate::commands::resolve_milestone_id(client, &project_id, milestone_val).await?;
        input["projectMilestoneId"] = serde_json::json!(milestone_id);
    }
    if let Some(ref cycle_val) = cycle {
        let cycle_id = crate::commands::resolve_cycle_id(client, &team_id, cycle_val).await?;
        input["cycleId"] = serde_json::json!(cycle_id);
    }
    if let Some(ref parent_val) = parent_ticket {
        let parent_id = resolve_issue_id(client, parent_val).await?;
        input["parentId"] = serde_json::json!(parent_id);
    }

    let response: IssueCreateResponse = client
        .request(queries::ISSUE_CREATE, serde_json::json!({ "input": input }))
        .await?;

    if response.issue_create.success {
        output::print_json(&response.issue_create.issue);
    } else {
        return Err(CliError::Other("Failed to create issue".to_string()));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn update(
    client: &GraphqlClient,
    issue_id: &str,
    title: Option<String>,
    description: Option<String>,
    status: Option<String>,
    priority: Option<u8>,
    assignee: Option<String>,
    project: Option<String>,
    labels: Option<String>,
    label_by: LabelMode,
    clear_labels: bool,
    parent_ticket: Option<String>,
    clear_parent_ticket: bool,
    project_milestone: Option<String>,
    clear_project_milestone: bool,
    cycle: Option<String>,
    clear_cycle: bool,
) -> Result<(), CliError> {
    let resolved_id = resolve_issue_id(client, issue_id).await?;

    let mut input = serde_json::Map::new();

    if let Some(t) = title {
        input.insert("title".to_string(), serde_json::json!(t));
    }
    if let Some(d) = description {
        input.insert("description".to_string(), serde_json::json!(d));
    }
    if let Some(p) = priority {
        input.insert("priority".to_string(), serde_json::json!(p));
    }
    if let Some(ref a) = assignee {
        input.insert("assigneeId".to_string(), serde_json::json!(a));
    }
    if let Some(ref project_val) = project {
        let project_id = resolve_project_id(client, project_val).await?;
        input.insert("projectId".to_string(), serde_json::json!(project_id));
    }

    // Fetch the issue once if any field needs it (status, labels, milestone, cycle)
    let needs_issue = status.is_some()
        || (labels.is_some() && !clear_labels)
        || (project_milestone.is_some() && project.is_none())
        || cycle.is_some();

    let issue_data = if needs_issue {
        let issue_query = queries::issue_read_by_id_query();
        let issue_resp: SingleIssueResponse = client
            .request(&issue_query, serde_json::json!({ "id": &resolved_id }))
            .await?;
        Some(issue_resp.issue)
    } else {
        None
    };

    // Status resolution requires the team
    if let Some(ref status_val) = status {
        let team = issue_data
            .as_ref()
            .and_then(|i| i.team.as_ref())
            .ok_or_else(|| CliError::Other("Issue has no team".to_string()))?;
        let team_id = team
            .id
            .as_deref()
            .ok_or_else(|| CliError::Other("Issue team has no id".to_string()))?;
        let state_id = resolve_status_id(client, team_id, status_val).await?;
        input.insert("stateId".to_string(), serde_json::json!(state_id));
    }

    // Label resolution
    if let Some(ref labels_val) = labels {
        if !clear_labels {
            let team_id = issue_data
                .as_ref()
                .and_then(|i| i.team.as_ref())
                .and_then(|t| t.id.as_deref());
            let new_label_ids = resolve_label_ids(client, labels_val, team_id).await?;

            match label_by {
                LabelMode::Adding => {
                    let mut all_ids: Vec<String> = issue_data
                        .as_ref()
                        .and_then(|i| i.labels.as_ref())
                        .map(|l| l.nodes.iter().map(|n| n.id.clone()).collect())
                        .unwrap_or_default();
                    for id in new_label_ids {
                        if !all_ids.contains(&id) {
                            all_ids.push(id);
                        }
                    }
                    input.insert("labelIds".to_string(), serde_json::json!(all_ids));
                }
                LabelMode::Overwriting => {
                    input.insert("labelIds".to_string(), serde_json::json!(new_label_ids));
                }
            }
        }
    }

    if clear_labels {
        input.insert("labelIds".to_string(), serde_json::json!([]));
    }

    // Parent ticket
    if let Some(ref parent_val) = parent_ticket {
        let parent_id = resolve_issue_id(client, parent_val).await?;
        input.insert("parentId".to_string(), serde_json::json!(parent_id));
    }
    if clear_parent_ticket {
        input.insert("parentId".to_string(), serde_json::Value::Null);
    }

    // Project milestone
    if let Some(ref milestone_val) = project_milestone {
        let project_id = if let Some(ref p) = project {
            resolve_project_id(client, p).await?
        } else if let Some(pid) = input.get("projectId").and_then(|v| v.as_str()) {
            pid.to_string()
        } else {
            issue_data
                .as_ref()
                .and_then(|i| i.project.as_ref())
                .and_then(|p| p.id.clone())
                .ok_or_else(|| CliError::RequiresParameter {
                    flag: "--project-milestone".to_string(),
                    required: "--project".to_string(),
                })?
        };
        let milestone_id =
            crate::commands::resolve_milestone_id(client, &project_id, milestone_val).await?;
        input.insert(
            "projectMilestoneId".to_string(),
            serde_json::json!(milestone_id),
        );
    }
    if clear_project_milestone {
        input.insert("projectMilestoneId".to_string(), serde_json::Value::Null);
    }

    // Cycle
    if let Some(ref cycle_val) = cycle {
        let team_id = issue_data
            .as_ref()
            .and_then(|i| i.team.as_ref())
            .and_then(|t| t.id.clone())
            .ok_or_else(|| CliError::Other("Issue has no team".to_string()))?;
        let cycle_id = crate::commands::resolve_cycle_id(client, &team_id, cycle_val).await?;
        input.insert("cycleId".to_string(), serde_json::json!(cycle_id));
    }
    if clear_cycle {
        input.insert("cycleId".to_string(), serde_json::Value::Null);
    }

    // Guard against no-op updates
    if input.is_empty() {
        return Err(CliError::InvalidParameter {
            param: "update".to_string(),
            reason: "No fields to update. Provide at least one flag.".to_string(),
        });
    }

    let response: IssueUpdateResponse = client
        .request(
            queries::ISSUE_UPDATE,
            serde_json::json!({ "id": resolved_id, "input": input }),
        )
        .await?;

    if response.issue_update.success {
        output::print_json(&response.issue_update.issue);
    } else {
        return Err(CliError::Other("Failed to update issue".to_string()));
    }
    Ok(())
}

async fn resolve_label_ids(
    client: &GraphqlClient,
    labels_csv: &str,
    team_id: Option<&str>,
) -> Result<Vec<String>, CliError> {
    let label_names: Vec<&str> = labels_csv.split(',').map(|s| s.trim()).collect();
    let mut ids = Vec::new();

    // Fetch team-scoped labels if team_id is provided
    let team_labels: Option<crate::models::LabelsResponse> = if let Some(tid) = team_id {
        Some(
            client
                .request(
                    queries::LABELS_LIST_BY_TEAM,
                    serde_json::json!({ "teamId": tid }),
                )
                .await?,
        )
    } else {
        None
    };

    // Fetch workspace labels once (lazily, only if needed)
    let mut ws_labels: Option<crate::models::LabelsResponse> = None;

    for name in label_names {
        if is_uuid(name) {
            ids.push(name.to_string());
            continue;
        }

        // Try team-scoped labels first
        let found_in_team = team_labels.as_ref().and_then(|tl| {
            tl.issue_labels
                .nodes
                .iter()
                .find(|l| l.name.as_deref().is_some_and(|n| n.eq_ignore_ascii_case(name)))
        });

        if let Some(label) = found_in_team {
            ids.push(label.id.clone());
            continue;
        }

        // Fall back to workspace labels (fetch once)
        if ws_labels.is_none() {
            ws_labels = Some(
                client
                    .request(queries::LABELS_LIST, serde_json::json!({}))
                    .await?,
            );
        }

        let found_in_ws = ws_labels
            .as_ref()
            .unwrap()
            .issue_labels
            .nodes
            .iter()
            .find(|l| l.name.as_deref().is_some_and(|n| n.eq_ignore_ascii_case(name)));

        match found_in_ws {
            Some(label) => ids.push(label.id.clone()),
            None => {
                return Err(CliError::NotFound {
                    entity: "Label".to_string(),
                    identifier: name.to_string(),
                });
            }
        }
    }

    Ok(ids)
}

async fn resolve_status_id(
    client: &GraphqlClient,
    team_id: &str,
    status: &str,
) -> Result<String, CliError> {
    if is_uuid(status) {
        return Ok(status.to_string());
    }

    let response: crate::models::WorkflowStatesResponse = client
        .request(
            queries::RESOLVE_WORKFLOW_STATES,
            serde_json::json!({ "teamId": team_id }),
        )
        .await?;

    response
        .workflow_states
        .nodes
        .iter()
        .find(|s| s.name.as_deref().is_some_and(|n| n.eq_ignore_ascii_case(status)))
        .map(|s| s.id.clone())
        .ok_or_else(|| CliError::NotFound {
            entity: "Status".to_string(),
            identifier: status.to_string(),
        })
}
