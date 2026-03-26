use std::{env, fs, path::Path};

fn main() {
    embuild::espidf::sysenv::output();

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set, is this being run by cargo?");
    let out_dir = Path::new(&out_dir);

    let html =
        fs::read_to_string("./src/web/index.html").expect("failed to read src/web/index.html");
    let css = fs::read_to_string("./src/web/style.css").expect("failed to read src/web/style.css");
    let js = fs::read_to_string("./src/web/app.js").expect("failed to read src/web/app.js");
    let js = strip_dev_block(&js, "// START_DEV", "// END_DEV");

    let combined = html
        .replace(
            r#"<link rel="stylesheet" href="style.css" />"#,
            &format!("<style>{css}</style>"),
        )
        .replace(
            r#"<script src="app.js"></script>"#,
            &format!("<script>{js}</script>"),
        );

    let minified = minify_html::minify(
        combined.as_bytes(),
        &minify_html::Cfg {
            minify_css: true,
            minify_js: true,
            ..Default::default()
        },
    );

    assert!(
        !minified.is_empty(),
        "minify_html produced empty output — check that src/web/index.html is well-formed"
    );

    fs::write(out_dir.join("ac.min.html"), &minified).unwrap();

    fs::copy("./src/web/manifest.json", out_dir.join("manifest.json"))
        .expect("failed to copy src/web/manifest.json");
    fs::copy("./src/web/icon.png", out_dir.join("icon.png"))
        .expect("failed to copy src/web/icon.png");

    println!("cargo:rerun-if-changed=./src/web/index.html");
    println!("cargo:rerun-if-changed=./src/web/style.css");
    println!("cargo:rerun-if-changed=./src/web/app.js");
    println!("cargo:rerun-if-changed=./src/web/manifest.json");
    println!("cargo:rerun-if-changed=./src/web/icon.png");
}

fn strip_dev_block(src: &str, start_marker: &str, end_marker: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut skipping = false;
    let mut found_start = false;
    let mut found_end = false;

    for line in src.lines() {
        let trimmed = line.trim_start();
        if !skipping && trimmed.starts_with(start_marker) {
            skipping = true;
            found_start = true;
            continue;
        }
        if skipping {
            if trimmed.starts_with(end_marker) {
                skipping = false;
                found_end = true;
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }

    assert!(
        !found_start || !skipping,
        "Found '{start_marker}' in app.js but no matching '{end_marker}' | \
         dev block is unterminated"
    );
    assert!(
        found_end == found_start,
        "Found '{end_marker}' in app.js but no matching '{start_marker}' | \
         dev block start marker is missing"
    );

    out
}
