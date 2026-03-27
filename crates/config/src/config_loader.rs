use crate::config_finder::ConfigFinder;
use crate::extensions_config::ExtensionsConfig;
use crate::formats::hcl::HclFormat;
use rex_common::color;
use schematic::{Config, ConfigLoader as Loader};
use std::ops::Deref;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default)]
pub struct ConfigLoader {
    pub dir: PathBuf, // .rex
    finder: ConfigFinder,
}

impl ConfigLoader {
    pub fn new(dir: impl AsRef<Path>) -> Self {
        Self {
            dir: dir.as_ref().to_path_buf(),
            finder: ConfigFinder::default(),
        }
    }

    pub fn locate_dir(&mut self, workspace_root: &Path) -> PathBuf {
        let rex_dir = workspace_root.join(".rex");
        let config_rex_dir = workspace_root.join(".config").join("rex");

        if config_rex_dir.exists() {
            self.dir = config_rex_dir;
        } else {
            self.dir = rex_dir;
        }

        self.dir.clone()
    }

    pub fn create_extensions_loader<P: AsRef<Path>>(
        &self,
        workspace_root: P,
    ) -> miette::Result<Loader<ExtensionsConfig>> {
        let mut loader = Loader::<ExtensionsConfig>::new();

        loader
            .set_help(color::muted_light(
                "https://moonrepo.dev/docs/config/extensions",
            ))
            .set_root(workspace_root);

        self.prepare_loader(&mut loader, self.get_extensions_files())?;

        Ok(loader)
    }

    pub fn load_extensions_config<P: AsRef<Path>>(
        &self,
        workspace_root: P,
    ) -> miette::Result<ExtensionsConfig> {
        let mut result = self.create_extensions_loader(workspace_root)?.load()?;

        #[cfg(feature = "proto")]
        {
            use rex_warpgate_api::PluginLocator;

            result.config.inherit_defaults()?;

            // Resolve plugin file locations
            for config in result.config.plugins.values_mut() {
                if let Some(PluginLocator::File(file)) = &mut config.plugin {
                    let file_path = file.get_unresolved_path();

                    file.path = Some(if file_path.is_absolute() {
                        file_path
                    } else {
                        self.dir.join(file_path)
                    });
                }
            }
        }

        Ok(result.config)
    }

    pub fn prepare_loader<T: Config>(
        &self,
        loader: &mut Loader<T>,
        files: Vec<PathBuf>,
    ) -> miette::Result<()> {
        loader.add_format(HclFormat::default());

        for file in files {
            loader.file_optional(file)?;
        }

        Ok(())
    }

    pub fn get_debug_label(&self, name: &str) -> String {
        self.finder.get_debug_label(name)
    }

    pub fn get_debug_label_root(&self, name: &str) -> String {
        self.finder.get_debug_label_root(name, &self.dir)
    }

    pub fn get_extensions_files(&self) -> Vec<PathBuf> {
        self.finder
            .get_extensions_file_names()
            .into_iter()
            .map(|name| self.dir.join(name))
            .collect()
    }

    pub fn get_project_files(&self, project_root: &Path) -> Vec<PathBuf> {
        self.finder
            .get_project_file_names()
            .into_iter()
            .map(|name| project_root.join(name))
            .collect()
    }

    pub fn get_tasks_files(&self, tasks_dir: &Path) -> miette::Result<Vec<PathBuf>> {
        self.finder.get_from_dir(tasks_dir.join("tasks"))
    }

    pub fn get_template_files(&self, template_root: &Path) -> Vec<PathBuf> {
        self.finder
            .get_template_file_names()
            .into_iter()
            .map(|name| template_root.join(name))
            .collect()
    }

    pub fn get_toolchains_files(&self) -> Vec<PathBuf> {
        self.finder
            .get_toolchains_file_names()
            .into_iter()
            .map(|name| self.dir.join(name))
            .collect()
    }

    pub fn get_workspace_files(&self) -> Vec<PathBuf> {
        self.finder
            .get_workspace_file_names()
            .into_iter()
            .map(|name| self.dir.join(name))
            .collect()
    }
}

impl Deref for ConfigLoader {
    type Target = ConfigFinder;

    fn deref(&self) -> &Self::Target {
        &self.finder
    }
}
