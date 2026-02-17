use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    Draft,
    InDevelopment,
    Stable,
    Deprecated,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Status::Draft => write!(f, "draft"),
            Status::InDevelopment => write!(f, "in-development"),
            Status::Stable => write!(f, "stable"),
            Status::Deprecated => write!(f, "deprecated"),
        }
    }
}

impl FromStr for Status {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(Status::Draft),
            "in-development" => Ok(Status::InDevelopment),
            "stable" => Ok(Status::Stable),
            "deprecated" => Ok(Status::Deprecated),
            other => Err(format!(
                "invalid status: {other}. Expected one of: draft, in-development, stable, deprecated."
            )),
        }
    }
}
