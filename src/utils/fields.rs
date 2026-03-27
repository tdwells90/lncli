use serde_json::Value;
use std::collections::HashMap;

pub fn parse_fields(fields_str: &str) -> Vec<String> {
    fields_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect()
}

fn group_fields_by_top_level(fields: &[String]) -> HashMap<&str, Vec<&str>> {
    let mut groups: HashMap<&str, Vec<&str>> = HashMap::new();
    for field in fields {
        if let Some((first, rest)) = field.split_once('.') {
            groups.entry(first).or_default().push(rest);
        } else {
            groups.entry(field).or_default();
        }
    }
    groups
}

fn filter_json_recursive(value: &Value, allowed_fields: &[String]) -> Value {
    match value {
        Value::Object(obj) => {
            let grouped = group_fields_by_top_level(allowed_fields);
            let mut filtered = serde_json::Map::new();

            for (key, nested_fields) in grouped {
                if nested_fields.is_empty() {
                    if let Some(v) = obj.get(key) {
                        if !v.is_null() {
                            filtered.insert(key.to_string(), v.clone());
                        }
                    }
                } else if let Some(v) = obj.get(key) {
                    // Auto-include "id" in nested objects so deserialization
                    // doesn't fail when the struct requires it.
                    let mut nested_with_id: Vec<String> = nested_fields
                        .iter()
                        .map(|s| (*s).to_string())
                        .collect();
                    if !nested_with_id.iter().any(|f| f == "id") {
                        nested_with_id.push("id".to_string());
                    }
                    let nested_filtered = filter_json_recursive(
                        v,
                        &nested_with_id,
                    );
                    if !nested_filtered.is_null() && !nested_filtered.is_object()
                        || !nested_filtered.as_object().map_or(false, |o| o.is_empty())
                    {
                        filtered.insert(key.to_string(), nested_filtered);
                    }
                }
            }
            Value::Object(filtered)
        }
        Value::Array(arr) => {
            let filtered: Vec<Value> = arr
                .iter()
                .map(|v| filter_json_recursive(v, allowed_fields))
                .filter(|v| !v.is_null())
                .collect();
            if filtered.is_empty() && !arr.is_empty() {
                Value::Null
            } else {
                Value::Array(filtered)
            }
        }
        _ => value.clone(),
    }
}

pub fn filter_json_nodes(
    value: &Value,
    allowed_fields: &[String],
    mandatory_fields: &[&str],
) -> Value {
    let mut all_fields: Vec<String> = allowed_fields.to_vec();
    for mf in mandatory_fields {
        if !all_fields.iter().any(|f| f == *mf) {
            all_fields.push((*mf).to_string());
        }
    }

    if let Value::Object(obj) = value {
        if let Some(Value::Array(arr)) = obj.get("nodes") {
            let filtered_nodes: Vec<Value> = arr
                .iter()
                .map(|v| filter_json_recursive(v, &all_fields))
                .collect();
            let mut result = obj.clone();
            result.insert("nodes".to_string(), Value::Array(filtered_nodes));
            return Value::Object(result);
        }
    }
    filter_json_recursive(value, &all_fields)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_fields() {
        assert_eq!(parse_fields("id,title"), vec!["id", "title"]);
        assert_eq!(
            parse_fields("user.name,description,title"),
            vec!["user.name", "description", "title"]
        );
        assert_eq!(
            parse_fields(" id , title , description "),
            vec!["id", "title", "description"]
        );
    }

    #[test]
    fn test_filter_json_basic() {
        let value = json!({"id": "1", "title": "Test", "description": "Desc"});
        let allowed = vec!["id".to_string(), "title".to_string()];
        let mandatory = vec!["id"];

        let result = filter_json_nodes(&value, &allowed, &mandatory);
        assert_eq!(result, json!({"id": "1", "title": "Test"}));
    }

    #[test]
    fn test_filter_json_nested() {
        let value = json!({
            "id": "1",
            "title": "Test",
            "user": {"id": "u1", "name": "John", "email": "john@example.com"}
        });
        let allowed = vec!["id".to_string(), "user.name".to_string()];
        let mandatory = vec!["id"];

        let result = filter_json_nodes(&value, &allowed, &mandatory);
        // id is auto-included in nested objects for deserialization
        assert_eq!(
            result,
            json!({"id": "1", "user": {"id": "u1", "name": "John"}})
        );
    }

    #[test]
    fn test_filter_json_nested_multiple_fields_same_parent() {
        let value = json!({
            "id": "1",
            "title": "Test",
            "user": {"id": "u1", "name": "John", "email": "john@example.com", "age": 30}
        });
        let allowed = vec![
            "id".to_string(),
            "user.name".to_string(),
            "user.email".to_string(),
        ];
        let mandatory = vec!["id"];

        let result = filter_json_nodes(&value, &allowed, &mandatory);
        assert_eq!(
            result,
            json!({"id": "1", "user": {"id": "u1", "name": "John", "email": "john@example.com"}})
        );
    }

    #[test]
    fn test_filter_json_nested_auto_includes_id_for_deserialization() {
        // Reproduces the bug: --fields state.name should include state.id
        let value = json!({
            "id": "issue1",
            "identifier": "ENG-123",
            "title": "Test issue",
            "state": {"id": "state1", "name": "Done", "type": "completed"}
        });
        let allowed = vec!["state.name".to_string()];
        let mandatory = vec!["id", "identifier"];

        let result = filter_json_nodes(&value, &allowed, &mandatory);
        assert_eq!(
            result,
            json!({"id": "issue1", "identifier": "ENG-123", "state": {"id": "state1", "name": "Done"}})
        );
    }

    #[test]
    fn test_filter_json_nested_without_id_field() {
        // Nested object has no id field - should still work fine
        let value = json!({
            "id": "1",
            "meta": {"color": "red", "size": 10}
        });
        let allowed = vec!["id".to_string(), "meta.color".to_string()];
        let mandatory = vec!["id"];

        let result = filter_json_nodes(&value, &allowed, &mandatory);
        assert_eq!(result, json!({"id": "1", "meta": {"color": "red"}}));
    }

    #[test]
    fn test_filter_json_mandatory_always_included() {
        let value = json!({"id": "1", "title": "Test"});
        let allowed = vec!["title".to_string()];
        let mandatory = vec!["id"];

        let result = filter_json_nodes(&value, &allowed, &mandatory);
        assert_eq!(result, json!({"id": "1", "title": "Test"}));
    }

    #[test]
    fn test_filter_json_nodes() {
        let value = json!({
            "nodes": [
                {"id": "1", "title": "Test1", "description": "Desc1"},
                {"id": "2", "title": "Test2", "description": "Desc2"}
            ]
        });
        let allowed = vec!["id".to_string(), "title".to_string()];
        let mandatory = vec!["id"];

        let result = filter_json_nodes(&value, &allowed, &mandatory);
        assert_eq!(
            result,
            json!({
                "nodes": [
                    {"id": "1", "title": "Test1"},
                    {"id": "2", "title": "Test2"}
                ]
            })
        );
    }

    #[test]
    fn test_filter_json_nodes_nested_with_multiple_fields() {
        let value = json!({
            "nodes": [
                {
                    "id": "1",
                    "title": "Test1",
                    "user": {"id": "u1", "name": "John", "email": "john@example.com"}
                }
            ]
        });
        let allowed = vec![
            "id".to_string(),
            "user.name".to_string(),
            "user.email".to_string(),
        ];
        let mandatory = vec!["id"];

        let result = filter_json_nodes(&value, &allowed, &mandatory);
        assert_eq!(
            result,
            json!({
                "nodes": [
                    {"id": "1", "user": {"id": "u1", "name": "John", "email": "john@example.com"}}
                ]
            })
        );
    }
}
