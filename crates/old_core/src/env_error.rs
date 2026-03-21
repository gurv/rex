use thiserror::Error;

#[derive(Error, Debug, miette::Diagnostic)]
pub enum RexEnvError {
    #[diagnostic(code(rex::env::home_dir))]
    #[error("Unable to determine your home directory.")]
    MissingHomeDir,

    #[diagnostic(code(rex::env::working_dir))]
    #[error("Unable to determine current working directory!")]
    MissingWorkingDir,
}
