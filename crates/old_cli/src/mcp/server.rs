use super::resources::*;
use crate::session::RexSession;
use rex_old_core::{
    RexConfigEnvOptions,
    get_rex_version,
};
use rmcp::{
    ErrorData as McpError, RoleServer, ServerHandler,
    handler::server::{router::tool::ToolRouter},
    model::*,
    service::RequestContext,
    tool, tool_handler, tool_router,
};
use serde_json::json;
use std::fmt::Display;

macro_rules! handle_tool_error {
    ($result:expr) => {
        match $result {
            Ok(inner) => inner,
            Err(error) => {
                return Ok(CallToolResult::error(vec![Annotated::new(
                    RawContent::text(error.to_string()),
                    None,
                )]));
            }
        }
    };
}

#[derive(Clone)]
pub struct RexMcp {
    session: RexSession,

    pub tool_router: ToolRouter<RexMcp>,
}

impl RexMcp {
    pub fn list_all_resources(&self) -> ListResourcesResult {
        ListResourcesResult {
            resources: vec![
                RawResource::new("rex://config", "Configuration".to_string()).no_annotation(),
                RawResource::new("rex://env", "Environment".to_string()).no_annotation(),
                RawResource::new("rex://tools", "Installed tools".to_string()).no_annotation(),
            ],
            next_cursor: None,
            meta: None,
        }
    }

    fn resource_config(&self) -> miette::Result<ConfigResource<'_>> {
        let env = &self.session.env;

        Ok(ConfigResource {
            working_dir: env.working_dir.clone(),
            config_mode: env.config_mode,
            config_files: env
                .load_file_manager()?
                .entries
                .iter()
                .map(|entry| &entry.path)
                .collect(),
            config: env.load_config()?,
        })
    }

    fn resource_env(&self) -> miette::Result<EnvResource<'_>> {
        let env = &self.session.env;
        let config = env.load_config()?;
        let options = RexConfigEnvOptions {
            include_shared: true,
            ..Default::default()
        };

        Ok(EnvResource {
            working_dir: env.working_dir.clone(),
            store_dir: env.store.dir.clone(),
            env_mode: env.env_mode.clone(),
            env_files: config.get_env_files(options.clone()),
            env_vars: config.get_env_vars(options)?,
            rex_version: get_rex_version().to_string(),
            system_arch: env.arch,
            system_os: env.os,
        })
    }

    async fn resource_tools(&self) -> miette::Result<ToolsResource> {
        let mut resource = ToolsResource {
            tools: Default::default(),
        };

        for tool in self.session.load_tools().await? {
            resource.tools.insert(
                tool.context.clone(),
                ToolResourceEntry {
                    tool_dir: tool.get_inventory_dir().to_path_buf(),
                    installed_versions: Vec::from_iter(
                        tool.inventory.manifest.installed_versions.clone(),
                    ),
                },
            );
        }

        Ok(resource)
    }
}

#[tool_router]
impl RexMcp {
    pub fn new(session: RexSession) -> Self {
        Self {
            session,
            tool_router: Self::tool_router(),
            // prompt_router: Self::prompt_router(),
        }
    }

    #[tool(description = "Get configuration for the current working directory.")]
    async fn get_config(&self) -> Result<CallToolResult, McpError> {
        let config = handle_tool_error!(self.session.load_config());

        Ok(CallToolResult::structured(
            serde_json::to_value(config).unwrap(),
        ))
    }
}

#[tool_handler]
impl ServerHandler for RexMcp {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(
            ServerCapabilities::builder()
                // .enable_prompts()
                .enable_resources()
                .enable_tools()
                .build()
        )
        .with_server_info(
            Implementation::from_build_env().with_website_url("https://moonrepo.dev/rex")
        )
        .with_instructions(
            "The rex MCP server provides resources and tools for managing your toolchain, environment, and more."
        )
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(self.list_all_resources())
    }

    async fn read_resource(
        &self,
        ReadResourceRequestParams { uri, .. }: ReadResourceRequestParams,
        _: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        let text = match uri.as_str() {
            "rex://config" => {
                let resource = self
                    .resource_config()
                    .map_err(|error| map_resource_error(error, &uri))?;

                serde_json::to_string_pretty(&resource).unwrap()
            }
            "rex://env" => {
                let resource = self
                    .resource_env()
                    .map_err(|error| map_resource_error(error, &uri))?;

                serde_json::to_string_pretty(&resource).unwrap()
            }
            "rex://tools" => {
                let resource = self
                    .resource_tools()
                    .await
                    .map_err(|error| map_resource_error(error, &uri))?;

                serde_json::to_string_pretty(&resource).unwrap()
            }
            _ => {
                return Err(McpError::resource_not_found(
                    "Resource does not exist.",
                    Some(json!({
                        "uri": uri
                    })),
                ));
            }
        };

        Ok(ReadResourceResult::new(vec![
            ResourceContents::TextResourceContents {
                uri,
                text,
                mime_type: Some("application/json".into()),
                meta: None,
            },
        ]))
    }
}

fn map_resource_error(error: impl Display, uri: &str) -> McpError {
    McpError::internal_error(
        error.to_string(),
        Some(json!({
            "uri": uri
        })),
    )
}
