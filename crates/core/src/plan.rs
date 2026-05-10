use crate::config::{AppConfig, InspectEndpointConfig, Manifest, ProjectConfig, SidecarConfig};
use crate::stamp::{Stamp, DEFAULT_SOURCE};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionPlan {
    pub project: String,
    pub namespace: String,
    pub root: String,
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
    pub stamp: Stamp,
    pub inspect_socket: Option<String>,
    pub health_url: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InspectEndpointPlan {
    pub name: String,
    pub kind: String,
    pub url: String,
}

impl ExecutionPlan {
    pub fn from_config(config: &Manifest) -> Self {
        Self {
            project: config.project.name.clone(),
            namespace: config.project.namespace.clone(),
            root: config.project.root.clone(),
            app: config.app.as_ref().map(AppPlan::from_config),
            sidecars: config
                .sidecars
                .iter()
                .map(|sidecar| SidecarPlan::from_config(sidecar, &config.project))
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
    fn from_config(config: &SidecarConfig, project: &ProjectConfig) -> Self {
        let stamp = Stamp {
            app: config.name.clone(),
            namespace: project.namespace.clone(),
            mode: config.mode.clone(),
            source: DEFAULT_SOURCE.to_string(),
        };
        Self {
            name: config.name.clone(),
            command: config.command.clone(),
            args: config.args.clone(),
            cwd: config.cwd.clone(),
            stamp,
            inspect_socket: config.inspect_socket.clone(),
            health_url: config.health_url.clone(),
        }
    }

    /// Final argv to spawn (sidecar args followed by stamp args).
    pub fn spawn_args(&self) -> Vec<String> {
        let mut argv = self.args.clone();
        argv.extend(self.stamp.args());
        argv
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
