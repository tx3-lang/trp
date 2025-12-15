use std::{collections::HashMap, ops::Deref as _};

use schemars::schema::{InstanceType, SchemaObject};

use crate::resolver::{ref_to_name, ResolvedType};

pub mod go;
pub mod python;
pub mod rust;
pub mod ts;

#[derive(Debug)]
pub struct LanguageContext {
    type_names: HashMap<String, String>,
    language: String,
}

impl LanguageContext {
    pub fn type_name(&self, raw: &str) -> String {
        self.type_names
            .get(raw)
            .cloned()
            .unwrap_or_else(|| sanitize_identifier(raw))
    }

    pub fn wrap_optional(&self, ty: &str) -> String {
        match self.language.as_str() {
            "ts" | "typescript" => format!("{} | null", ty),
            "python" => format!("Optional[{}]", ty),
            "go" => format!("*{}", ty),
            "rust" => format!("Option<{}>", ty),
            _ => ty.to_string(),
        }
    }

    pub fn language(&self) -> &str {
        &self.language
    }
}

pub fn build_context(types: &[ResolvedType], lang: &str) -> LanguageContext {
    let mut type_names = HashMap::new();
    for ty in types {
        let name = match lang {
            "ts" | "typescript" => ts::type_name(&ty.name),
            "python" => python::type_name(&ty.name),
            "go" => go::type_name(&ty.name),
            "rust" => rust::type_name(&ty.name),
            _ => sanitize_identifier(&ty.name),
        };
        type_names.insert(ty.name.clone(), name);
    }

    LanguageContext {
        type_names,
        language: lang.to_string(),
    }
}

pub fn map_reference(schema: &SchemaObject, ctx: &LanguageContext) -> Option<String> {
    schema
        .reference
        .as_deref()
        .and_then(|r| ref_to_name(r).ok())
        .map(|n| ctx.type_name(&n))
}

pub fn map_primitive(schema: &SchemaObject) -> Option<InstanceType> {
    match &schema.instance_type {
        Some(types) => match types {
            schemars::schema::SingleOrVec::Single(t) => Some(t.deref().clone()),
            schemars::schema::SingleOrVec::Vec(list) => list.first().copied(),
        },
        None => None,
    }
}

pub fn sanitize_identifier(name: &str) -> String {
    let mut out = String::new();
    let mut capitalize = true;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            if capitalize {
                out.extend(ch.to_uppercase());
                capitalize = false;
            } else {
                out.push(ch);
            }
        } else {
            capitalize = true;
        }
    }

    if out.is_empty() {
        "Type".to_string()
    } else {
        out
    }
}
