use schemars::schema::{InstanceType, SchemaObject};

use super::{map_primitive, map_reference, sanitize_identifier, LanguageContext};
use crate::resolver::{any_of, array_item_schema, object_additional_properties, one_of};

pub fn type_name(raw: &str) -> String {
    sanitize_identifier(raw)
}

pub fn field_name(raw: &str) -> String {
    let mut ident = sanitize_identifier(raw);
    if let Some(first) = ident.get_mut(0..1) {
        first.make_ascii_uppercase();
    }
    ident
}

pub fn map_type(schema: &SchemaObject, ctx: &LanguageContext) -> String {
    if let Some(reference) = map_reference(schema, ctx) {
        return reference;
    }

    if let Some(subschemas) = &schema.subschemas {
        if one_of(subschemas).is_some() || any_of(subschemas).is_some() {
            return "interface{}".to_string();
        }
    }

    if let Some(InstanceType::Array) = map_primitive(schema) {
        if let Some(array) = &schema.array {
            if let Some(item) = array_item_schema(array) {
                return format!("[]{}", map_type(&item, ctx));
            }
        }
        return "[]interface{}".to_string();
    }

    if let Some(InstanceType::Object) = map_primitive(schema) {
        if let Some(object) = &schema.object {
            if let Some(additional) = object_additional_properties(object) {
                return format!("map[string]{}", map_type(&additional, ctx));
            }
        }
        return "map[string]interface{}".to_string();
    }

    match map_primitive(schema) {
        Some(InstanceType::String) => "string".to_string(),
        Some(InstanceType::Integer) => "int64".to_string(),
        Some(InstanceType::Number) => "float64".to_string(),
        Some(InstanceType::Boolean) => "bool".to_string(),
        Some(InstanceType::Null) => "interface{}".to_string(),
        _ => "interface{}".to_string(),
    }
}
