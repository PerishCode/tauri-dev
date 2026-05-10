use crate::config::{AppConfig, InspectEndpointConfig, SidecarConfig, TauriDevConfig};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionPlan {
    pub project: String,
    pub app: Option<AppPlan>,
    pub sidecars: Vec<SidecarPlan>,
    pub inspect_endpoints: Vec<InspectEndpointPlan>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppPlan {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub health_url: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SidecarPlan {
    pub name: String,
    pub command: String,
    pub args: Vec<String>,
    pub cwd: String,
    pub socket: Option<String>,
    pub health_url: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectEndpointPlan {
    pub name: String,
    pub kind: String,
    pub url: String,
}

impl ExecutionPlan {
    pub fn from_config(config: &TauriDevConfig) -> Self {
        Self {
            project: config.project.name.clone(),
            app: config.app.as_ref().map(AppPlan::from_config),
            sidecars: config
                .sidecars
                .iter()
                .map(SidecarPlan::from_config)
                .collect(),
            inspect_endpoints: config
                .inspect
                .endpoints
                .iter()
                .map(InspectEndpointPlan::from_config)
                .collect(),
        }
    }
}

impl AppPlan {
    fn from_config(config: &AppConfig) -> Self {
        Self {
            name: config.name.clone(),
            command: config.command.clone(),
            args: config.args.clone(),
            cwd: config.cwd.clone(),
            health_url: config.health_url.clone(),
        }
    }
}

impl SidecarPlan {
    fn from_config(config: &SidecarConfig) -> Self {
        Self {
            name: config.name.clone(),
            command: config.command.clone(),
            args: config.args.clone(),
            cwd: config.cwd.clone(),
            socket: config.socket.clone(),
            health_url: config.health_url.clone(),
        }
    }
}

impl InspectEndpointPlan {
    fn from_config(config: &InspectEndpointConfig) -> Self {
        Self {
            name: config.name.clone(),
            kind: config.kind.clone(),
            url: config.url.clone(),
        }
    }
}
