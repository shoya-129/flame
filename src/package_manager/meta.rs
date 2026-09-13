use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlameParamMeta {
    pub name: String,
    pub type_name: String,
    #[serde(default)]
    pub is_callback: bool,
    #[serde(default)]
    pub is_ref: bool,
    #[serde(default)]
    pub is_mut: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlameFunctionMeta {
    pub name: String,
    #[serde(default)]
    pub flame_name: String,
    pub params: Vec<FlameParamMeta>,
    pub return_type: String,
    #[serde(default)]
    pub is_static: bool,
    #[serde(default)]
    pub is_generic: bool,
    #[serde(default)]
    pub is_async: bool,
    #[serde(default)]
    pub is_constructor: bool,
    #[serde(default)]
    pub persistent_runtime: bool,
    #[serde(default)]
    pub receiver: Option<String>,
    #[serde(default)]
    pub docs: Option<String>,
    #[serde(default)]
    pub requires: Vec<String>,
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlameStructFieldMeta {
    pub name: String,
    pub type_name: String,
    pub docs: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlameStructMeta {
    pub name: String,
    #[serde(default)]
    pub flame_name: String,
    pub methods: Vec<FlameFunctionMeta>,
    #[serde(default)]
    pub fields: Vec<FlameStructFieldMeta>,
    #[serde(default)]
    pub docs: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlameMeta {
    pub module: String,
    pub kind: String,
    pub lib: Option<String>,
    pub functions: Vec<FlameFunctionMeta>,
    pub structs: Vec<FlameStructMeta>,
    #[serde(default)]
    pub docs: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginSpec {
    pub name: String,
    pub source: String,
    pub version: Option<String>,
    pub is_local: bool,
}

