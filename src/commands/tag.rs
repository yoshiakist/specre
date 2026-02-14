// @specre 01KHB48EYB9686YYQMYFYQ5R1Z
use crate::cli::TagArgs;
use crate::ulid;
use std::fs;
use std::path::Path;

fn comment_syntax(ext: &str) -> Option<(&'static str, &'static str)> {
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

fn to_forward_slash(s: &str) -> String {
    s.replace('\\', "/")
}

pub fn execute(args: TagArgs) -> Result<(), String> {
    if !ulid::is_valid(&args.ulid) {
        return Err(
            "invalid ULID format. Expected 26 uppercase alphanumeric characters.".to_string(),
        );
    }

    let file_path = Path::new(&args.file);

    if !file_path.exists() {
        return Err(format!("file not found: {}", to_forward_slash(&args.file)));
    }

    if file_path.is_dir() {
        return Err(format!(
            "'{}' is a directory, not a file",
            to_forward_slash(&args.file)
        ));
    }

    let content = fs::read_to_string(file_path)
        .map_err(|e| format!("Failed to read '{}': {e}", args.file))?;

    // Check if marker already exists
    let marker_pattern = format!("@specre {}", args.ulid);
    if content.contains(&marker_pattern) {
        println!("Marker already exists in {}", to_forward_slash(&args.file));
        return Ok(());
    }

    // Determine comment syntax from file extension
    let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
    let (prefix, suffix) = comment_syntax(ext).ok_or_else(|| {
        format!(
            "unsupported file extension '.{}' — comment syntax is unknown",
            ext
        )
    })?;

    let marker_line = format!("{prefix}@specre {}{suffix}\n", args.ulid);
    let new_content = format!("{marker_line}{content}");

    fs::write(file_path, &new_content)
        .map_err(|e| format!("Failed to write '{}': {e}", args.file))?;

    println!("Tagged {} with {}", to_forward_slash(&args.file), args.ulid);

    Ok(())
}
