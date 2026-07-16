pub mod app;
pub mod audio;
pub mod color;
pub mod capture;
pub mod db;
pub mod external_tools;
pub mod i18n;
pub mod navigation;
pub mod paths;
pub mod pipeline;
pub mod script;
pub mod clip_preview;
pub mod icons;
pub mod slot_image;
pub mod alarm;
pub mod shortcuts;

slint::include_modules!();

pub use app::run;

/// Affiche une erreur fatale au démarrage (release Windows sans console).
pub fn show_fatal_error(message: &str) {
    #[cfg(windows)]
    {
        use std::ffi::c_void;
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONERROR, MB_OK};

        fn wide(s: &str) -> Vec<u16> {
            std::ffi::OsStr::new(s)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect()
        }
        let text = wide(message);
        let title = wide("Streamdeck");
        unsafe {
            MessageBoxW(
                std::ptr::null_mut::<c_void>(),
                text.as_ptr(),
                title.as_ptr(),
                MB_OK | MB_ICONERROR,
            );
        }
    }
    #[cfg(not(windows))]
    {
        eprintln!("Erreur: {message}");
    }
}
