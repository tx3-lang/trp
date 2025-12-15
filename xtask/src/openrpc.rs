use std::collections::HashMap;

use schemars::schema::Schema;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct OpenRpc {
    pub openrpc: String,
    pub info: Option<Info>,
    pub methods: Vec<Method>,
    pub components: Option<Components>,
}

#[derive(Deserialize, Debug)]
pub struct Info {
    pub title: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct Method {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct Components {
    pub schemas: Option<HashMap<String, Schema>>,
}
