use dioxus::prelude::*;

use crate::{icons::icon_html, models::*};

// ── Icon component (renders SVG via dangerous_inner_html) ─────────────────────

#[component]
pub fn Icon(name: &'static str, size: u32, class: Option<String>) -> Element {
    let html = icon_html(name, size);
    rsx! {
        span {
            class: "icon-inline {class.as_deref().unwrap_or(\"\")}",
            dangerous_inner_html: "{html}"
        }
    }
}

// ── Avatar ────────────────────────────────────────────────────────────────────

#[component]
pub fn Avatar(initials: String, color: String, name: String, size: u32) -> Element {
    let font_size = (size as f32 * 0.42) as u32;
    rsx! {
        div {
            class: "av",
            title: "{name}",
            style: "width:{size}px;height:{size}px;background:{color};font-size:{font_size}px",
            "{initials}"
        }
    }
}

#[component]
pub fn AvatarStack(contributors: Vec<Contributor>, max: usize, size: u32) -> Element {
    let shown = contributors.iter().take(max).cloned().collect::<Vec<_>>();
    let extra = contributors.len().saturating_sub(max);
    let font_size = (size as f32 * 0.4) as u32;
    let extra_font_size = (size as f32 * 0.36) as u32;
    rsx! {
        div { class: "av-stack",
            for c in shown {
                {
                    let color = c.color();
                    let initials = c.initials();
                    rsx! {
                        div {
                            class: "av",
                            title: "{c.name}",
                            style: "width:{size}px;height:{size}px;background:{color};font-size:{font_size}px",
                            "{initials}"
                        }
                    }
                }
            }
            if extra > 0 {
                div {
                    class: "av",
                    style: "width:{size}px;height:{size}px;background:var(--s4);color:var(--text-2);font-size:{extra_font_size}px",
                    "+{extra}"
                }
            }
        }
    }
}

// ── StatusPill ────────────────────────────────────────────────────────────────

#[component]
pub fn StatusPill(repo: RepoSummary) -> Element {
    let (kind, label) = status_kind(&repo);
    let icon = match kind {
        "clean" => "check",
        "dirty" => "diff",
        "ahead" => "arrowUp",
        _ => "arrowDown",
    };
    let html = icon_html(icon, 11);
    rsx! {
        span { class: "pill {kind}",
            span { dangerous_inner_html: "{html}" }
            "{label}"
        }
    }
}

// ── Trait for initials ────────────────────────────────────────────────────────

pub trait Initials {
    fn initials(&self) -> String;
}

impl Initials for Contributor {
    fn initials(&self) -> String {
        self.name
            .split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase()
    }
}

impl Contributor {
    pub fn color(&self) -> String {
        // Derive a color from the handle deterministically
        let colors = [
            "#6366F1", "#EC4899", "#14B8A6", "#F59E0B",
            "#8B5CF6", "#06B6D4", "#F43F5E", "#22C55E",
        ];
        let idx = self.handle.bytes().fold(0usize, |a, b| a.wrapping_add(b as usize)) % colors.len();
        colors[idx].to_string()
    }
}

// ── Toast ─────────────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub struct Toast {
    pub id: String,
    pub msg: String,
    pub mono: String,
}

impl Toast {
    pub fn new(msg: impl Into<String>, mono: impl Into<String>) -> Self {
        Self {
            id: format!("{}", js_sys::Date::now() as u64),
            msg: msg.into(),
            mono: mono.into(),
        }
    }
}
