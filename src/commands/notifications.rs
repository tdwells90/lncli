use crate::cli::{NotificationsArgs, NotificationsCommand};
use crate::graphql::client::GraphqlClient;
use crate::graphql::queries;
use crate::models::{NotificationUpdateResponse, NotificationsResponse};
use crate::utils::error::CliError;
use crate::utils::fields::{self, filter_json_nodes};
use crate::utils::output;
use chrono::Utc;

const NOTIFICATION_MANDATORY_FIELDS: &[&str] = &["id"];

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
                mark_read(client, &notification_id.unwrap()).await
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
    let raw_response: serde_json::Value = client
        .request_raw(
            queries::NOTIFICATIONS_LIST,
            serde_json::json!({ "first": limit }),
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

    let notifications = if unread {
        response
            .notifications
            .nodes
            .into_iter()
            .filter(|n| n.read_at.is_none())
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
            serde_json::json!({ "first": 250 }),
        )
        .await?;

    let response: NotificationsResponse = serde_json::from_value(
        serde_json::json!({ "notifications": raw_response["notifications"] }),
    )?;

    let unread: Vec<_> = response
        .notifications
        .nodes
        .iter()
        .filter(|n| n.read_at.is_none())
        .collect();

    if unread.is_empty() {
        output::print_json(&serde_json::json!({
            "success": true,
            "message": "No unread notifications",
            "count": 0
        }));
        return Ok(());
    }

    let now = Utc::now().to_rfc3339();
    let mut marked_count = 0u32;

    for notification in &unread {
        let input = serde_json::json!({ "readAt": now });
        let result: NotificationUpdateResponse = client
            .request(
                queries::NOTIFICATION_UPDATE,
                serde_json::json!({ "id": notification.id, "input": input }),
            )
            .await?;

        if result.notification_update.success {
            marked_count += 1;
        }
    }

    output::print_json(&serde_json::json!({
        "success": true,
        "count": marked_count
    }));
    Ok(())
}
