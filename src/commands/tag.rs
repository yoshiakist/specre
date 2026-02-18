// @specre 01KHB48EYB9686YYQMYFYQ5R1Z
// @specre 01KHG0A2V4YXE918WMJCY7WFE8
use crate::card::to_forward_slash;
use crate::cli::TagArgs;
use crate::error::SpecreError;
use crate::ulid;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize)]
struct TagOutput {
    id: String,
    file: String,
    line: usize,
}

pub fn comment_syntax(ext: &str) -> Option<(&'static str, &'static str)> {
    match ext {
        // // style — C-family, JVM, modern languages, shaders
        "rs" | "js" | "ts" | "jsx" | "tsx" | "java" | "c" | "cpp" | "h" | "hpp" | "cs" | "go"
        | "swift" | "kt" | "kts" | "scala" | "groovy" | "gradle" | "dart" | "php" | "zig"
        | "proto" | "prisma" | "jsonc" | "shader" | "hlsl" | "cginc" | "compute" | "usf"
        | "ush" | "gdshader" | "glsl" | "vert" | "frag" | "geom" | "wgsl" => Some(("// ", "")),
        // # style — scripting, config, data
        "rb" | "py" | "sh" | "bash" | "zsh" | "yaml" | "yml" | "toml" | "gd" | "pl" | "pm"
        | "r" | "R" | "ex" | "exs" | "nim" | "ps1" | "psm1" | "psd1" | "fish" | "nix" | "tf"
        | "tfvars" | "hcl" | "cmake" | "mk" | "env" | "conf" | "properties" | "graphql" | "gql"
        | "unity" | "prefab" | "asset" | "mat" | "meta" => Some(("# ", "")),
        // /* */ style — stylesheets
        "css" | "scss" | "sass" | "less" | "uss" => Some(("/* ", " */")),
        // <!-- --> style — markup, SFC
        "html" | "htm" | "xml" | "svg" | "vue" | "svelte" | "astro" | "uxml" | "xsl" | "xslt" => {
            Some(("<!-- ", " -->"))
        }
        // -- style — SQL, Lua, Haskell
        "sql" | "lua" | "hs" => Some(("-- ", "")),
        // ; style — Godot data, INI
        "tscn" | "tres" | "godot" | "ini" | "cfg" => Some(("; ", "")),
        // {# #} style — Jinja / Twig templates
        "j2" | "jinja" | "jinja2" | "twig" => Some(("{# ", " #}")),
        // <%# %> style — embedded templates
        "erb" | "ejs" => Some(("<%# ", " %>")),
        // {{!-- --}} style — Handlebars
        "hbs" | "handlebars" => Some(("{{!-- ", " --}}")),
        // @* *@ style — Razor
        "cshtml" => Some(("@* ", " *@")),
        // //- style — Pug / Jade
        "pug" | "jade" => Some(("//- ", "")),
        // -# style — Haml
        "haml" => Some(("-# ", "")),
        // unsupported
        _ => None,
    }
}

/// # Errors
///
/// Returns [`SpecreError`] on invalid arguments, I/O, or serialization failure.
pub fn execute(args: TagArgs, json: bool) -> Result<(), SpecreError> {
    if !ulid::is_valid(&args.ulid) {
        return Err(SpecreError::InvalidArgument(
            "invalid ULID format. Expected 26 uppercase alphanumeric characters.".to_string(),
        ));
    }

    let file_path = Path::new(&args.file);

    if !file_path.exists() {
        return Err(SpecreError::InvalidArgument(format!(
            "file not found: {}",
            to_forward_slash(file_path)
        )));
    }

    if file_path.is_dir() {
        return Err(SpecreError::InvalidArgument(format!(
            "'{}' is a directory, not a file",
            to_forward_slash(file_path)
        )));
    }

    let content = fs::read_to_string(file_path).map_err(|e| SpecreError::Io {
        path: file_path.to_path_buf(),
        source: e,
    })?;

    // Check if marker already exists
    let marker_pattern = format!("@specre {}", args.ulid);
    if content.contains(&marker_pattern) {
        if json {
            // Find the line number of the existing marker
            let line = content
                .lines()
                .position(|l| l.contains(&marker_pattern))
                .map_or(1, |n| n + 1);
            let output = TagOutput {
                id: args.ulid,
                file: to_forward_slash(file_path).into_owned(),
                line,
            };
            let json_str = serde_json::to_string_pretty(&output)?;
            println!("{json_str}");
        } else {
            println!("Marker already exists in {}", to_forward_slash(file_path));
        }
        return Ok(());
    }

    // Determine comment syntax from file extension
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let (prefix, suffix) = comment_syntax(ext).ok_or_else(|| {
        SpecreError::InvalidArgument(format!(
            "unsupported file extension '.{ext}' — comment syntax is unknown"
        ))
    })?;

    let marker_line = format!("{prefix}@specre {}{suffix}\n", args.ulid);
    let new_content = format!("{marker_line}{content}");

    fs::write(file_path, &new_content).map_err(|e| SpecreError::Io {
        path: file_path.to_path_buf(),
        source: e,
    })?;

    if json {
        let output = TagOutput {
            id: args.ulid,
            file: to_forward_slash(file_path).into_owned(),
            line: 1,
        };
        let json_str = serde_json::to_string_pretty(&output)?;
        println!("{json_str}");
    } else {
        println!("Tagged {} with {}", to_forward_slash(file_path), args.ulid);
    }

    Ok(())
}
