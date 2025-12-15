use std::collections::HashMap;

use anyhow::{Context, Result};
use schemars::schema::{
    ArrayValidation, InstanceType, ObjectValidation, Schema, SchemaObject, SubschemaValidation,
};

#[derive(Clone, Debug)]
pub struct ResolvedType {
    pub name: String,
    pub schema: SchemaObject,
    pub fields: Vec<ResolvedField>,
}

#[derive(Clone, Debug)]
pub struct ResolvedField {
    pub name: String,
    pub schema: SchemaObject,
    pub required: bool,
}

pub fn resolve_components(spec: &OpenRpc) -> Result<Vec<ResolvedType>> {
    let components = spec
        .components
        .as_ref()
        .and_then(|c| c.schemas.as_ref())
        .context("no components.schemas present in OpenRPC spec")?;

    let mut resolved = Vec::new();
    for (name, schema) in components {
        let schema_obj = normalize_schema(schema);
        let fields = collect_fields(&schema_obj, components)?;
        resolved.push(ResolvedType {
            name: name.clone(),
            schema: schema_obj,
            fields,
        });
    }

    Ok(resolved)
}

fn normalize_schema(schema: &Schema) -> SchemaObject {
    match schema {
        Schema::Bool(_) => SchemaObject::default(),
        Schema::Object(obj) => obj.clone(),
    }
}

fn collect_fields(
    schema: &SchemaObject,
    components: &HashMap<String, Schema>,
) -> Result<Vec<ResolvedField>> {
    if let Some(reference) = &schema.reference {
        let target_name = ref_to_name(reference)?;
        let target_schema = components
            .get(&target_name)
            .with_context(|| format!("missing referenced schema {}", target_name))?;
        return collect_fields(&normalize_schema(target_schema), components);
    }

    if let Some(subschemas) = &schema.subschemas {
        if let Some(all_of) = &subschemas.all_of {
            let mut fields = Vec::new();
            for sub in all_of {
                fields.extend(collect_fields(&normalize_schema(sub), components)?);
            }
            return Ok(fields);
        }
    }

    if let Some(object) = &schema.object {
        return Ok(object
            .properties
            .iter()
            .map(|(name, schema)| ResolvedField {
                name: name.clone(),
                schema: normalize_schema(schema),
                required: object.required.contains(name),
            })
            .collect());
    }

    Ok(Vec::new())
}

pub fn ref_to_name(reference: &str) -> Result<String> {
    reference
        .split('/')
        .last()
        .map(|s| s.to_string())
        .context("invalid reference string")
}

// Helpers for mapping
pub fn array_item_schema(array: &ArrayValidation) -> Option<SchemaObject> {
    array.items.as_ref().and_then(|items| match items {
        schemars::schema::SingleOrVec::Single(schema) => Some(normalize_schema(schema)),
        schemars::schema::SingleOrVec::Vec(list) => list.first().map(normalize_schema),
    })
}

pub fn object_additional_properties(object: &ObjectValidation) -> Option<SchemaObject> {
    object
        .additional_properties
        .as_ref()
        .and_then(|props| match props {
            schemars::schema::AdditionalProperties::Schema(schema) => {
                Some(normalize_schema(schema))
            }
            schemars::schema::AdditionalProperties::Bool(true) => Some(SchemaObject {
                instance_type: Some(vec![InstanceType::Object].into()),
                ..Default::default()
            }),
            _ => None,
        })
}

pub fn one_of(subschemas: &SubschemaValidation) -> Option<Vec<SchemaObject>> {
    subschemas
        .one_of
        .as_ref()
        .map(|schemas| schemas.iter().map(normalize_schema).collect())
}

pub fn any_of(subschemas: &SubschemaValidation) -> Option<Vec<SchemaObject>> {
    subschemas
        .any_of
        .as_ref()
        .map(|schemas| schemas.iter().map(normalize_schema).collect())
}
