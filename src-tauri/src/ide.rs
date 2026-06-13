use std::path::Path;

use crate::models::Editor;

// (id, display name, .app path (relative to /Applications unless absolute), accent, CLI cmd)
const KNOWN: &[(&str, &str, &str, &str, &str)] = &[
    ("vscode",    "Visual Studio Code", "Visual Studio Code.app", "#2D9CDB", "code"),
    ("cursor",    "Cursor",             "Cursor.app",             "#E5E7EB", "cursor"),
    ("xcode",     "Xcode",              "Xcode.app",              "#1C7BFE", "xed"),
    ("zed",       "Zed",                "Zed.app",                "#0E7FE0", "zed"),
    ("jetbrains", "IntelliJ IDEA",      "IntelliJ IDEA.app",      "#FE315D", "idea"),
    ("terminal",  "Terminal",
        "/System/Applications/Utilities/Terminal.app", "#3FB950", "open -a Terminal"),
    ("finder",    "Finder",
        "/System/Library/CoreServices/Finder.app",     "#1C9CF6", "open"),
    ("sublime",   "Sublime Text",       "Sublime Text.app",       "#FF9800", "subl"),
];

pub fn detect_ides() -> Vec<Editor> {
    KNOWN
        .iter()
        .map(|(id, name, app, accent, cmd)| {
            let full_path = if app.starts_with('/') {
                app.to_string()
            } else {
                format!("/Applications/{}", app)
            };
            let detected = Path::new(&full_path).exists();
            Editor {
                id: id.to_string(),
                name: name.to_string(),
                detected,
                path: if detected { full_path } else { String::new() },
                accent: accent.to_string(),
                mark: make_mark(name),
                cmd: cmd.to_string(),
            }
        })
        .collect()
}

fn make_mark(name: &str) -> String {
    let words: Vec<&str> = name.split_whitespace().collect();
    match words.as_slice() {
        [] => "??".into(),
        [w] => w.chars().take(2).collect::<String>().to_uppercase(),
        [a, b, ..] => {
            let a = a.chars().next().unwrap_or(' ');
            let b = b.chars().next().unwrap_or(' ');
            format!("{}{}", a, b).to_uppercase()
        }
    }
}
