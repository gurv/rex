#![allow(dead_code)]

use rex_old_core::{RexConfig, RexFileManager, get_exe_file_name};
use starbase_sandbox::{Sandbox, assert_cmd};
use std::collections::HashMap;
use std::fs;
use std::ops::Deref;
use std::path::{Path, PathBuf};

pub struct RexSandbox {
    pub sandbox: Sandbox,
}

impl RexSandbox {
    pub fn new(mut sandbox: Sandbox) -> Self {
        apply_settings(&mut sandbox);

        Self { sandbox }
    }
}

impl Deref for RexSandbox {
    type Target = Sandbox;

    fn deref(&self) -> &Self::Target {
        &self.sandbox
    }
}

fn apply_settings(sandbox: &mut Sandbox) {
    let root = sandbox.path().to_path_buf();
    let home_dir = sandbox.path().join(".home");
    let rex_dir = sandbox.path().join(".rex");

    fs::create_dir_all(&home_dir).unwrap();
    fs::create_dir_all(&rex_dir).unwrap();

    let mut env = HashMap::new();
    env.insert("RUST_BACKTRACE", "1");
    env.insert("WASMTIME_BACKTRACE_DETAILS", "1");
    env.insert("NO_COLOR", "1");
    env.insert("REX_SANDBOX", root.to_str().unwrap());
    env.insert("REX_HOME", rex_dir.to_str().unwrap());
    env.insert("REX_LOG", "trace");
    env.insert("REX_TEST", "true");

    sandbox.settings.bin = "rex".into();
    sandbox.settings.timeout = 300;

    sandbox
        .settings
        .env
        .extend(env.into_iter().map(|(k, v)| (k.to_owned(), v.to_owned())));
}

pub fn create_empty_rex_sandbox() -> RexSandbox {
    RexSandbox::new(starbase_sandbox::create_empty_sandbox())
}

pub fn create_rex_sandbox<N: AsRef<str>>(fixture: N) -> RexSandbox {
    RexSandbox::new(starbase_sandbox::create_sandbox(fixture))
}

pub fn load_config<T: AsRef<Path>>(dir: T) -> RexConfig {
    let manager = RexFileManager::load(dir, None, None).unwrap();
    let config = manager.get_merged_config().unwrap();
    config.to_owned()
}

pub fn create_bin_command<T: AsRef<Path>>(path: T, name: &str) -> assert_cmd::Command {
    let mut cmd = assert_cmd::Command::from_std(create_bin_command_std(path, name));
    cmd.timeout(std::time::Duration::from_secs(240));
    cmd
}

pub fn create_bin_command_std<T: AsRef<Path>>(path: T, name: &str) -> std::process::Command {
    let path = path.as_ref();

    let mut cmd = std::process::Command::new(get_bin_path(path, name));
    cmd.env("REX_LOG", "trace");
    cmd.env("REX_HOME", path.join(".rex"));
    cmd.env(format!("REX_{}_VERSION", name.to_uppercase()), "latest");
    cmd
}

pub fn get_bin_path<T: AsRef<Path>>(path: T, name: &str) -> PathBuf {
    path.as_ref()
        .join(".rex/bin")
        .join(get_exe_file_name(name))
}

pub fn link_bin(input_path: &Path, output_path: &Path) {
    fs::create_dir_all(output_path.parent().unwrap()).unwrap();

    #[cfg(windows)]
    {
        fs::copy(input_path, output_path).unwrap();
    }

    #[cfg(not(windows))]
    {
        std::os::unix::fs::symlink(input_path, output_path).unwrap();
    }
}
