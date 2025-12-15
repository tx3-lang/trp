use schemars::schema::{InstanceType, SchemaObject};

use super::{map_primitive, map_reference, sanitize_identifier, LanguageContext};
use crate::resolver::{any_of, array_item_schema, object_additional_properties, one_of};

pub fn type_name(raw: &str) -> String {
    sanitize_identifier(raw)
}

pub fn field_name(raw: &str) -> String {
    let mut out = String::new();
    for (i, ch) in raw.chars().enumerate() {
        if ch.is_ascii_alphanumeric() {
            if i == 0 {
                out.push(ch.to_ascii_lowercase());
            } else {
                out.push(ch);
            }
        } else if !out.ends_with('_') {
            out.push('_');
        }
    }
    if out.is_empty() {
        "field".to_string()
    } else {
        out
    }
}

pub fn map_type(schema: &SchemaObject, ctx: &LanguageContext) -> String {
    if let Some(reference) = map_reference(schema, ctx) {
        return reference;
    }

    if let Some(subschemas) = &schema.subschemas {
        if let Some(options) = one_of(subschemas) {
            let joined: Vec<String> = options.iter().map(|s| map_type(s, ctx)).collect();
            return format!("std::vec::Vec<{}>", joined.join(" | "));
        }
        if let Some(options) = any_of(subschemas) {
            let joined: Vec<String> = options.iter().map(|s| map_type(s, ctx)).collect();
            return format!("std::vec::Vec<{}>", joined.join(" | "));
        }
    }

    if let Some(InstanceType::Array) = map_primitive(schema) {
        if let Some(array) = &schema.array {
            if let Some(item) = array_item_schema(array) {
                return format!("Vec<{}>", map_type(&item, ctx));
            }
        }
        return "Vec<serde_json::Value>".to_string();
    }

    if let Some(InstanceType::Object) = map_primitive(schema) {
        if let Some(object) = &schema.object {
            if let Some(additional) = object_additional_properties(object) {
                return format!(
                    "std::collections::HashMap<String, {}>",
                    map_type(&additional, ctx)
                );
            }
        }
        return "std::collections::HashMap<String, serde_json::Value>".to_string();
    }

    match map_primitive(schema) {
        Some(InstanceType::String) => "String".to_string(),
        Some(InstanceType::Integer) => "i64".to_string(),
        Some(InstanceType::Number) => "f64".to_string(),
        Some(InstanceType::Boolean) => "bool".to_string(),
        Some(InstanceType::Null) => "Option<serde_json::Value>".to_string(),
        _ => "serde_json::Value".to_string(),
    }
}
