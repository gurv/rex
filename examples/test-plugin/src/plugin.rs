//! Implementation of `vg:rex/plugin` for the test plugin

use crate::bindings::wasi::logging::logging::{Level, log};
use crate::bindings::vg::rex::types::{
    Command, CommandArgument, HookType, Metadata, Runner,
};

impl crate::bindings::exports::vg::rex::plugin::Guest for crate::Component {
    /// Called by wash to retrieve the plugin metadata
    fn info() -> Metadata {
        Metadata {
            id: "dev.rex.test".to_string(),
            name: "test-enhanced".to_string(),
            description: "Test".to_string(),
            contact: "Vladimir Gurinovich <vladimir.gurinovich@gmail.com>".to_string(),
            url: "https://github.com/gurv/rex".to_string(),
            license: "Apache-2.0".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            command: Some(Command {
                id: "test-enhanced".to_string(),
                name: "test-enhanced".to_string(),
                description: "Test"
                    .to_string(),
                flags: vec![],
                arguments: vec![CommandArgument {
                    name: "component_path".to_string(),
                    description: "Path to the WebAssembly component file or project directory to test. If omitted, inspects the current directory as a project.".to_string(),
                    env: None,
                    default: Some(".".to_string()),
                    value: None,
                }],
                usage: vec![
                    "wash test-enhanced".to_string(),
                    "wash test-enhanced ./my-component.wasm".to_string(),
                    "wash test-enhanced /path/to/component.wasm".to_string(),
                    "wash test-enhanced ./my-project/".to_string(),
                ],
            }),
            sub_commands: vec![],
            // hooks: vec![HookType::AfterDev],
            hooks: vec![],
        }
    }

    fn initialize(_runner: Runner) -> Result<String, String> {
        log(Level::Info, "", "test plugin initialized successfully");
        Ok("test plugin ready".to_string())
    }

    fn run(runner: Runner, command: Command) -> Result<String, String> {
        log(
            Level::Debug,
            "",
            &format!("Executing test command: {}", command.name),
        );
        Ok("Test plugin run".to_string())
    }

    fn hook(runner: Runner, hook: HookType) -> Result<String, String> {
        log(
            Level::Debug,
            "",
            &format!("Hook type {:?}", hook),
        );
        Ok("Test plugin run".to_string())
    }
}
