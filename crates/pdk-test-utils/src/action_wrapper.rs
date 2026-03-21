use rex_pdk_api::*;
use std::path::PathBuf;
use rex_warpgate::PluginContainer;

pub struct ActionTestWrapper {
    pub metadata: RegisterActionOutput,
    pub plugin: PluginContainer,
    pub root: PathBuf,
}

impl ActionTestWrapper {
    pub fn create_context(&self) -> RexContext {
        RexContext {
            working_dir: self.plugin.to_virtual_path(&self.root),
        }
    }

    pub async fn execute_action(&self, mut input: ExecuteActionInput) {
        input.context = self.create_context();

        self.plugin
            .call_func_without_output("execute_action", input)
            .await
            .unwrap();
    }

    pub async fn register_action(
        &self,
        input: RegisterActionInput,
    ) -> RegisterActionOutput {
        self.plugin
            .call_func_with("register_action", input)
            .await
            .unwrap()
    }

    pub async fn action_command(&self, mut input: ExtendCommandInput) -> ExtendCommandOutput {
        input.context = self.create_context();

        self.plugin
            .call_func_with("action_command", input)
            .await
            .unwrap()
    }
}
