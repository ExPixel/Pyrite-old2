use std::fmt;
use std::path::{Path, PathBuf};

#[derive(serde::Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub graphics: GraphicsConfig,
    pub gba: GbaConfig,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphicsConfig {
    pub vsync: Option<bool>,
    pub fps: Option<u32>,
}

impl Default for GraphicsConfig {
    fn default() -> GraphicsConfig {
        GraphicsConfig {
            vsync: Some(true),
            fps: None,
        }
    }
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GbaConfig {
    pub rom: Option<PathBuf>,
}

#[allow(clippy::derivable_impls)]
impl Default for GbaConfig {
    fn default() -> GbaConfig {
        GbaConfig { rom: None }
    }
}

pub fn from_toml_str(config_source: &str) -> Result<Config, Error> {
    toml::from_str(config_source).map_err(Into::into)
}

pub fn from_toml_path<P>(config_path: P) -> Result<Config, Error>
where
    P: AsRef<Path>,
{
    let config_path = config_path.as_ref();
    let source =
        std::fs::read_to_string(config_path).map_err(|err| Error::Io(config_path.into(), err))?;
    from_toml_str(&source)
}

#[derive(Debug)]
pub enum Error {
    Deserialize(toml::de::Error),
    Io(PathBuf, std::io::Error),
}

impl From<toml::de::Error> for Error {
    fn from(toml_err: toml::de::Error) -> Self {
        Error::Deserialize(toml_err)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Deserialize(_) => write!(f, "error occurred while deserializing TOML source"),
            Error::Io(path, _) => write!(f, "error occurred reading path `{}`", path.display()),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Deserialize(err) => Some(err),
            Error::Io(_, err) => Some(err),
        }
    }
}
