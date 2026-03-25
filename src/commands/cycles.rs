use crate::cli::{CyclesArgs, CyclesCommand};
use crate::commands::resolve_team_id;
use crate::graphql::client::GraphqlClient;
use crate::graphql::queries;
use crate::models::{CycleCreateResponse, CycleUpdateResponse, CyclesResponse, SingleCycleResponse};
use crate::utils::error::CliError;
use crate::utils::identifiers::is_uuid;
use crate::utils::output;

pub async fn execute(client: &GraphqlClient, args: CyclesArgs) -> Result<(), CliError> {
    match args.command {
        CyclesCommand::List {
            team,
            active,
            around_active,
        } => list(client, team, active, around_active).await,
        CyclesCommand::Create {
            team,
            name,
            starts_at,
            ends_at,
            description,
        } => create(client, &team, name, &starts_at, &ends_at, description).await,
        CyclesCommand::Update {
            cycle_id_or_name,
            team,
            name,
            starts_at,
            ends_at,
            description,
        } => {
            update(
                client,
                &cycle_id_or_name,
                team,
                name,
                starts_at,
                ends_at,
                description,
            )
            .await
        }
        CyclesCommand::Read {
            cycle_id_or_name,
            team,
            issues_first,
        } => read(client, &cycle_id_or_name, team, issues_first).await,
    }
}

async fn list(
    client: &GraphqlClient,
    team: Option<String>,
    active: bool,
    around_active: Option<u32>,
) -> Result<(), CliError> {
    let mut filter = serde_json::Map::new();

    if let Some(ref team_val) = team {
        let team_id = resolve_team_id(client, team_val).await?;
        filter.insert(
            "team".to_string(),
            serde_json::json!({ "id": { "eq": team_id } }),
        );
    }

    if active {
        filter.insert(
            "isActive".to_string(),
            serde_json::json!({ "eq": true }),
        );
    }

    let variables = serde_json::json!({
        "first": 250,
        "filter": if filter.is_empty() { serde_json::Value::Null } else { serde_json::Value::Object(filter) }
    });

    let response: CyclesResponse = client.request(queries::CYCLES_LIST, variables).await?;
    let mut cycles = response.cycles.nodes;

    // Handle --around-active: find the active cycle and return +/- n around it
    if let Some(n) = around_active {
        let now = chrono::Utc::now();
        let active_idx = cycles.iter().position(|c| {
            let started = c
                .starts_at
                .as_deref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|d| d <= now)
                .unwrap_or(false);
            let not_ended = c
                .ends_at
                .as_deref()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|d| d >= now)
                .unwrap_or(false);
            started && not_ended
        });
        if let Some(idx) = active_idx {
            let start = idx.saturating_sub(n as usize);
            let end = (idx + 1 + n as usize).min(cycles.len());
            cycles = cycles[start..end].to_vec();
        }
    }

    output::print_json(&cycles);
    Ok(())
}

async fn create(
    client: &GraphqlClient,
    team: &str,
    name: Option<String>,
    starts_at: &str,
    ends_at: &str,
    description: Option<String>,
) -> Result<(), CliError> {
    let team_id = resolve_team_id(client, team).await?;

    let mut input = serde_json::json!({
        "teamId": team_id,
        "startsAt": starts_at,
        "endsAt": ends_at,
    });

    if let Some(n) = name {
        input["name"] = serde_json::json!(n);
    }
    if let Some(d) = description {
        input["description"] = serde_json::json!(d);
    }

    let response: CycleCreateResponse = client
        .request(
            queries::CYCLE_CREATE,
            serde_json::json!({ "input": input }),
        )
        .await?;

    if response.cycle_create.success {
        output::print_json(&response.cycle_create.cycle);
    } else {
        return Err(CliError::Other("Failed to create cycle".to_string()));
    }
    Ok(())
}

async fn update(
    client: &GraphqlClient,
    cycle_id_or_name: &str,
    team: Option<String>,
    name: Option<String>,
    starts_at: Option<String>,
    ends_at: Option<String>,
    description: Option<String>,
) -> Result<(), CliError> {
    let cycle_id = if is_uuid(cycle_id_or_name) {
        cycle_id_or_name.to_string()
    } else {
        let team_val = team.as_deref().ok_or_else(|| CliError::RequiresParameter {
            flag: "cycle name".to_string(),
            required: "--team".to_string(),
        })?;
        let team_id = resolve_team_id(client, team_val).await?;
        crate::commands::resolve_cycle_id(client, &team_id, cycle_id_or_name).await?
    };

    let mut input = serde_json::Map::new();

    if let Some(n) = name {
        input.insert("name".to_string(), serde_json::json!(n));
    }
    if let Some(sa) = starts_at {
        input.insert("startsAt".to_string(), serde_json::json!(sa));
    }
    if let Some(ea) = ends_at {
        input.insert("endsAt".to_string(), serde_json::json!(ea));
    }
    if let Some(d) = description {
        input.insert("description".to_string(), serde_json::json!(d));
    }

    let response: CycleUpdateResponse = client
        .request(
            queries::CYCLE_UPDATE,
            serde_json::json!({ "id": cycle_id, "input": input }),
        )
        .await?;

    if response.cycle_update.success {
        output::print_json(&response.cycle_update.cycle);
    } else {
        return Err(CliError::Other("Failed to update cycle".to_string()));
    }
    Ok(())
}

async fn read(
    client: &GraphqlClient,
    cycle_id_or_name: &str,
    team: Option<String>,
    issues_first: u32,
) -> Result<(), CliError> {
    let cycle_id = if is_uuid(cycle_id_or_name) {
        cycle_id_or_name.to_string()
    } else {
        // Need team to resolve by name
        let team_val = team.as_deref().ok_or_else(|| CliError::RequiresParameter {
            flag: "cycle name".to_string(),
            required: "--team".to_string(),
        })?;
        let team_id = resolve_team_id(client, team_val).await?;
        crate::commands::resolve_cycle_id(client, &team_id, cycle_id_or_name).await?
    };

    let query = queries::cycle_read_query();
    let response: SingleCycleResponse = client
        .request(
            &query,
            serde_json::json!({
                "id": cycle_id,
                "issuesFirst": issues_first
            }),
        )
        .await?;

    output::print_json(&response.cycle);
    Ok(())
}
