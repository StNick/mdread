//! Locate good system fonts and register them with egui.
//!
//! Preference order (body): Ubuntu Sans (variable, so bold is a real weight axis),
//! Ubuntu static, Noto Sans, DejaVu Sans, then egui's bundled fonts.
//! Preference order (code): JetBrains Mono, Ubuntu Sans Mono, DejaVu Sans Mono, egui's Hack.
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

const SANS: &[Candidate] = &[
    Candidate {
        name: "Ubuntu Sans",
        regular: "/usr/share/fonts/truetype/ubuntu/UbuntuSans[wdth,wght].ttf",
        italic: Some("/usr/share/fonts/truetype/ubuntu/UbuntuSans-Italic[wdth,wght].ttf"),
        bold: None,
        bold_italic: None,
        variable: true,
    },
    Candidate {
        name: "Ubuntu",
        regular: "/usr/share/fonts/truetype/ubuntu/Ubuntu-R.ttf",
        italic: Some("/usr/share/fonts/truetype/ubuntu/Ubuntu-RI.ttf"),
        bold: Some("/usr/share/fonts/truetype/ubuntu/Ubuntu-B.ttf"),
        bold_italic: Some("/usr/share/fonts/truetype/ubuntu/Ubuntu-BI.ttf"),
        variable: false,
    },
    Candidate {
        name: "Noto Sans",
        regular: "/usr/share/fonts/truetype/noto/NotoSans-Regular.ttf",
        italic: Some("/usr/share/fonts/truetype/noto/NotoSans-Italic.ttf"),
        bold: Some("/usr/share/fonts/truetype/noto/NotoSans-Bold.ttf"),
        bold_italic: Some("/usr/share/fonts/truetype/noto/NotoSans-BoldItalic.ttf"),
        variable: false,
    },
    Candidate {
        name: "DejaVu Sans",
        regular: "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
        italic: Some("/usr/share/fonts/truetype/dejavu/DejaVuSans-Oblique.ttf"),
        bold: Some("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf"),
        bold_italic: Some("/usr/share/fonts/truetype/dejavu/DejaVuSans-BoldOblique.ttf"),
        variable: false,
    },
];

const MONO: &[(&str, &str)] = &[
    ("JetBrains Mono", "~/.local/share/fonts/ttf/JetBrainsMono-Regular.ttf"),
    ("JetBrains Mono", "~/.local/share/fonts/JetBrainsMono-Regular.ttf"),
    ("JetBrains Mono", "/usr/share/fonts/truetype/jetbrains-mono/JetBrainsMono-Regular.ttf"),
    ("JetBrains Mono", "/usr/share/fonts/truetype/JetBrainsMono-Regular.ttf"),
    ("Ubuntu Sans Mono", "/usr/share/fonts/truetype/ubuntu/UbuntuSansMono[wght].ttf"),
    ("Ubuntu Mono", "/usr/share/fonts/truetype/ubuntu/UbuntuMono-R.ttf"),
    ("Fira Code", "/usr/share/fonts/truetype/firacode/FiraCode-Regular.ttf"),
    ("DejaVu Sans Mono", "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf"),
];

fn expand(p: &str) -> PathBuf {
    if let Some(rest) = p.strip_prefix("~/") {
        if let Some(home) = std::env::var_os("HOME") {
            return Path::new(&home).join(rest);
        }
    }
    PathBuf::from(p)
}

fn read(p: &str) -> Option<Vec<u8>> {
    std::fs::read(expand(p)).ok()
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
    if let Some(bytes) = mono_bytes {
        defs.font_data
            .insert("mdread-mono".to_owned(), Arc::new(FontData::from_owned(bytes)));
        let mono = defs.families.entry(FontFamily::Monospace).or_default();
        mono.insert(0, "mdread-mono".to_owned());
    }

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
