use clap::ValueEnum;
use std::fmt;

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum AppTheme {
    #[default]
    Dark,
    Light,
}

impl fmt::Display for AppTheme {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                Self::Dark => "dark",
                Self::Light => "light",
            }
        )
    }
}

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum LogLevel {
    Off,
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
    Verbose,
}

impl LogLevel {
    pub fn is_verbose(&self) -> bool {
        matches!(self, Self::Verbose)
    }
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                Self::Off => "off",
                Self::Error => "error",
                Self::Warn => "warn",
                Self::Info => "info",
                Self::Debug => "debug",
                // Must map to tracing levels
                Self::Trace | Self::Verbose => "trace",
            }
        )
    }
}

#[derive(ValueEnum, Clone, Debug, Default)]
pub enum SummaryOption {
    None,
    Minimal,
    #[default]
    Normal,
    Detailed,
}

impl fmt::Display for SummaryOption {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        write!(
            f,
            "{}",
            match self {
                Self::None => "none",
                Self::Minimal => "minimal",
                Self::Normal => "normal",
                Self::Detailed => "detailed",
            }
        )
    }
}
