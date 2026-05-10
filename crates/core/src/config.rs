use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Manifest {
    pub project: ProjectConfig,
    #[serde(default)]
    pub app: Option<AppConfig>,
    #[serde(default)]
    pub sidecars: Vec<SidecarConfig>,
    #[serde(default)]
    pub inspect: InspectConfig,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProjectConfig {
    pub name: String,
    #[serde(default = "default_namespace")]
    pub namespace: String,
    #[serde(default = "default_root")]
    pub root: String,
    #[serde(default)]
    pub data_dir: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AppConfig {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_root")]
    pub cwd: String,
    #[serde(default)]
    pub health_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SidecarConfig {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default = "default_root")]
    pub cwd: String,
    #[serde(default = "default_mode")]
    pub mode: String,
    #[serde(default)]
    pub inspect_socket: Option<String>,
    #[serde(default)]
    pub health_url: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct InspectConfig {
    #[serde(default)]
    pub endpoints: Vec<InspectEndpointConfig>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct InspectEndpointConfig {
    pub name: String,
    pub kind: String,
    pub url: String,
}

fn default_root() -> String {
    ".".to_string()
}

fn default_namespace() -> String {
    crate::stamp::DEFAULT_NAMESPACE.to_string()
}

fn default_mode() -> String {
    crate::stamp::DEFAULT_MODE.to_string()
}
