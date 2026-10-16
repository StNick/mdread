//! Locate good system fonts and register them with egui.
//!
//! Preference order (body): Ubuntu Sans (variable, so bold is a real weight axis),
//! Ubuntu static, Noto Sans, DejaVu Sans, SF Pro (macOS, variable), Arial, then egui's
//! bundled fonts.
//! Preference order (code): JetBrains Mono, any installed Nerd Font, Ubuntu Sans Mono,
//! Ubuntu Mono, Fira Code, DejaVu Sans Mono, SF Mono / Menlo-era Monaco (macOS), egui's Hack.
//! Files are looked up by bare name in `FONT_DIRS`, so a user-installed copy in
//! `~/.local/share/fonts` or `~/Library/Fonts` wins over a distro one.
//! Override with MDREAD_FONT / MDREAD_MONO_FONT (paths to .ttf/.otf files).

use std::path::{Path, PathBuf};
use std::sync::Arc;

use egui::{Context, FontData, FontDefinitions, FontFamily};

#[derive(Clone, Debug)]
pub struct FontSet {
    /// Regular body face (also installed as `FontFamily::Proportional`).
    pub sans: FontFamily,
    /// True italic face, or `sans` again if none was found (egui then synthesises a slant).
    pub sans_italic: FontFamily,
    /// Static bold face when the body font is not variable.
    pub sans_bold: Option<FontFamily>,
    pub sans_bold_italic: Option<FontFamily>,
    /// Code face (also installed as `FontFamily::Monospace`).
    pub mono: FontFamily,
    /// True when `sans` is a variable font with a `wght` axis, so bold is done with coords.
    pub variable_weight: bool,
    pub sans_name: String,
    pub mono_name: String,
}

struct Candidate {
    name: &'static str,
    regular: &'static str,
    italic: Option<&'static str>,
    bold: Option<&'static str>,
    bold_italic: Option<&'static str>,
    variable: bool,
}

/// Directories searched, in order, for the bare file names in `SANS` and `MONO`.
/// User font directories come first so a locally installed font beats the system copy.
#[cfg(target_os = "macos")]
const FONT_DIRS: &[&str] = &[
    "~/Library/Fonts",
    "/Library/Fonts",
    "/System/Library/Fonts",
    "/System/Library/Fonts/Supplemental",
];
#[cfg(not(target_os = "macos"))]
const FONT_DIRS: &[&str] = &[
    "~/.local/share/fonts",
    "~/.local/share/fonts/ttf",
    "~/.fonts",
    "/usr/share/fonts/truetype/ubuntu",
    "/usr/share/fonts/truetype/noto",
    "/usr/share/fonts/truetype/dejavu",
    "/usr/share/fonts/truetype/jetbrains-mono",
    "/usr/share/fonts/truetype/firacode",
    "/usr/share/fonts/truetype",
    "/usr/share/fonts",
];

const SANS: &[Candidate] = &[
    Candidate {
        name: "Ubuntu Sans",
        regular: "UbuntuSans[wdth,wght].ttf",
        italic: Some("UbuntuSans-Italic[wdth,wght].ttf"),
        bold: None,
        bold_italic: None,
        variable: true,
    },
    Candidate {
        name: "Ubuntu",
        regular: "Ubuntu-R.ttf",
        italic: Some("Ubuntu-RI.ttf"),
        bold: Some("Ubuntu-B.ttf"),
        bold_italic: Some("Ubuntu-BI.ttf"),
        variable: false,
    },
    Candidate {
        name: "Noto Sans",
        regular: "NotoSans-Regular.ttf",
        italic: Some("NotoSans-Italic.ttf"),
        bold: Some("NotoSans-Bold.ttf"),
        bold_italic: Some("NotoSans-BoldItalic.ttf"),
        variable: false,
    },
    Candidate {
        name: "DejaVu Sans",
        regular: "DejaVuSans.ttf",
        italic: Some("DejaVuSans-Oblique.ttf"),
        bold: Some("DejaVuSans-Bold.ttf"),
        bold_italic: Some("DejaVuSans-BoldOblique.ttf"),
        variable: false,
    },
    // macOS system UI font: a variable TrueType with a `wght` axis, like Ubuntu Sans.
    Candidate {
        name: "SF Pro",
        regular: "SFNS.ttf",
        italic: Some("SFNSItalic.ttf"),
        bold: None,
        bold_italic: None,
        variable: true,
    },
    // Ships with macOS in /System/Library/Fonts/Supplemental.
    Candidate {
        name: "Arial",
        regular: "Arial.ttf",
        italic: Some("Arial Italic.ttf"),
        bold: Some("Arial Bold.ttf"),
        bold_italic: Some("Arial Bold Italic.ttf"),
        variable: false,
    },
];

const MONO: &[(&str, &str)] = &[("JetBrains Mono", "JetBrainsMono-Regular.ttf")];

/// Any Nerd Font patched monospace face found in `FONT_DIRS`, tried after `MONO` and
/// before the distro fallbacks in `MONO_FALLBACK`. Nerd Fonts v3 names files
/// `<Family>NerdFontMono-Regular.ttf` (icons fitted to one cell) and
/// `<Family>NerdFont-Regular.ttf`; the Mono variant is preferred since it never
/// disturbs code alignment.
const NERD_FONT_SUFFIXES: &[&str] = &["NerdFontMono-Regular.ttf", "NerdFont-Regular.ttf"];

const MONO_FALLBACK: &[(&str, &str)] = &[
    ("Ubuntu Sans Mono", "UbuntuSansMono[wght].ttf"),
    ("Ubuntu Mono", "UbuntuMono-R.ttf"),
    ("Fira Code", "FiraCode-Regular.ttf"),
    ("DejaVu Sans Mono", "DejaVuSansMono.ttf"),
    ("SF Mono", "SFNSMono.ttf"),
    ("Monaco", "Monaco.ttf"),
];

fn expand(p: &str) -> PathBuf {
    if let Some(rest) = p.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return Path::new(&home).join(rest);
        }
    }
    PathBuf::from(p)
}

/// Read a font file. Absolute (or `~/`) paths are read as given; a bare file name is
/// looked up in each of `FONT_DIRS` in turn.
fn read(p: &str) -> Option<Vec<u8>> {
    if p.starts_with('/') || p.starts_with("~/") {
        return std::fs::read(expand(p)).ok();
    }
    FONT_DIRS.iter().find_map(|d| std::fs::read(expand(d).join(p)).ok())
}

/// First Nerd Font monospace face found, as (display name, bytes). Only the top level of
/// each font directory is listed (a handful of entries), so this costs well under a
/// millisecond and never walks the whole of /usr/share/fonts.
fn find_nerd_font() -> Option<(String, Vec<u8>)> {
    for suffix in NERD_FONT_SUFFIXES {
        for dir in FONT_DIRS {
            let Ok(entries) = std::fs::read_dir(expand(dir)) else { continue };
            let mut names: Vec<String> = entries
                .filter_map(|e| e.ok())
                .filter_map(|e| e.file_name().into_string().ok())
                .filter(|n| n.ends_with(suffix))
                .collect();
            names.sort();
            for file in names {
                if let Ok(bytes) = std::fs::read(expand(dir).join(&file)) {
                    let family = file.trim_end_matches(suffix);
                    let kind = if suffix.starts_with("NerdFontMono") { "Nerd Font Mono" } else { "Nerd Font" };
                    return Some((format!("{family} {kind}"), bytes));
                }
            }
        }
    }
    None
}

/// Emoji / symbol fallbacks that ship inside egui and should end every family list.
const TAIL: &[&str] = &["Ubuntu-Light", "NotoEmoji-Regular", "emoji-icon-font"];

fn family(defs: &mut FontDefinitions, name: &str, primary: Vec<String>) -> FontFamily {
    let mut list = primary;
    for t in TAIL {
        if defs.font_data.contains_key(*t) && !list.iter().any(|n| n == t) {
            list.push((*t).to_owned());
        }
    }
    let fam = FontFamily::Name(Arc::from(name));
    defs.families.insert(fam.clone(), list);
    fam
}

pub fn install(ctx: &Context) -> FontSet {
    let mut defs = FontDefinitions::default();

    // ---- body font -------------------------------------------------------
    let mut sans_name = String::from("egui default");
    let mut regular: Option<Vec<u8>> = None;
    let mut italic: Option<Vec<u8>> = None;
    let mut bold: Option<Vec<u8>> = None;
    let mut bold_italic: Option<Vec<u8>> = None;
    let mut variable = false;

    if let Some(p) = std::env::var_os("MDREAD_FONT") {
        let p = p.to_string_lossy().to_string();
        if let Some(bytes) = read(&p) {
            variable = p.contains("[") || p.to_lowercase().contains("wght");
            sans_name = Path::new(&p)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or(p.clone());
            regular = Some(bytes);
        }
    }
    if regular.is_none() {
        for cand in SANS {
            if let Some(bytes) = read(cand.regular) {
                regular = Some(bytes);
                italic = cand.italic.and_then(read);
                bold = cand.bold.and_then(read);
                bold_italic = cand.bold_italic.and_then(read);
                variable = cand.variable;
                sans_name = cand.name.to_owned();
                break;
            }
        }
    }

    let mut sans_bold = None;
    let mut sans_bold_italic = None;
    let sans;
    let sans_italic;
    if let Some(bytes) = regular {
        defs.font_data
            .insert("mdread-sans".to_owned(), Arc::new(FontData::from_owned(bytes)));
        // Put our face first for the standard proportional family so all UI chrome uses it.
        let prop = defs.families.entry(FontFamily::Proportional).or_default();
        prop.insert(0, "mdread-sans".to_owned());
        sans = FontFamily::Proportional;

        if let Some(bytes) = italic {
            defs.font_data
                .insert("mdread-sans-italic".to_owned(), Arc::new(FontData::from_owned(bytes)));
            sans_italic = family(
                &mut defs,
                "mdread-sans-italic",
                vec!["mdread-sans-italic".to_owned(), "mdread-sans".to_owned()],
            );
        } else {
            sans_italic = FontFamily::Proportional;
        }
        if !variable {
            if let Some(bytes) = bold {
                defs.font_data
                    .insert("mdread-sans-bold".to_owned(), Arc::new(FontData::from_owned(bytes)));
                sans_bold = Some(family(
                    &mut defs,
                    "mdread-sans-bold",
                    vec!["mdread-sans-bold".to_owned(), "mdread-sans".to_owned()],
                ));
            }
            if let Some(bytes) = bold_italic {
                defs.font_data.insert(
                    "mdread-sans-bold-italic".to_owned(),
                    Arc::new(FontData::from_owned(bytes)),
                );
                sans_bold_italic = Some(family(
                    &mut defs,
                    "mdread-sans-bold-italic",
                    vec![
                        "mdread-sans-bold-italic".to_owned(),
                        "mdread-sans-bold".to_owned(),
                        "mdread-sans".to_owned(),
                    ],
                ));
            }
        }
    } else {
        sans = FontFamily::Proportional;
        sans_italic = FontFamily::Proportional;
    }

    // ---- code font -------------------------------------------------------
    let mut mono_name = String::from("Hack");
    let mut mono_bytes: Option<Vec<u8>> = None;
    if let Some(p) = std::env::var_os("MDREAD_MONO_FONT") {
        let p = p.to_string_lossy().to_string();
        if let Some(bytes) = read(&p) {
            mono_name = Path::new(&p)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or(p.clone());
            mono_bytes = Some(bytes);
        }
    }
    if mono_bytes.is_none() {
        for (name, path) in MONO {
            if let Some(bytes) = read(path) {
                mono_name = (*name).to_owned();
                mono_bytes = Some(bytes);
                break;
            }
        }
    }
    if mono_bytes.is_none() {
        if let Some((name, bytes)) = find_nerd_font() {
            mono_name = name;
            mono_bytes = Some(bytes);
        }
    }
    if mono_bytes.is_none() {
        for (name, path) in MONO_FALLBACK {
            if let Some(bytes) = read(path) {
                mono_name = (*name).to_owned();
                mono_bytes = Some(bytes);
                break;
            }
        }
    }
    if let Some(bytes) = mono_bytes {
        defs.font_data
            .insert("mdread-mono".to_owned(), Arc::new(FontData::from_owned(bytes)));
        let mono = defs.families.entry(FontFamily::Monospace).or_default();
        mono.insert(0, "mdread-mono".to_owned());
    }

    log::debug!("fonts: body {sans_name} (variable weight: {variable}), code {mono_name}");
    ctx.set_fonts(defs);

    FontSet {
        sans,
        sans_italic,
        sans_bold,
        sans_bold_italic,
        mono: FontFamily::Monospace,
        variable_weight: variable,
        sans_name,
        mono_name,
    }
}
