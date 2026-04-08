use crate::cli::{NotificationsArgs, NotificationsCommand};
use crate::graphql::client::GraphqlClient;
use crate::graphql::queries;
use crate::models::{NotificationUpdateResponse, NotificationsResponse};
use crate::utils::error::CliError;
use crate::utils::fields::{self, filter_json_nodes};
use crate::utils::output;
use chrono::Utc;

const NOTIFICATION_MANDATORY_FIELDS: &[&str] = &["id"];
const MARK_ALL_READ_CAP: u32 = 250;

pub async fn execute(
    client: &GraphqlClient,
    args: NotificationsArgs,
    fields_filter: Option<&str>,
) -> Result<(), CliError> {
    match args.command {
        NotificationsCommand::List { limit, unread } => {
            list(client, limit, unread, fields_filter).await
        }
        NotificationsCommand::MarkRead {
            notification_id,
            all,
        } => {
            if all {
                mark_all_read(client).await
            } else {
                mark_read(
                    client,
                    &notification_id.expect("clap guarantees notification_id when --all is absent"),
                )
                .await
            }
        }
    }
}

async fn list(
    client: &GraphqlClient,
    limit: u32,
    unread: bool,
    fields_filter: Option<&str>,
) -> Result<(), CliError> {
    // When filtering to unread, over-fetch so we can return up to `limit`
    // unread notifications even when many recent notifications are already read.
    let fetch_count = if unread { limit * 5 } else { limit };

    let raw_response: serde_json::Value = client
        .request_raw(
            queries::NOTIFICATIONS_LIST,
            serde_json::json!({ "first": fetch_count }),
        )
        .await?;

    let notifications_value = if let Some(filter_str) = fields_filter {
        let parsed_fields = fields::parse_fields(filter_str);
        filter_json_nodes(
            &raw_response["notifications"],
            &parsed_fields,
            NOTIFICATION_MANDATORY_FIELDS,
        )
    } else {
        raw_response["notifications"].clone()
    };

    let response: NotificationsResponse =
        serde_json::from_value(serde_json::json!({ "notifications": notifications_value }))?;

    let notifications: Vec<_> = if unread {
        response
            .notifications
            .nodes
            .into_iter()
            .filter(|n| n.read_at.is_none())
            .take(limit as usize)
            .collect()
    } else {
        response.notifications.nodes
    };

    output::print_json(&notifications);
    Ok(())
}

async fn mark_read(client: &GraphqlClient, notification_id: &str) -> Result<(), CliError> {
    let now = Utc::now().to_rfc3339();
    let input = serde_json::json!({ "readAt": now });

    let response: NotificationUpdateResponse = client
        .request(
            queries::NOTIFICATION_UPDATE,
            serde_json::json!({ "id": notification_id, "input": input }),
        )
        .await?;

    if response.notification_update.success {
        output::print_json(&response.notification_update.notification);
    } else {
        return Err(CliError::Other(
            "Failed to mark notification as read".to_string(),
        ));
    }
    Ok(())
}

async fn mark_all_read(client: &GraphqlClient) -> Result<(), CliError> {
    let raw_response: serde_json::Value = client
        .request_raw(
            queries::NOTIFICATIONS_LIST,
            serde_json::json!({ "first": MARK_ALL_READ_CAP }),
        )
        .await?;

    let response: NotificationsResponse = serde_json::from_value(
        serde_json::json!({ "notifications": raw_response["notifications"] }),
    )?;

    let total_fetched = response.notifications.nodes.len() as u32;

    let unread_ids: Vec<String> = response
        .notifications
        .nodes
        .iter()
        .filter(|n| n.read_at.is_none())
        .map(|n| n.id.clone())
        .collect();

    if unread_ids.is_empty() {
        output::print_json(&serde_json::json!({
            "success": true,
            "message": "No unread notifications",
            "count": 0
        }));
        return Ok(());
    }

    let now = Utc::now().to_rfc3339();

    let mut handles = Vec::with_capacity(unread_ids.len());
    for id in &unread_ids {
        let input = serde_json::json!({ "readAt": now });
        let vars = serde_json::json!({ "id": id, "input": input });
        handles
            .push(client.request::<NotificationUpdateResponse>(queries::NOTIFICATION_UPDATE, vars));
    }

    let results = futures::future::join_all(handles).await;
    let marked_count = results
        .iter()
        .filter(|r| {
            r.as_ref()
                .is_ok_and(|resp| resp.notification_update.success)
        })
        .count() as u32;

    let capped = total_fetched >= MARK_ALL_READ_CAP;
    output::print_json(&serde_json::json!({
        "success": true,
        "count": marked_count,
        "capped": capped
    }));
    Ok(())
}
