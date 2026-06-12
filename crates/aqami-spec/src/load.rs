use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::Value;
use thiserror::Error;

use crate::AqamiProjectSpec;

#[derive(Debug, Clone)]
pub struct LoadedProjectSpec {
    pub path: PathBuf,
    pub raw_value: Value,
    pub project: AqamiProjectSpec,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SpecFormat {
    Json,
    Yaml,
}

impl SpecFormat {
    fn from_path(path: &Path) -> Self {
        match path.extension().and_then(|value| value.to_str()) {
            Some("json") => Self::Json,
            _ => Self::Yaml,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Json => "JSON",
            Self::Yaml => "YAML",
        }
    }
}

#[derive(Debug, Error)]
pub enum SpecLoadError {
    #[error("failed to read AQAMI spec file {path}: {message}")]
    Read { path: PathBuf, message: String },
    #[error("failed to parse {format} AQAMI spec file {path}: {message}")]
    Parse {
        path: PathBuf,
        format: &'static str,
        message: String,
    },
    #[error("failed to deserialize AQAMI project spec from {path}: {message}")]
    Deserialize { path: PathBuf, message: String },
}

pub fn load_project_spec(path: impl AsRef<Path>) -> Result<LoadedProjectSpec, SpecLoadError> {
    let path = path.as_ref().to_path_buf();
    let format = SpecFormat::from_path(&path);
    let source = fs::read_to_string(&path).map_err(|error| SpecLoadError::Read {
        path: path.clone(),
        message: error.to_string(),
    })?;
    let raw_value = parse_raw_value(&path, format, &source)?;
    let project =
        serde_json::from_value(raw_value.clone()).map_err(|error| SpecLoadError::Deserialize {
            path: path.clone(),
            message: error.to_string(),
        })?;

    Ok(LoadedProjectSpec {
        path,
        raw_value,
        project,
    })
}

fn parse_raw_value(path: &Path, format: SpecFormat, source: &str) -> Result<Value, SpecLoadError> {
    match format {
        SpecFormat::Json => serde_json::from_str(source).map_err(|error| SpecLoadError::Parse {
            path: path.to_path_buf(),
            format: format.as_str(),
            message: error.to_string(),
        }),
        SpecFormat::Yaml => yaml_serde::from_str(source).map_err(|error| SpecLoadError::Parse {
            path: path.to_path_buf(),
            format: format.as_str(),
            message: error.to_string(),
        }),
    }
}
