// Lucide-style SVG icon paths. All paths are 24x24 viewBox.
// Multiple sub-paths separated by M for compound shapes.

const ICONS: &[(&str, &str)] = &[
    ("search",       "M11 11m-7 0a7 7 0 1014 0a7 7 0 10-14 0M21 21l-4.3-4.3"),
    ("server",       "M3 4h18v6H3zM3 14h18v6H3zM7 7h.01M7 17h.01"),
    ("browser",      "M3 4h18v16H3zM3 9h18"),
    ("phone",        "M7 2h10v20H7zM11 18h2"),
    ("cloud",        "M6 17a4 4 0 010-8 5 5 0 019.6-1A4 4 0 0118 17H6z"),
    ("flask",        "M9 3h6M10 3v6l-5 9a2 2 0 002 3h10a2 2 0 002-3l-5-9V3M6.5 14h11"),
    ("star",         "M12 3l2.6 5.6 6.1.8-4.5 4.2 1.2 6L12 17l-5.4 2.6 1.2-6L3.3 9.4l6.1-.8z"),
    ("branch",       "M6 3v12M6 21a3 3 0 100-6 3 3 0 000 6zM6 6a3 3 0 100-6 3 3 0 000 6zM18 9a3 3 0 100-6 3 3 0 000 6zM18 6c0 4-6 3-6 9"),
    ("commit",       "M12 12m-4 0a4 4 0 108 0a4 4 0 10-8 0M3 12h5M16 12h5"),
    ("clock",        "M12 12m-9 0a9 9 0 1018 0a9 9 0 10-18 0M12 7v5l3 2"),
    ("folder",       "M3 7a2 2 0 012-2h4l2 2h8a2 2 0 012 2v8a2 2 0 01-2 2H5a2 2 0 01-2-2z"),
    ("settings",     "M12 15a3 3 0 100-6 3 3 0 000 6zM19.4 15a1.6 1.6 0 00.3 1.8l.1.1a2 2 0 11-2.8 2.8l-.1-.1a1.6 1.6 0 00-2.7 1.1V21a2 2 0 11-4 0v-.1a1.6 1.6 0 00-2.7-1.1l-.1.1a2 2 0 11-2.8-2.8l.1-.1a1.6 1.6 0 00-1.1-2.7H3a2 2 0 110-4h.1a1.6 1.6 0 001.1-2.7l-.1-.1a2 2 0 112.8-2.8l.1.1a1.6 1.6 0 001.8.3h.1A1.6 1.6 0 0010 3.1V3a2 2 0 014 0v.1a1.6 1.6 0 002.7 1.1l.1-.1a2 2 0 112.8 2.8l-.1.1a1.6 1.6 0 00-.3 1.8v.1a1.6 1.6 0 001.5 1H21a2 2 0 010 4h-.1a1.6 1.6 0 00-1.5 1z"),
    ("plus",         "M12 5v14M5 12h14"),
    ("chevronDown",  "M6 9l6 6 6-6"),
    ("chevronRight", "M9 6l6 6-6 6"),
    ("check",        "M20 6L9 17l-5-5"),
    ("terminal",     "M4 5h16v14H4zM7 9l3 3-3 3M13 15h4"),
    ("external",     "M15 3h6v6M10 14L21 3M18 13v6a2 2 0 01-2 2H5a2 2 0 01-2-2V8a2 2 0 012-2h6"),
    ("arrowDown",    "M12 5v14M19 12l-7 7-7-7"),
    ("arrowUp",      "M12 19V5M5 12l7-7 7 7"),
    ("diff",         "M12 3v18M3 7.5h6M6 4.5v6M15 16.5h6"),
    ("users",        "M16 21v-2a4 4 0 00-4-4H6a4 4 0 00-4 4v2M9 11a4 4 0 100-8 4 4 0 000 8zM22 21v-2a4 4 0 00-3-3.9M16 3.1a4 4 0 010 7.8"),
    ("book",         "M4 19.5A2.5 2.5 0 016.5 17H20M4 19.5A2.5 2.5 0 016.5 22H20V2H6.5A2.5 2.5 0 004 4.5z"),
    ("grid",         "M3 3h8v8H3zM13 3h8v8h-8zM13 13h8v8h-8zM3 13h8v8H3z"),
    ("list",         "M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01"),
    ("sun",          "M12 12m-4 0a4 4 0 108 0a4 4 0 10-8 0M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"),
    ("moon",         "M21 12.8A9 9 0 1111.2 3a7 7 0 009.8 9.8z"),
    ("x",            "M18 6L6 18M6 6l12 12"),
    ("activity",     "M22 12h-4l-3 9L9 3l-3 9H2"),
    ("dashboard",    "M3 3h8v10H3zM13 3h8v6h-8zM13 13h8v8h-8zM3 17h8v4H3z"),
    ("code",         "M16 18l6-6-6-6M8 6l-6 6 6 6"),
    ("copy",         "M9 9h11v11H9zM5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1"),
    ("refresh",      "M21 2v6h-6M3 12a9 9 0 0115-6.7L21 8M3 22v-6h6M21 12a9 9 0 01-15 6.7L3 16"),
    ("archive",      "M3 4h18v4H3zM5 8v12h14V8M9 12h6"),
    ("sparkle",      "M12 3l1.9 5.1L19 10l-5.1 1.9L12 17l-1.9-5.1L5 10l5.1-1.9zM19 3v4M21 5h-4"),
    ("arrowLeftRight", "M8 3L4 7l4 4M4 7h16M16 21l4-4-4-4M20 17H4"),
    ("eye",          "M2 12s4-7 10-7 10 7 10 7-4 7-10 7-10-7-10-7zM12 12m-3 0a3 3 0 106 0a3 3 0 10-6 0"),
];

/// Render an SVG icon as raw HTML string.
pub fn icon_html(name: &str, size: u32) -> String {
    let paths = ICONS.iter().find(|(n, _)| *n == name).map(|(_, p)| *p).unwrap_or("");
    let path_els: String = paths
        .split('M')
        .filter(|s| !s.trim().is_empty())
        .map(|s| format!(r#"<path d="M{}"/>"#, s.trim()))
        .collect();
    format!(
        r#"<svg width="{s}" height="{s}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" style="display:block;flex-shrink:0">{}</svg>"#,
        path_els,
        s = size
    )
}
