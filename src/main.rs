//! Binaire GUI : pas de fenêtre console en release (Windows).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(e) = streamdeck::run() {
        #[cfg(not(debug_assertions))]
        streamdeck::show_fatal_error(&format!("{e:#}"));
        #[cfg(debug_assertions)]
        eprintln!("Erreur: {e:#}");
        std::process::exit(1);
    }
}
