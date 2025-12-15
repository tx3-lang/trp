use schemars::schema::{InstanceType, SchemaObject};

use super::{map_primitive, map_reference, sanitize_identifier, LanguageContext};
use crate::resolver::{any_of, array_item_schema, object_additional_properties, one_of};

pub fn type_name(raw: &str) -> String {
    sanitize_identifier(raw)
}

pub fn field_name(raw: &str) -> String {
    let mut chars = raw.chars();
    match chars.next() {
        Some(first) => first.to_ascii_lowercase().to_string() + chars.as_str(),
        None => "field".to_string(),
    }
}

pub fn map_type(schema: &SchemaObject, ctx: &LanguageContext) -> String {
    if let Some(reference) = map_reference(schema, ctx) {
        return reference;
    }

    if let Some(subschemas) = &schema.subschemas {
        if let Some(options) = one_of(subschemas) {
            let joined: Vec<String> = options.iter().map(|s| map_type(s, ctx)).collect();
            return joined.join(" | ");
        }
        if let Some(options) = crate::resolver::any_of(subschemas) {
            let joined: Vec<String> = options.iter().map(|s| map_type(s, ctx)).collect();
            return joined.join(" | ");
        }
    }

    if let Some(enum_values) = &schema.enum_values {
        let variants: Vec<String> = enum_values
            .iter()
            .filter_map(|v| v.as_str().map(|s| format!("\"{}\"", s)))
            .collect();
        if !variants.is_empty() {
            return variants.join(" | ");
        }
    }

    if let Some(InstanceType::Array) = map_primitive(schema) {
        if let Some(array) = &schema.array {
            if let Some(item) = array_item_schema(array) {
                return format!("{}[]", map_type(&item, ctx));
            }
        }
        return "any[]".to_string();
    }

    if let Some(InstanceType::Object) = map_primitive(schema) {
        if let Some(object) = &schema.object {
            if let Some(additional) = object_additional_properties(object) {
                return format!("Record<string, {}>", map_type(&additional, ctx));
            }
        }
        return "Record<string, any>".to_string();
    }

    match map_primitive(schema) {
        Some(InstanceType::String) => "string".to_string(),
        Some(InstanceType::Integer) | Some(InstanceType::Number) => "number".to_string(),
        Some(InstanceType::Boolean) => "boolean".to_string(),
        Some(InstanceType::Null) => "null".to_string(),
        _ => "any".to_string(),
    }
}
