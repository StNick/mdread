//! mdread — a very fast GPU-rendered Markdown viewer.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod document;
mod files;
mod fonts;
mod highlight;
mod render;
mod search;
mod theme;

use std::path::PathBuf;

use app::{Args, ThemeChoice};

const HELP: &str = "\
mdread — fast Markdown viewer

USAGE:
    mdread [OPTIONS] [FILE[#anchor]]
    cat FILE.md | mdread -

OPTIONS:
    --dark, --light      Force a theme for this session (default: follow the system)
    --find TERM          Open with the search box filled in
    --screenshot PATH    Save a PNG of the window after 1.5 s and exit
    --screenshot-delay N Delay in ms for --screenshot
    -h, --help           Show this help
    -V, --version        Show version

ENVIRONMENT:
    MDREAD_FONT          Path to a .ttf/.otf to use for body text
    MDREAD_MONO_FONT     Path to a .ttf/.otf to use for code
";

fn parse_args() -> Args {
    let mut args = Args { screenshot_delay_ms: 1500, ..Args::default() };
    let mut it = std::env::args_os().skip(1);
    while let Some(a) = it.next() {
        let s = a.to_string_lossy();
        match s.as_ref() {
            "--screenshot" => args.screenshot = it.next().map(PathBuf::from),
            "--find" => args.find = it.next().map(|v| v.to_string_lossy().to_string()),
            "--screenshot-delay" => {
                args.screenshot_delay_ms = it.next().and_then(|v| v.to_string_lossy().parse().ok()).unwrap_or(1500);
            }
            "-h" | "--help" => {
                print!("{HELP}");
                std::process::exit(0);
            }
            "-V" | "--version" => {
                println!("mdread {}", env!("CARGO_PKG_VERSION"));
                std::process::exit(0);
            }
            "--dark" => args.theme = Some(ThemeChoice::Dark),
            "--light" => args.theme = Some(ThemeChoice::Light),
            "--system" => args.theme = Some(ThemeChoice::System),
            "-" => args.stdin = true,
            _ if s.starts_with("--") => {
                eprintln!("unknown option {s}\n{HELP}");
                std::process::exit(2);
            }
            _ => {
                // Allow `file.md#section` to open straight at a heading.
                if let Some((file, frag)) = s.split_once('#') {
                    if !std::path::Path::new(s.as_ref()).exists() && std::path::Path::new(file).exists() {
                        args.files.push(PathBuf::from(file));
                        args.anchor = Some(frag.to_owned());
                        continue;
                    }
                }
                args.files.push(PathBuf::from(a));
            }
        }
    }
    args
}

/// Window icon, used for `_NET_WM_ICON` on X11, the Dock on macOS and the task bar on
/// Windows. Wayland compositors ignore it and take the icon from the desktop entry that
/// matches the app id instead, which `install.sh` installs together with the icon files.
fn app_icon() -> egui::IconData {
    eframe::icon_data::from_png_bytes(include_bytes!("../assets/icons/hicolor/256x256/apps/mdread.png"))
        .expect("embedded icon PNG is valid")
}

fn main() -> eframe::Result {
    let launched = std::time::Instant::now();
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();
    let mut args = parse_args();
    args.launched = Some(launched);

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("mdread")
            .with_app_id("mdread")
            .with_icon(app_icon())
            .with_inner_size([1500.0, 1250.0])
            .with_min_inner_size([420.0, 300.0]),
        persist_window: true,
        ..Default::default()
    };
    eframe::run_native("mdread", native_options, Box::new(move |cc| Ok(Box::new(app::App::new(cc, args)))))
}
