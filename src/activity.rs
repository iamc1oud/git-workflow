#![allow(non_snake_case)]

use dioxus::prelude::*;

use crate::{invoke, models::{ActivityItem, from_now}};

const AVATAR_COLORS: &[&str] = &[
    "#7C6BFF", "#FF6B6B", "#4ECDC4", "#45B7D1",
    "#96CEB4", "#E17055", "#A29BFE", "#FD79A8",
    "#FECA57", "#54A0FF", "#5F27CD", "#48DBFB",
];

fn author_color(name: &str) -> &'static str {
    let h: usize = name.bytes().fold(0usize, |a, b| a.wrapping_add(b as usize));
    AVATAR_COLORS[h % AVATAR_COLORS.len()]
}

fn author_initials(name: &str) -> String {
    let parts: Vec<&str> = name.split_whitespace().collect();
    match parts.len() {
        0 => "?".into(),
        1 => name.chars().take(2).collect::<String>().to_uppercase(),
        _ => {
            let f = parts[0].chars().next().unwrap_or('?');
            let l = parts[parts.len() - 1].chars().next().unwrap_or('?');
            format!("{f}{l}").to_uppercase()
        }
    }
}

fn time_bucket(iso: &Option<String>) -> u8 {
    let iso = match iso {
        Some(s) if !s.is_empty() => s,
        _ => return 3,
    };
    let now = js_sys::Date::now();
    let dt = js_sys::Date::new(&wasm_bindgen::JsValue::from_str(iso));
    let diff_ms = now - dt.get_time();
    let diff_h = diff_ms / 3_600_000.0;
    if diff_h < 24.0 { 0 }
    else if diff_h < 48.0 { 1 }
    else if diff_ms < 7.0 * 86_400_000.0 { 2 }
    else { 3 }
}

#[component]
pub fn ActivityView() -> Element {
    let mut items: Signal<Vec<ActivityItem>> = use_signal(Vec::new);

    use_effect(move || {
        spawn(async move {
            items.set(invoke::list_recent_activity().await);
        });
    });

    let data = items.read().clone();

    let mut today: Vec<ActivityItem> = Vec::new();
    let mut yesterday: Vec<ActivityItem> = Vec::new();
    let mut this_week: Vec<ActivityItem> = Vec::new();
    let mut older: Vec<ActivityItem> = Vec::new();

    for item in data {
        match time_bucket(&item.date) {
            0 => today.push(item),
            1 => yesterday.push(item),
            2 => this_week.push(item),
            _ => older.push(item),
        }
    }

    let is_empty = today.is_empty() && yesterday.is_empty() && this_week.is_empty() && older.is_empty();

    rsx! {
        div { class: "list act-feed scroll",
            div { class: "act-page-header",
                h2 { class: "act-title", "Activity" }
                p { class: "act-subtitle", "Commits across all repositories" }
            }

            if is_empty {
                div { class: "list-empty",
                    p { "No commits yet. Add repositories to see activity." }
                }
            }

            if !today.is_empty() {
                CommitGroup { label: "Today", items: today }
            }
            if !yesterday.is_empty() {
                CommitGroup { label: "Yesterday", items: yesterday }
            }
            if !this_week.is_empty() {
                CommitGroup { label: "This Week", items: this_week }
            }
            if !older.is_empty() {
                CommitGroup { label: "Older", items: older }
            }
        }
    }
}

#[component]
fn CommitGroup(label: String, items: Vec<ActivityItem>) -> Element {
    rsx! {
        div { class: "act-group",
            div { class: "act-group-label", "{label}" }
            for item in items {
                CommitRow { item }
            }
        }
    }
}

#[component]
fn CommitRow(item: ActivityItem) -> Element {
    let color = author_color(&item.author);
    let initials = author_initials(&item.author);
    let when = from_now(&item.date);
    let hash_short: String = item.hash.chars().take(7).collect();

    rsx! {
        div { class: "act-commit-row",
            div { class: "act-avatar", style: "background:{color}", "{initials}" }
            div { class: "act-body",
                div { class: "act-commit-msg", "{item.msg}" }
                div { class: "act-meta-row",
                    span { class: "act-repo-name mono", "{item.repo_name}" }
                    span { class: "act-sep", "·" }
                    span { class: "act-author", "{item.author}" }
                    span { class: "act-sep", "·" }
                    span { class: "act-hash mono", "{hash_short}" }
                }
            }
            div { class: "act-right",
                span { class: "act-when", "{when}" }
                div { class: "act-diff",
                    span { class: "adds", "+{item.additions}" }
                    span { class: "dels", "-{item.deletions}" }
                }
            }
        }
    }
}
