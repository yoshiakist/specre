// @specre 01KHAKAYN5WPTDVR99D5Q5TMJE
// @specre 01KHB48EES4FR5TFV6ZP2W3MGT
// @specre 01KHFTCYJN8YJMW2RNHJTAQV49
// @specre 01KHAN6JE712ZAKXPP97854PKJ
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
            Self::Draft => write!(f, "draft"),
            Self::InDevelopment => write!(f, "in-development"),
            Self::Stable => write!(f, "stable"),
            Self::Deprecated => write!(f, "deprecated"),
        }
    }
}

impl FromStr for Status {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "draft" => Ok(Self::Draft),
            "in-development" => Ok(Self::InDevelopment),
            "stable" => Ok(Self::Stable),
            "deprecated" => Ok(Self::Deprecated),
            other => Err(format!(
                "invalid status: {other}. Expected one of: draft, in-development, stable, deprecated."
            )),
        }
    }
}
