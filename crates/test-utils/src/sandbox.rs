#![allow(clippy::disallowed_types)]

use rex_config::{
    PartialExtensionsConfig, PartialToolchainsConfig, PartialWorkspaceConfig,
    PartialWorkspaceProjects, PartialWorkspaceProjectsConfig,
};
pub use starbase_sandbox::{Sandbox, SandboxAssert, SandboxSettings, create_temp_dir};
use starbase_utils::yaml;
use std::collections::HashMap;
use std::ops::Deref;

pub struct RexSandbox {
    pub sandbox: Sandbox,
}

impl RexSandbox {
    pub fn new(mut sandbox: Sandbox, create: bool) -> Self {
        apply_settings(&mut sandbox);

        if create {
            create_workspace_files(&sandbox);
        }

        Self { sandbox }
    }

    pub fn capture_plugin_logs(&self) {
        extism::set_log_callback(
            |line| {
                println!("{line}");
            },
            "debug",
        )
        .unwrap();
    }

    pub fn update_extensions_config(&self, op: impl FnOnce(&mut PartialExtensionsConfig)) {
        let path = self.path().join(".rex/extensions.yml");

        let mut config: PartialExtensionsConfig = if path.exists() {
            yaml::read_file(&path).unwrap()
        } else {
            Default::default()
        };

        op(&mut config);

        yaml::write_file(path, &config).unwrap();
    }

    pub fn update_toolchains_config(&self, op: impl FnOnce(&mut PartialToolchainsConfig)) {
        let path = self.path().join(".rex/toolchains.yml");

        let mut config: PartialToolchainsConfig = if path.exists() {
            yaml::read_file(&path).unwrap()
        } else {
            Default::default()
        };

        op(&mut config);

        yaml::write_file(path, &config).unwrap();
    }

    pub fn update_workspace_config(&self, op: impl FnOnce(&mut PartialWorkspaceConfig)) {
        let path = self.path().join(".rex/workspace.yml");

        let mut config: PartialWorkspaceConfig = if path.exists() {
            yaml::read_file(&path).unwrap()
        } else {
            Default::default()
        };

        op(&mut config);

        yaml::write_file(path, &config).unwrap();
    }

    pub fn with_default_projects(&self) {
        self.update_workspace_config(|config| {
            let mut projects = PartialWorkspaceProjectsConfig {
                globs: Some(vec![
                    "*".into(),
                    "!.home".into(),
                    "!.rex".into(),
                    "!.proto".into(),
                ]),
                ..Default::default()
            };

            if self.path().join("rex.yml").exists() {
                projects
                    .sources
                    .get_or_insert_default()
                    .insert("root".try_into().unwrap(), ".".into());
            }

            config.projects = Some(PartialWorkspaceProjects::Both(projects));
        });
    }
}

impl Deref for RexSandbox {
    type Target = Sandbox;

    fn deref(&self) -> &Self::Target {
        &self.sandbox
    }
}

fn apply_settings(sandbox: &mut Sandbox) {
    let rex_dir = sandbox.path().join(".rex");

    let mut env = HashMap::new();
    env.insert("RUST_BACKTRACE", "1");
    env.insert("WASMTIME_BACKTRACE_DETAILS", "1");
    env.insert("NO_COLOR", "1");
    env.insert("COLUMNS", "150");
    // Store plugins in the sandbox
    env.insert("REX_HOME", rex_dir.to_str().unwrap());
    // env.insert("PROTO_HOME", path.join(".proto"));
    // Let our code know we're running tests
    env.insert("REX_TEST", "true");
    env.insert("STARBASE_TEST", "true");
    // Don't exhaust all cores on the machine
    env.insert("REX_CONCURRENCY", "2");
    // Hide install output as it disrupts testing snapshots
    env.insert("REX_TEST_HIDE_INSTALL_OUTPUT", "true");
    // Standardize file system paths for testing snapshots
    env.insert("REX_TEST_STANDARDIZE_PATHS", "true");
    // Enable logging for code coverage
    env.insert("REX_LOG", "trace");
    // Advanced debugging
    // env.insert("PROTO_LOG", "trace");
    // env.insert("REX_DEBUG_WASM", "true");

    sandbox.settings.bin = "rex".into();

    sandbox
        .settings
        .env
        .extend(env.into_iter().map(|(k, v)| (k.to_owned(), v.to_owned())));
}

fn create_workspace_files(sandbox: &Sandbox) {
    if !sandbox.path().join(".rex/workspace.yml").exists() {
        sandbox.create_file(".rex/workspace.yml", "projects: ['*']");
    }
}

pub fn create_empty_sandbox() -> RexSandbox {
    RexSandbox::new(starbase_sandbox::create_empty_sandbox(), false)
}

pub fn create_empty_rex_sandbox() -> RexSandbox {
    RexSandbox::new(starbase_sandbox::create_empty_sandbox(), true)
}

pub fn create_rex_sandbox<N: AsRef<str>>(fixture: N) -> RexSandbox {
    RexSandbox::new(starbase_sandbox::create_sandbox(fixture), true)
}
