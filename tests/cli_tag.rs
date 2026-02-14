// @specre 01KHB48EYB9686YYQMYFYQ5R1Z
use assert_cmd::cargo::cargo_bin_cmd;
use assert_fs::TempDir;
use predicates::prelude::*;
use std::fs;

fn specre() -> assert_cmd::Command {
    cargo_bin_cmd!("specre")
}

fn write_source(dir: &std::path::Path, rel_path: &str, content: &str) {
    let path = dir.join(rel_path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

// -- Scenario: Basic invocation inserts marker at line 1 --

#[test]
fn tag_inserts_marker_for_rust_file() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "src/example.rs", "fn main() {}\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/example.rs"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Tagged src/example.rs with 01AAAAAAAAAAAAAAAAAAAAAAAA",
        ));

    let content = fs::read_to_string(tmp.path().join("src/example.rs")).unwrap();
    assert_eq!(
        content,
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn main() {}\n"
    );
}

// -- Scenario: Comment syntax varies by language --

#[test]
fn tag_uses_hash_comment_for_python() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "src/app.py", "print('hello')\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/app.py"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("src/app.py")).unwrap();
    assert!(content.starts_with("# @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n"));
}

#[test]
fn tag_uses_hash_comment_for_ruby() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "src/app.rb", "puts 'hello'\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/app.rb"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("src/app.rb")).unwrap();
    assert!(content.starts_with("# @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n"));
}

#[test]
fn tag_uses_block_comment_for_css() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "src/style.css", "body { color: red; }\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/style.css"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("src/style.css")).unwrap();
    assert!(content.starts_with("/* @specre 01AAAAAAAAAAAAAAAAAAAAAAAA */\n"));
}

#[test]
fn tag_uses_html_comment_for_html() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "src/page.html", "<html></html>\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/page.html"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("src/page.html")).unwrap();
    assert!(content.starts_with("<!-- @specre 01AAAAAAAAAAAAAAAAAAAAAAAA -->\n"));
}

#[test]
fn tag_uses_dash_dash_comment_for_sql() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "src/query.sql", "SELECT 1;\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/query.sql"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("src/query.sql")).unwrap();
    assert!(content.starts_with("-- @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n"));
}

#[test]
fn tag_rejects_unsupported_extension() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "src/file.xyz", "data\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/file.xyz"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unsupported file extension '.xyz'",
        ));

    // File must not be modified
    let content = fs::read_to_string(tmp.path().join("src/file.xyz")).unwrap();
    assert_eq!(content, "data\n");
}

// -- Scenario: Marker already exists in file --

#[test]
fn tag_already_exists_does_not_modify() {
    let tmp = TempDir::new().unwrap();
    let original = "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nfn main() {}\n";
    write_source(tmp.path(), "src/example.rs", original);

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/example.rs"])
        .current_dir(tmp.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Marker already exists in src/example.rs",
        ));

    let content = fs::read_to_string(tmp.path().join("src/example.rs")).unwrap();
    assert_eq!(content, original);
}

// -- Scenario: File does not exist --

#[test]
fn tag_file_not_found() {
    let tmp = TempDir::new().unwrap();

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/nonexistent.rs"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "file not found: src/nonexistent.rs",
        ));
}

// -- Scenario: Invalid ULID format --

#[test]
fn tag_invalid_ulid() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "src/example.rs", "fn main() {}\n");

    specre()
        .args(["tag", "abc123", "src/example.rs"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "invalid ULID format. Expected 26 uppercase alphanumeric characters.",
        ));
}

// -- Scenario: Target is a directory --

#[test]
fn tag_target_is_directory() {
    let tmp = TempDir::new().unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains("is a directory, not a file"));
}

// -- Scenario: Preserves existing file content --

#[test]
fn tag_preserves_existing_content() {
    let tmp = TempDir::new().unwrap();
    let original = "line1\nline2\nline3\n";
    write_source(tmp.path(), "src/example.js", original);

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/example.js"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("src/example.js")).unwrap();
    assert_eq!(
        content,
        "// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\nline1\nline2\nline3\n"
    );
}

// -- Scenario: Paths use forward slashes in output --

#[test]
fn tag_output_uses_forward_slashes() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "src/nested/deep/example.rs", "fn main() {}\n");

    let output = specre()
        .args([
            "tag",
            "01AAAAAAAAAAAAAAAAAAAAAAAA",
            "src/nested/deep/example.rs",
        ])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("src/nested/deep/example.rs"),
        "Output should contain forward-slash paths"
    );
    assert!(
        !stdout.contains('\\'),
        "Output should not contain backslashes"
    );
}

// -- Scenario: Hash comment for shell files --

#[test]
fn tag_uses_hash_comment_for_shell() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "scripts/deploy.sh", "#!/bin/bash\necho hi\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "scripts/deploy.sh"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("scripts/deploy.sh")).unwrap();
    assert!(content.starts_with("# @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n"));
}

// -- Scenario: Hash comment for YAML files --

#[test]
fn tag_uses_hash_comment_for_yaml() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "config/settings.yml", "key: value\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "config/settings.yml"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("config/settings.yml")).unwrap();
    assert!(content.starts_with("# @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n"));
}

// -- Scenario: Godot GDScript uses # --

#[test]
fn tag_uses_hash_comment_for_gdscript() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "src/player.gd", "extends Node2D\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/player.gd"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("src/player.gd")).unwrap();
    assert!(content.starts_with("# @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n"));
}

// -- Scenario: Godot scene file uses ; --

#[test]
fn tag_uses_semicolon_comment_for_godot_tscn() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "scenes/main.tscn", "[gd_scene format=3]\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "scenes/main.tscn"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("scenes/main.tscn")).unwrap();
    assert!(content.starts_with("; @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n"));
}

// -- Scenario: Godot resource file uses ; --

#[test]
fn tag_uses_semicolon_comment_for_godot_tres() {
    let tmp = TempDir::new().unwrap();
    write_source(
        tmp.path(),
        "resources/data.tres",
        "[gd_resource format=3]\n",
    );

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "resources/data.tres"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("resources/data.tres")).unwrap();
    assert!(content.starts_with("; @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n"));
}

// -- Scenario: INI file uses ; --

#[test]
fn tag_uses_semicolon_comment_for_ini() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "config/settings.ini", "[section]\nkey=value\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "config/settings.ini"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("config/settings.ini")).unwrap();
    assert!(content.starts_with("; @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n"));
}

// -- Scenario: Lua uses -- --

#[test]
fn tag_uses_dash_dash_comment_for_lua() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "scripts/init.lua", "print('hello')\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "scripts/init.lua"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("scripts/init.lua")).unwrap();
    assert!(content.starts_with("-- @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n"));
}

// -- Scenario: Vue SFC uses <!-- --> --

#[test]
fn tag_uses_html_comment_for_vue() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "src/App.vue", "<template></template>\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/App.vue"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("src/App.vue")).unwrap();
    assert!(content.starts_with("<!-- @specre 01AAAAAAAAAAAAAAAAAAAAAAAA -->\n"));
}

// -- Scenario: Svelte uses <!-- --> --

#[test]
fn tag_uses_html_comment_for_svelte() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "src/App.svelte", "<script></script>\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/App.svelte"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("src/App.svelte")).unwrap();
    assert!(content.starts_with("<!-- @specre 01AAAAAAAAAAAAAAAAAAAAAAAA -->\n"));
}

// -- Scenario: Jinja2 template uses {# #} --

#[test]
fn tag_uses_jinja_comment_for_j2() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "templates/base.j2", "{{ content }}\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "templates/base.j2"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("templates/base.j2")).unwrap();
    assert!(content.starts_with("{# @specre 01AAAAAAAAAAAAAAAAAAAAAAAA #}\n"));
}

// -- Scenario: Twig template uses {# #} --

#[test]
fn tag_uses_jinja_comment_for_twig() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "templates/base.twig", "{{ content }}\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "templates/base.twig"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("templates/base.twig")).unwrap();
    assert!(content.starts_with("{# @specre 01AAAAAAAAAAAAAAAAAAAAAAAA #}\n"));
}

// -- Scenario: ERB template uses <%# %> --

#[test]
fn tag_uses_erb_comment_for_erb() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "views/index.erb", "<%= @title %>\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "views/index.erb"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("views/index.erb")).unwrap();
    assert!(content.starts_with("<%# @specre 01AAAAAAAAAAAAAAAAAAAAAAAA %>\n"));
}

// -- Scenario: EJS template uses <%# %> --

#[test]
fn tag_uses_erb_comment_for_ejs() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "views/index.ejs", "<%= title %>\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "views/index.ejs"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("views/index.ejs")).unwrap();
    assert!(content.starts_with("<%# @specre 01AAAAAAAAAAAAAAAAAAAAAAAA %>\n"));
}

// -- Scenario: Handlebars uses {{!-- --}} --

#[test]
fn tag_uses_handlebars_comment_for_hbs() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "views/index.hbs", "{{content}}\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "views/index.hbs"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("views/index.hbs")).unwrap();
    assert!(content.starts_with("{{!-- @specre 01AAAAAAAAAAAAAAAAAAAAAAAA --}}\n"));
}

// -- Scenario: Razor uses @* *@ --

#[test]
fn tag_uses_razor_comment_for_cshtml() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "Views/Index.cshtml", "@model string\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "Views/Index.cshtml"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("Views/Index.cshtml")).unwrap();
    assert!(content.starts_with("@* @specre 01AAAAAAAAAAAAAAAAAAAAAAAA *@\n"));
}

// -- Scenario: Pug uses //- --

#[test]
fn tag_uses_pug_comment_for_pug() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "views/index.pug", "h1 Hello\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "views/index.pug"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("views/index.pug")).unwrap();
    assert!(content.starts_with("//- @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n"));
}

// -- Scenario: Haml uses -# --

#[test]
fn tag_uses_haml_comment_for_haml() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "views/index.haml", "%h1 Hello\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "views/index.haml"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("views/index.haml")).unwrap();
    assert!(content.starts_with("-# @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n"));
}

// -- Scenario: Unity shader uses // --

#[test]
fn tag_uses_slash_slash_for_unity_shader() {
    let tmp = TempDir::new().unwrap();
    write_source(
        tmp.path(),
        "Shaders/Custom.shader",
        "Shader \"Custom\" {}\n",
    );

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "Shaders/Custom.shader"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("Shaders/Custom.shader")).unwrap();
    assert!(content.starts_with("// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n"));
}

// -- Scenario: Unity meta (YAML) uses # --

#[test]
fn tag_uses_hash_comment_for_unity_meta() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "Assets/Script.cs.meta", "guid: abc123\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "Assets/Script.cs.meta"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("Assets/Script.cs.meta")).unwrap();
    assert!(content.starts_with("# @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n"));
}

// -- Scenario: Unity USS uses /* */ --

#[test]
fn tag_uses_block_comment_for_unity_uss() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "UI/style.uss", ".label { color: white; }\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "UI/style.uss"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("UI/style.uss")).unwrap();
    assert!(content.starts_with("/* @specre 01AAAAAAAAAAAAAAAAAAAAAAAA */\n"));
}

// -- Scenario: Godot shader uses // --

#[test]
fn tag_uses_slash_slash_for_gdshader() {
    let tmp = TempDir::new().unwrap();
    write_source(
        tmp.path(),
        "shaders/effect.gdshader",
        "shader_type canvas_item;\n",
    );

    specre()
        .args([
            "tag",
            "01AAAAAAAAAAAAAAAAAAAAAAAA",
            "shaders/effect.gdshader",
        ])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("shaders/effect.gdshader")).unwrap();
    assert!(content.starts_with("// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n"));
}

// -- Scenario: PHP uses // --

#[test]
fn tag_uses_slash_slash_for_php() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "src/index.php", "<?php echo 'hi'; ?>\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "src/index.php"])
        .current_dir(tmp.path())
        .assert()
        .success();

    let content = fs::read_to_string(tmp.path().join("src/index.php")).unwrap();
    assert!(content.starts_with("// @specre 01AAAAAAAAAAAAAAAAAAAAAAAA\n"));
}

// -- Scenario: Unsupported extension is rejected --

#[test]
fn tag_unsupported_extension_does_not_modify_file() {
    let tmp = TempDir::new().unwrap();
    write_source(tmp.path(), "data/file.bin", "\x00\x01\x02\n");

    specre()
        .args(["tag", "01AAAAAAAAAAAAAAAAAAAAAAAA", "data/file.bin"])
        .current_dir(tmp.path())
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unsupported file extension '.bin'",
        ));

    let content = fs::read_to_string(tmp.path().join("data/file.bin")).unwrap();
    assert_eq!(content, "\x00\x01\x02\n");
}
