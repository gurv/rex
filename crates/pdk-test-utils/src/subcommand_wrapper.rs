use rex_pdk_api::*;
use std::path::PathBuf;
use std::sync::Arc;
use rex_warpgate::PluginContainer;

pub struct SubcommandTestWrapper {
    pub metadata: RegisterSubcommandOutput,
    pub plugin: Arc<PluginContainer>,
    pub root: PathBuf,
}

impl SubcommandTestWrapper {
    pub fn create_context(&self) -> RexContext {
        RexContext {
            working_dir: self.plugin.to_virtual_path(&self.root),
        }
    }

    pub async fn execute_subcommand(&self, mut input: ExecuteSubcommandInput) {
        input.context = self.create_context();

        self.plugin
            .call_func_without_output("execute_subcommand", input)
            .await
            .unwrap();
    }

    pub async fn register_subcommand(
        &self,
        input: RegisterSubcommandInput,
    ) -> RegisterSubcommandOutput {
        self.plugin
            .call_func_with("register_subcommand", input)
            .await
            .unwrap()
    }

    pub async fn subcommand_command(&self, mut input: ExtendCommandInput) -> ExtendCommandOutput {
        input.context = self.create_context();

        self.plugin
            .call_func_with("subcommand_command", input)
            .await
            .unwrap()
    }
}
