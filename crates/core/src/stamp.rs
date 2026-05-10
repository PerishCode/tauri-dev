//! Stamp args protocol — labels injected into a sidecar process command line so
//! the `sidecar` tool can identify and operate on it later.
//!
//! Canonical flag names are `--sidecar-stamp-{app,namespace,mode,source}`.
//! Discovery only relies on these flags (not env vars), so any consumer that
//! emits the canonical flags is interoperable with the `sidecar` CLI.

pub const STAMP_APP_FLAG: &str = "--sidecar-stamp-app";
pub const STAMP_NAMESPACE_FLAG: &str = "--sidecar-stamp-namespace";
pub const STAMP_MODE_FLAG: &str = "--sidecar-stamp-mode";
pub const STAMP_SOURCE_FLAG: &str = "--sidecar-stamp-source";

pub const DEFAULT_NAMESPACE: &str = "default";
pub const DEFAULT_MODE: &str = "dev";
pub const DEFAULT_SOURCE: &str = "tool:sidecar";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Stamp {
    pub app: String,
    pub namespace: String,
    pub mode: String,
    pub source: String,
}

impl Stamp {
    pub fn args(&self) -> Vec<String> {
        vec![
            format!("{STAMP_APP_FLAG}={}", self.app),
            format!("{STAMP_NAMESPACE_FLAG}={}", self.namespace),
            format!("{STAMP_MODE_FLAG}={}", self.mode),
            format!("{STAMP_SOURCE_FLAG}={}", self.source),
        ]
    }
}

pub fn read_flag(args: &[String], flag: &str) -> Option<String> {
    let prefix = format!("{flag}=");
    for (index, value) in args.iter().enumerate() {
        if value == flag {
            return args.get(index + 1).cloned();
        }
        if let Some(stripped) = value.strip_prefix(&prefix) {
            return Some(stripped.to_string());
        }
    }
    None
}

pub fn read_stamp(args: &[String]) -> Option<Stamp> {
    Some(Stamp {
        app: read_flag(args, STAMP_APP_FLAG)?,
        namespace: read_flag(args, STAMP_NAMESPACE_FLAG)?,
        mode: read_flag(args, STAMP_MODE_FLAG)?,
        source: read_flag(args, STAMP_SOURCE_FLAG)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn args_emit_canonical_flags() {
        let stamp = Stamp {
            app: "controller".into(),
            namespace: "default".into(),
            mode: "dev".into(),
            source: "tool:sidecar".into(),
        };
        let args = stamp.args();
        assert_eq!(args[0], "--sidecar-stamp-app=controller");
        assert_eq!(args[1], "--sidecar-stamp-namespace=default");
        assert_eq!(args[2], "--sidecar-stamp-mode=dev");
        assert_eq!(args[3], "--sidecar-stamp-source=tool:sidecar");
    }

    #[test]
    fn read_flag_supports_inline_and_separated_forms() {
        let inline = vec!["--sidecar-stamp-app=api".to_string()];
        assert_eq!(read_flag(&inline, STAMP_APP_FLAG).as_deref(), Some("api"));

        let separated = vec![
            "--sidecar-stamp-namespace".to_string(),
            "design".to_string(),
        ];
        assert_eq!(
            read_flag(&separated, STAMP_NAMESPACE_FLAG).as_deref(),
            Some("design")
        );
    }

    #[test]
    fn read_stamp_requires_all_four_flags() {
        let partial = vec!["--sidecar-stamp-app=api".to_string()];
        assert!(read_stamp(&partial).is_none());

        let full = vec![
            "--sidecar-stamp-app=api".into(),
            "--sidecar-stamp-namespace=default".into(),
            "--sidecar-stamp-mode=dev".into(),
            "--sidecar-stamp-source=tool:sidecar".into(),
        ];
        let stamp = read_stamp(&full).unwrap();
        assert_eq!(stamp.app, "api");
        assert_eq!(stamp.source, "tool:sidecar");
    }
}
