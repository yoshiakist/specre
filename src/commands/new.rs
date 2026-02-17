// @specre 01JMBJK7QRVX3N4P5G6H8W9Y0Z
// @specre 01KHDF9WHR5HFM4RQCF6HS3KCC
// @specre 01KHG0A2V4YXE918WMJCY7WFE8
use crate::cli::NewArgs;
use crate::card::to_forward_slash;
use crate::config;
use crate::error::SpecreError;
use crate::template;
use crate::ulid;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
struct NewOutput {
    id: String,
    path: String,
}

/// # Errors
///
/// Returns [`SpecreError`] on invalid arguments, I/O, or serialization failure.
pub fn execute(args: &NewArgs, json: bool) -> Result<(), SpecreError> {
    let target = Path::new(&args.target_dir);

    if target.is_file() {
        return Err(SpecreError::InvalidArgument(format!(
            "'{}' is a file, not a directory",
            target.display()
        )));
    }

    let file_name = format!("{}.md", args.name);
    let file_path = target.join(&file_name);

    if file_path.exists() {
        return Err(SpecreError::InvalidArgument(format!(
            "'{}' already exists",
            file_path.display()
        )));
    }

    if !target.exists() {
        fs::create_dir_all(target).map_err(|e| SpecreError::Io {
            path: target.to_path_buf(),
            source: e,
        })?;
    }

    let language = config::load_language();
    let id = ulid::generate();
    let content = template::render(&id, &args.name, &language);

    fs::write(&file_path, &content).map_err(|e| SpecreError::Io {
        path: file_path.clone(),
        source: e,
    })?;

    if json {
        let output = NewOutput {
            id,
            path: to_forward_slash(&file_path).into_owned(),
        };
        let json_str = serde_json::to_string_pretty(&output)?;
        println!("{json_str}");
    } else {
        println!("{}", file_path.display());
    }

    Ok(())
}
