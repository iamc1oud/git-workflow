use dioxus::prelude::*;

use crate::{
    icons::icon_html,
    invoke,
    models::*,
    shared::Toast,
};

#[derive(Clone, PartialEq)]
enum PaletteMode {
    Search,
    NewFolder,
    AddRepo(String), // folder_id
    ScanDir,
}

#[component]
pub fn CommandPalette(
    repos: Vec<RepoSummary>,
    folders: Vec<Folder>,
    on_close: EventHandler,
    on_open_repo: EventHandler<RepoSummary>,
    on_refresh: EventHandler,
    toasts: Signal<Vec<Toast>>,
) -> Element {
    let mut query = use_signal(|| String::new());
    let mut mode = use_signal(|| PaletteMode::Search);
    let mut selected_idx = use_signal(|| 0usize);

    // Extra inputs for sub-modes
    let mut folder_name = use_signal(|| String::new());
    let mut folder_color = use_signal(|| "#7C6BFF".to_string());
    let mut repo_path = use_signal(|| String::new());
    let mut scan_results = use_signal(|| Vec::<String>::new());
    let mut scanning = use_signal(|| false);

    let q = query.read().to_lowercase();
    let current_mode = mode.read().clone();

    let filtered_repos: Vec<RepoSummary> = repos
        .iter()
        .filter(|r| q.is_empty() || r.name.to_lowercase().contains(&q) || r.path.to_lowercase().contains(&q))
        .cloned()
        .collect();

    let commands: Vec<(&str, &str, &str)> = vec![
        ("New folder…", "new-folder", "folder"),
        ("Add repo to folder…", "add-repo", "plus"),
        ("Scan directory for repos…", "scan", "search"),
    ];

    let filtered_cmds: Vec<_> = commands
        .iter()
        .filter(|(label, _, _)| q.is_empty() || label.to_lowercase().contains(&q))
        .collect();

    rsx! {
        div {
            class: "cmdk-overlay",
            onclick: move |_| on_close.call(()),
            div {
                class: "cmdk",
                onclick: |e| e.stop_propagation(),

                // Input
                div { class: "cmdk-inp-wrap",
                    span { class: "cmdk-ico", dangerous_inner_html: "{icon_html(\"search\", 15)}" }
                    match current_mode.clone() {
                        PaletteMode::NewFolder => rsx! {
                            input {
                                class: "cmdk-inp",
                                placeholder: "Folder name…",
                                autofocus: true,
                                value: "{folder_name}",
                                oninput: move |e| folder_name.set(e.value().clone()),
                                onkeydown: move |e| {
                                    if e.key() == Key::Escape { mode.set(PaletteMode::Search); }
                                },
                            }
                        },
                        PaletteMode::AddRepo(_) => rsx! {
                            div { class: "cmdk-path-row",
                                span { class: "cmdk-path-val mono",
                                    if repo_path.read().is_empty() {
                                        span { class: "cmdk-path-placeholder", "No folder selected" }
                                    } else {
                                        "{repo_path}"
                                    }
                                }
                                button {
                                    class: "cmdk-browse-btn",
                                    onclick: move |_| {
                                        spawn(async move {
                                            if let Some(p) = invoke::pick_folder().await {
                                                repo_path.set(p);
                                            }
                                        });
                                    },
                                    span { dangerous_inner_html: "{icon_html(\"folder\", 13)}" }
                                    "Browse…"
                                }
                            }
                        },
                        PaletteMode::ScanDir => rsx! {
                            div { class: "cmdk-path-row",
                                span { class: "cmdk-path-val mono",
                                    if repo_path.read().is_empty() {
                                        span { class: "cmdk-path-placeholder", "No folder selected" }
                                    } else {
                                        "{repo_path}"
                                    }
                                }
                                button {
                                    class: "cmdk-browse-btn",
                                    onclick: move |_| {
                                        spawn(async move {
                                            if let Some(p) = invoke::pick_folder().await {
                                                repo_path.set(p);
                                            }
                                        });
                                    },
                                    span { dangerous_inner_html: "{icon_html(\"folder\", 13)}" }
                                    "Browse…"
                                }
                            }
                        },
                        PaletteMode::Search => rsx! {
                            input {
                                class: "cmdk-inp",
                                placeholder: "Search repos or type a command…",
                                autofocus: true,
                                value: "{query}",
                                oninput: move |e| query.set(e.value().clone()),
                                onkeydown: move |e| {
                                    if e.key() == Key::Escape { on_close.call(()); }
                                },
                            }
                        },
                    }
                    button {
                        class: "cmdk-close",
                        onclick: move |_| on_close.call(()),
                        span { dangerous_inner_html: "{icon_html(\"x\", 14)}" }
                    }
                }

                // Body
                div { class: "cmdk-body",
                    match current_mode.clone() {
                        PaletteMode::NewFolder => rsx! {
                            div { class: "cmdk-section",
                                div { class: "cmdk-sec-h", "New Folder" }
                                div { class: "cmdk-form",
                                    div { class: "cmdk-row",
                                        span { class: "cmdk-lbl", "Name" }
                                        span { class: "cmdk-val", "{folder_name}" }
                                    }
                                    div { class: "cmdk-row",
                                        span { class: "cmdk-lbl", "Color" }
                                        div { class: "color-row",
                                            for c in ["#7C6BFF","#EC4899","#14B8A6","#F59E0B","#EF4444","#22C55E","#06B6D4","#F97316"] {
                                                {
                                                    let color_str = c.to_string();
                                                    let is_sel = *folder_color.read() == c;
                                                    rsx! {
                                                        span {
                                                            class: if is_sel { "color-dot sel" } else { "color-dot" },
                                                            style: "background:{c}",
                                                            onclick: move |_| folder_color.set(color_str.clone()),
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    button {
                                        class: "btn-primary w-full",
                                        onclick: move |_| {
                                            let name = folder_name.read().trim().to_string();
                                            if name.is_empty() { return; }
                                            let color = folder_color.read().clone();
                                            spawn(async move {
                                                invoke::add_folder(name.clone(), "folder".into(), color).await;
                                                on_refresh.call(());
                                                on_close.call(());
                                            });
                                        },
                                        "Create Folder"
                                    }
                                }
                            }
                        },

                        PaletteMode::AddRepo(ref fid) => {
                            let fid = fid.clone();
                            let folder = folders.iter().find(|f| f.id == fid).cloned();
                            rsx! {
                                div { class: "cmdk-section",
                                    div { class: "cmdk-sec-h",
                                        "Add Repo to "
                                        if let Some(f) = folder {
                                            span { style: "color:{f.color}", "{f.name}" }
                                        }
                                    }
                                    div { class: "cmdk-form",
                                        div { class: "cmdk-row",
                                            span { class: "cmdk-lbl mono", "Path" }
                                            span { class: "cmdk-val mono", "{repo_path}" }
                                        }
                                        button {
                                            class: "btn-primary w-full",
                                            onclick: move |_| {
                                                let path = repo_path.read().trim().to_string();
                                                if path.is_empty() { return; }
                                                let fid2 = fid.clone();
                                                spawn(async move {
                                                    let result = invoke::add_repo(fid2, path.clone(), String::new()).await;
                                                    let mut t = toasts.write();
                                                    if result.is_some() {
                                                        t.push(Toast::new("Repo added", &path));
                                                        on_refresh.call(());
                                                        on_close.call(());
                                                    } else {
                                                        t.push(Toast::new("Failed", "Not a valid git repo or path not found"));
                                                    }
                                                });
                                            },
                                            "Add Repository"
                                        }
                                    }
                                }
                            }
                        },

                        PaletteMode::ScanDir => rsx! {
                            div { class: "cmdk-section",
                                div { class: "cmdk-sec-h", "Scan for Repos" }
                                div { class: "cmdk-form",
                                    button {
                                        class: "btn-primary",
                                        disabled: *scanning.read(),
                                        onclick: move |_| {
                                            let parent = repo_path.read().trim().to_string();
                                            if parent.is_empty() { return; }
                                            scanning.set(true);
                                            spawn(async move {
                                                let results = invoke::scan_dir(parent).await;
                                                scan_results.set(results);
                                                scanning.set(false);
                                            });
                                        },
                                        if *scanning.read() { "Scanning…" } else { "Scan" }
                                    }
                                }
                                if !scan_results.read().is_empty() {
                                    div { class: "scan-results",
                                        div { class: "cmdk-sec-h", "Found {scan_results.read().len()} repos" }
                                        for path in scan_results.read().iter() {
                                            div { class: "scan-row",
                                                span { class: "mono", "{path}" }
                                            }
                                        }
                                    }
                                }
                            }
                        },

                        PaletteMode::Search => rsx! {
                            // Repos
                            if !filtered_repos.is_empty() {
                                div { class: "cmdk-section",
                                    div { class: "cmdk-sec-h", "Repositories" }
                                    for r in filtered_repos.iter().take(8) {
                                        {
                                            let r2 = r.clone();
                                            let r3 = r.clone();
                                            rsx! {
                                                div {
                                                    class: "cmdk-item",
                                                    onclick: move |_| {
                                                        on_open_repo.call(r2.clone());
                                                        on_close.call(());
                                                    },
                                                    span { dangerous_inner_html: "{icon_html(\"archive\", 14)}" }
                                                    span { class: "cmdk-item-name", "{r3.name}" }
                                                    span { class: "cmdk-item-sub mono", "{r3.path}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }

                            // Commands
                            if !filtered_cmds.is_empty() {
                                {
                                    let first_folder_id = folders.first().map(|f| f.id.clone()).unwrap_or_default();
                                    let has_folders = !folders.is_empty();
                                    rsx! {
                                div { class: "cmdk-section",
                                    div { class: "cmdk-sec-h", "Commands" }
                                    for (label, cmd_id, ico) in filtered_cmds.iter() {
                                        {
                                            let label = *label;
                                            let cmd_id = *cmd_id;
                                            let ico = *ico;
                                            let fid_for_click = first_folder_id.clone();
                                            rsx! {
                                                div {
                                                    class: "cmdk-item",
                                                    onclick: move |_| {
                                                        match cmd_id {
                                                            "new-folder" => mode.set(PaletteMode::NewFolder),
                                                            "add-repo" => mode.set(PaletteMode::AddRepo(fid_for_click.clone())),
                                                            "scan" => mode.set(PaletteMode::ScanDir),
                                                            _ => {}
                                                        }
                                                    },
                                                    span { dangerous_inner_html: "{icon_html(ico, 14)}" }
                                                    span { class: "cmdk-item-name", "{label}" }
                                                }
                                            }
                                        }
                                    }

                                    // Per-folder add repo
                                    if has_folders {
                                        for f in folders.iter() {
                                            {
                                                let fid = f.id.clone();
                                                let fname = f.name.clone();
                                                let fcolor = f.color.clone();
                                                rsx! {
                                                    div {
                                                        class: "cmdk-item",
                                                        onclick: move |_| {
                                                            mode.set(PaletteMode::AddRepo(fid.clone()));
                                                        },
                                                        span { class: "fdot", style: "background:{fcolor}" }
                                                        span { class: "cmdk-item-name", "Add repo to {fname}" }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                    }
                                }
                            }

                            if filtered_repos.is_empty() && filtered_cmds.is_empty() {
                                div { class: "cmdk-empty", "No results for "{q}"" }
                            }
                        },
                    }
                }

                // Footer hint
                div { class: "cmdk-foot",
                    span { class: "kbd", "↵" } "select  "
                    span { class: "kbd", "esc" } "close"
                }
            }
        }
    }
}
