use miette::IntoDiagnostic;
use rex_common::{is_ci, is_test_env};
use rex_env::RexEnvironment;
use rex_env_var::GlobalEnvBag;
use semver::Version;
use serde::{Deserialize, Serialize};
use starbase_utils::{fs, json};
use std::collections::BTreeMap;
use std::env::consts;
use std::path::Path;
use std::sync::{Arc, OnceLock};
use tracing::{debug, instrument};
use uuid::Uuid;

static INSTANCE: OnceLock<Arc<Launchpad>> = OnceLock::new();

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(default)]
pub struct CurrentVersion {
    pub current_version: String,
    pub message: Option<String>,
}

#[derive(Serialize)]
pub struct ToolchainUsage {
    pub toolchains: BTreeMap<String, String>,
}

fn load_or_create_anonymous_uid(id_path: &Path) -> miette::Result<String> {
    if id_path.exists() {
        return Ok(fs::read_file(id_path)?);
    }

    let id = Uuid::new_v4().to_string();

    fs::write_file(id_path, &id)?;

    Ok(id)
}

fn create_anonymous_rid(workspace_root: &Path) -> String {
    format!(
        "{:x}",
        md5::compute(
            GlobalEnvBag::instance()
                .get("REX_VCS_REPO_SLUG")
                .unwrap_or_else(|| fs::file_name(workspace_root)),
        )
    )
}

pub struct VersionCheck {
    pub local_version: Version,
    pub remote_version: Version,
    pub message: Option<String>,
    pub update_available: bool,
}

pub struct Launchpad {
    #[allow(dead_code)]
    rex_env: Arc<RexEnvironment>,
    rex_version: String,
    user_id: String,
    repo_id: String,
}

impl Launchpad {
    pub fn register(rex_env: Arc<RexEnvironment>) -> miette::Result<()> {
        let user_id = load_or_create_anonymous_uid(&rex_env.id_file)?;
        let repo_id = create_anonymous_rid(&rex_env.workspace_root);

        let rex_version = GlobalEnvBag::instance()
            .get("REX_VERSION")
            .unwrap_or_default();

        let _ = INSTANCE.set(Arc::new(Self {
            rex_env,
            rex_version,
            user_id,
            repo_id,
        }));

        Ok(())
    }

    pub fn instance() -> Option<Arc<Launchpad>> {
        INSTANCE.get().map(Arc::clone)
    }

    #[instrument(skip_all)]
    pub async fn check_version(
        &self,
        manifest_url: &str,
    ) -> miette::Result<Option<VersionCheck>> {
        if is_test_env() || rex_common::is_offline() {
            return Ok(None);
        }

        if let Some(result) = self.check_version_without_cache(manifest_url).await? {
            return Ok(Some(result));
        }

        Ok(None)
    }

    pub async fn check_version_without_cache(
        &self,
        manifest_url: &str,
    ) -> miette::Result<Option<VersionCheck>> {
        if is_test_env() || rex_common::is_offline() {
            return Ok(None);
        }

        let version = &self.rex_version;

        debug!(
            current_version = &version,
            manifest_url = manifest_url,
            "Checking for a new version of rex"
        );

        let request = self
            .create_request(manifest_url)?
            .header(
                "X-Rex-CI-Provider",
                format!("{:?}", ci_env::detect_provider()),
            )
            .header(
                "X-Rex-CD-Provider",
                format!("{:?}", cd_env::detect_provider()),
            );

        let Ok(response) = request.send().await else {
            return Ok(None);
        };

        let Ok(text) = response.text().await else {
            return Ok(None);
        };

        let data: CurrentVersion = json::parse(text)?;
        let local_version = Version::parse(version).into_diagnostic()?;
        let remote_version = Version::parse(&data.current_version).into_diagnostic()?;
        let update_available = remote_version > local_version;

        if update_available {
            debug!(
                latest_version = &data.current_version,
                "Found a newer version"
            );
        }

        Ok(Some(VersionCheck {
            local_version,
            remote_version,
            message: data.message,
            update_available,
        }))
    }

    pub async fn track_toolchain_usage(
        &self,
        toolchains: BTreeMap<String, String>,
    ) -> miette::Result<()> {
        if !is_ci() || is_test_env() || rex_common::is_offline() {
            return Ok(());
        }

        let request = self
            .create_request("https://launch.moonrepo.app/rex/toolchain_usage")?
            .json(&ToolchainUsage { toolchains });

        let _response = request.send().await.into_diagnostic()?;

        Ok(())
    }

    fn create_request(&self, url: &str) -> miette::Result<reqwest::RequestBuilder> {
        let client = reqwest::Client::new()
            .post(url)
            .header("X-Rex-OS", consts::OS.to_owned())
            .header("X-Rex-Arch", consts::ARCH.to_owned())
            .header("X-Rex-Version", self.rex_version.clone())
            .header("X-Rex-CI", ci_env::is_ci().to_string())
            .header("X-Rex-CD", cd_env::is_cd().to_string())
            .header("X-Rex-UID", self.user_id.clone())
            .header("X-Rex-RID", self.repo_id.clone());

        Ok(client)
    }
}
