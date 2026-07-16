mod grid_model;
mod grid_shortcuts;

use crate::clip_preview::{
    analyze_clip_source, format_clip_time, range_to_seconds, ClipSourceInfo,
};
use crate::pipeline::UrlProbeInfo;
use crate::alarm::{
    alarm_time_from_indices, indices_from_alarm_time, is_timer_mode, now_ms, AlarmFireTracker,
};
use crate::capture::{capture_source_label, CaptureSource, RecordingSession};
use crate::color::{brush_from_rgb, hex_to_rgb, parse_hex, rgb_to_hex};
use crate::icons::{is_catalog_icon_id, load_icon};
use crate::slot_image::{
    delete_slot_image_file, image_library_label, import_slot_image_from_file,
    import_slot_image_from_url, is_image_appearance, list_slot_images, load_slot_photo,
};
use crate::audio::SharedAudioEngine;
use crate::db::{init_database, DbRepository, Slot, SlotKind};
use crate::external_tools::tool_status_message;
use crate::navigation::{is_back_slot, is_home_slot, Navigation};
use crate::paths::AppPaths;
use crate::pipeline::{
    download_url_audio_full, probe_url_info, SoundPipeline, URL_DEFAULT_MAX_DURATION_SECS,
};
use anyhow::{Context, Result};
use grid_model::{
    build_grid_slots, default_color_for_kind, hex_for_slot_editor, DEFAULT_SLOT_COLOR,
    DEFAULT_SLOT_ICON, NEUTRAL_SLOT_COLOR,
};
use grid_shortcuts::wire_grid_shortcuts;
use slint::winit_030::WinitWindowAccessor;
use slint::{ComponentHandle, ModelRc, SharedString, Timer, TimerMode, VecModel};
use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::AppWindow;

/// Fenêtre principale compacte (grille + barre + volume).
const COMPACT_WINDOW_W: u32 = 520;
const COMPACT_WINDOW_H: u32 = 362;
const SETTINGS_MODAL_W: u32 = 340;
const SETTINGS_MODAL_H: u32 = 300;
const APP_SETTINGS_MODAL_W: u32 = 380;
const APP_SETTINGS_MODAL_H: u32 = 730;
const SLOT_EDITOR_MODAL_W: u32 = 480;
const SLOT_EDITOR_MODAL_H_MIN: u32 = 300;
const SLOT_EDITOR_MODAL_H_MAX: u32 = 1020;
const TOOLS_MODAL_W: u32 = 360;
const TOOLS_MODAL_H: u32 = 280;
const LIBRARY_MODAL_W: u32 = 420;
const LIBRARY_MODAL_H: u32 = 520;
const IMAGE_LIBRARY_MODAL_W: u32 = 460;
const IMAGE_LIBRARY_MODAL_H: u32 = 600;
const CAPTURE_MODAL_W: u32 = 440;
const CAPTURE_MODAL_H: u32 = 480;
const CLIP_MODAL_W: u32 = 560;
const CLIP_MODAL_H: u32 = 580;
const URL_MODAL_W: u32 = 560;
const URL_MODAL_H: u32 = 580;

#[derive(Copy, Clone, Eq, PartialEq)]
enum UtilityModal {
    Settings,
    AppSettings,
    Tools,
    Library,
    ImageLibrary,
    Capture,
    ClipEditor,
    UrlClip,
}

pub struct Application {
    ui: AppWindow,
    paths: AppPaths,
    repo: DbRepository,
    nav: RefCell<Navigation>,
    root_id: i64,
    audio: SharedAudioEngine,
    pipeline: SoundPipeline,
    edit_mode: RefCell<bool>,
    editing_slot: RefCell<Option<(i32, i32)>>,
    recording: RefCell<Option<RecordingSession>>,
    last_capture_wav: RefCell<Option<PathBuf>>,
    capture_poll: RefCell<Option<Timer>>,
    /// Taille fenêtre avant ouverture d’un modal (réglages, éditeur de touche, etc.).
    window_size_before_overlay: RefCell<Option<(u32, u32)>>,
    alarm_tracker: RefCell<AlarmFireTracker>,
    alarm_poll: RefCell<Option<Timer>>,
    /// Fichier image importé (assets/images/) en cours d’édition, pas encore sauvegardé sur la touche.
    slot_editor_custom_image: RefCell<Option<String>>,
    /// Noms de fichiers alignés sur `image-library-titles` dans l’UI.
    image_library_files: RefCell<Vec<String>>,
    image_library_pick_mode: RefCell<bool>,
    library_pick_mode: RefCell<bool>,
    slot_editor_selected_sound_id: RefCell<Option<i64>>,
    modal_back_stack: RefCell<Vec<UtilityModal>>,
    /// Fichier audio temporaire chargé depuis une URL (aperçu / découpe).
    url_preview_wav: Arc<Mutex<Option<PathBuf>>>,
    grid_shortcuts_enabled: RefCell<bool>,
    shortcut_capture_active: RefCell<bool>,
    editor_shortcut_override: RefCell<Option<String>>,
    editor_shortcut_use_default: RefCell<bool>,
    modifiers_down: RefCell<winit::keyboard::ModifiersState>,
}

pub fn run() -> Result<()> {
    init_logging();
    let paths = AppPaths::discover()?;
    let repo = init_database(&paths)?;
    let root_id = repo.root_page_id()?;
    let nav = RefCell::new(Navigation::new(root_id));

    let policy = repo.audio_policy()?;
    let max_ch: u32 = repo
        .get_setting("max_channels")?
        .and_then(|v| v.parse().ok())
        .unwrap_or(3);
    let global_vol = repo.global_volume()?;
    let lufs: f32 = repo
        .get_setting("normalize_target_lufs")?
        .and_then(|v| v.parse().ok())
        .unwrap_or(-16.0);

    let audio = Arc::new(Mutex::new(crate::audio::AudioEngine::new(
        paths.clone(),
        policy,
        max_ch,
        global_vol,
    )?));
    let pipeline = SoundPipeline::new(paths.clone(), lufs);

    let ui = AppWindow::new().context("create UI")?;
    let lang = crate::i18n::Lang::from_setting(
        &repo
            .get_setting("ui_language")?
            .unwrap_or_else(|| "en".into()),
    );
    crate::i18n::apply_slint(lang);
    ui.set_settings_language(SharedString::from(lang.as_setting()));
    ui.set_global_volume(global_vol);
    let _ = tool_status_message(&paths.base);
    ui.set_capture_loopback_available(CaptureSource::loopback_available());

    let app = Rc::new(Application {
        ui,
        paths,
        repo,
        nav,
        root_id,
        audio,
        pipeline,
        edit_mode: RefCell::new(false),
        editing_slot: RefCell::new(None),
        recording: RefCell::new(None),
        last_capture_wav: RefCell::new(None),
        capture_poll: RefCell::new(None),
        window_size_before_overlay: RefCell::new(None),
        alarm_tracker: RefCell::new(AlarmFireTracker::new()),
        alarm_poll: RefCell::new(None),
        slot_editor_custom_image: RefCell::new(None),
        image_library_files: RefCell::new(Vec::new()),
        image_library_pick_mode: RefCell::new(false),
        library_pick_mode: RefCell::new(false),
        slot_editor_selected_sound_id: RefCell::new(None),
        modal_back_stack: RefCell::new(Vec::new()),
        url_preview_wav: Arc::new(Mutex::new(None)),
        grid_shortcuts_enabled: RefCell::new(true),
        shortcut_capture_active: RefCell::new(false),
        editor_shortcut_override: RefCell::new(None),
        editor_shortcut_use_default: RefCell::new(true),
        modifiers_down: RefCell::new(winit::keyboard::ModifiersState::empty()),
    });

    app.load_grid_shortcuts_setting();
    wire_callbacks(app.clone());
    wire_window_chrome(app.clone());
    wire_grid_shortcuts(app.clone());
    app.init_slot_editor_lists();
    app.refresh_alarm_presets_ui()?;
    app.start_alarm_poll();
    app.refresh_grid()?;
    app.refresh_library()?;
    app.restore_window_size()?;
    app.apply_always_on_top_setting();
    app.apply_native_window_chrome();
    #[cfg(windows)]
    apply_windows_rounded_corners(&app.ui.window());

    app.ui.run().context("run UI")?;
    app.save_window_size()?;
    Ok(())
}

fn init_logging() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("streamdeck=info")
        .try_init();
}

fn wire_callbacks(app: Rc<Application>) {
    let a = app.clone();
    app.ui.on_slot_clicked(move |row, col| {
        let _ = a.handle_slot_click(row, col);
    });

    let a = app.clone();
    app.ui.on_volume_changed(move |v| {
        let _ = a.handle_volume(v);
    });

    let a = app.clone();
    app.ui.on_stop_audio(move || a.audio.lock().unwrap().stop_all());

    let a = app.clone();
    app.ui.on_toggle_edit(move || a.toggle_edit());

    let a = app.clone();
    app.ui.on_refresh_grid(move || {
        let _ = a.refresh_grid();
    });

    let a = app.clone();
    app.ui.on_open_library(move || {
        *a.library_pick_mode.borrow_mut() = false;
        a.ui.set_library_pick_mode(false);
        a.open_utility_modal(UtilityModal::Library);
        let _ = a.refresh_library();
    });
    let a = app.clone();
    app.ui.on_close_library(move || {
        if *a.library_pick_mode.borrow() {
            a.ui.set_show_library(false);
            *a.library_pick_mode.borrow_mut() = false;
            a.ui.set_library_pick_mode(false);
            a.ui.set_show_slot_editor(true);
        } else {
            a.close_utility_modal(UtilityModal::Library);
        }
    });
    let a = app.clone();
    app.ui.on_library_choose(move || {
        if let Err(e) = a.library_choose() {
            a.set_status(SharedString::from(format!("Bibliothèque : {e:#}")));
        }
    });
    let a = app.clone();
    app.ui.on_slot_open_sound_library(move || a.open_sound_library_picker());

    let a = app.clone();
    app.ui.on_open_image_library(move || {
        *a.image_library_pick_mode.borrow_mut() = false;
        a.ui.set_image_library_pick_mode(false);
        a.open_utility_modal(UtilityModal::ImageLibrary);
        a.open_image_library();
    });
    let a = app.clone();
    app.ui.on_close_image_library(move || {
        a.close_utility_modal(UtilityModal::ImageLibrary);
        *a.image_library_pick_mode.borrow_mut() = false;
        a.ui.set_image_library_pick_mode(false);
    });
    let a = app.clone();
    app.ui.on_image_library_add_file(move || {
        if let Err(e) = a.image_library_add_file() {
            a.set_status(SharedString::from(format!("Image : {e:#}")));
        }
    });
    let a = app.clone();
    app.ui.on_image_library_add_url(move |url| {
        if let Err(e) = a.image_library_add_url(url.as_str()) {
            a.set_status(SharedString::from(format!("Image : {e:#}")));
        }
    });
    let a = app.clone();
    app.ui.on_image_library_delete(move || {
        let _ = a.image_library_delete();
    });
    let a = app.clone();
    app.ui.on_image_library_preview(move || {
        let _ = a.image_library_preview_thumb();
    });
    let a = app.clone();
    app.ui.on_image_library_choose(move || {
        let _ = a.image_library_choose();
    });
    let a = app.clone();
    app.ui.on_slot_open_image_library(move || {
        *a.image_library_pick_mode.borrow_mut() = true;
        a.ui.set_image_library_pick_mode(true);
        a.open_image_library();
    });

    let a = app.clone();
    app.ui.on_open_settings(move || a.load_settings_ui());
    let a = app.clone();
    app.ui
        .on_close_settings(move || a.close_utility_modal(UtilityModal::Settings));
    let a = app.clone();
    app.ui.on_open_app_settings(move || a.open_app_settings_ui());
    let a = app.clone();
    app.ui
        .on_close_app_settings(move || a.close_utility_modal(UtilityModal::AppSettings));
    let a = app.clone();
    app.ui.on_settings_save(move || {
        let _ = a.save_settings();
    });

    let a = app.clone();
    app.ui.on_open_capture(move || {
        a.open_utility_modal(UtilityModal::Capture);
        a.reset_capture_ui();
        a.ui.set_capture_title("".into());
        let _ = a.sync_capture_source_ui();
    });
    let a = app.clone();
    app.ui.on_capture_set_source(move |src| {
        a.set_capture_source(&src);
    });
    let a = app.clone();
    app.ui.on_close_capture(move || {
        a.cancel_capture();
        a.close_utility_modal(UtilityModal::Capture);
    });
    let a = app.clone();
    app.ui.on_capture_record_toggle(move || {
        if let Err(e) = Application::capture_toggle(&a) {
            a.set_status(SharedString::from(format!("Micro : {e:#}")));
        }
    });
    let a = app.clone();
    app.ui.on_capture_cancel(move || a.cancel_capture());
    let a = app.clone();
    app.ui.on_capture_save(move || {
        if let Err(e) = a.capture_save() {
            a.set_status(SharedString::from(format!("Sauvegarde : {e:#}")));
        }
    });
    let a = app.clone();
    app.ui.on_capture_refine(move || {
        let _ = a.capture_refine();
    });

    let a = app.clone();
    app.ui.on_open_clip_editor(move || {
        a.prepare_clip_editor();
        a.open_utility_modal(UtilityModal::ClipEditor);
    });
    let a = app.clone();
    app.ui
        .on_close_clip_editor(move || a.close_utility_modal(UtilityModal::ClipEditor));
    let a = app.clone();
    app.ui.on_clip_browse(move || {
        if let Err(e) = a.clip_browse() {
            a.ui.set_clip_status_msg(format!("{e:#}").into());
        }
    });
    let a = app.clone();
    app.ui.on_clip_range_start_changed(move |r| {
        a.ui.set_clip_range_start(r);
        a.sync_clip_range_labels();
    });
    let a = app.clone();
    app.ui.on_clip_range_end_changed(move |r| {
        a.ui.set_clip_range_end(r);
        a.sync_clip_range_labels();
    });
    let a = app.clone();
    app.ui.on_clip_preview_selection(move || {
        if let Err(e) = a.clip_preview_selection() {
            a.set_status(SharedString::from(format!("Clip : {e:#}")));
        }
    });
    let a = app.clone();
    app.ui.on_clip_export(move || {
        if let Err(e) = a.clip_export() {
            a.set_status(SharedString::from(format!("Clip : {e:#}")));
        }
    });

    let a = app.clone();
    app.ui.on_open_url_clip(move || {
        a.prepare_url_clip();
        a.open_utility_modal(UtilityModal::UrlClip);
    });
    let a = app.clone();
    app.ui.on_close_url_clip(move || {
        a.cleanup_url_preview();
        a.close_utility_modal(UtilityModal::UrlClip);
    });
    let a = app.clone();
    app.ui.on_url_load(move || {
        a.url_load();
    });
    let a = app.clone();
    app.ui.on_url_range_start_changed(move |r| {
        a.ui.set_url_range_start(r);
        a.sync_url_range_labels();
    });
    let a = app.clone();
    app.ui.on_url_range_end_changed(move |r| {
        a.ui.set_url_range_end(r);
        a.sync_url_range_labels();
    });
    let a = app.clone();
    app.ui.on_url_preview_selection(move || {
        if let Err(e) = a.url_preview_selection() {
            a.set_status(SharedString::from(format!("URL : {e:#}")));
        }
    });
    let a = app.clone();
    app.ui.on_url_extract(move || {
        if let Err(e) = a.url_extract() {
            a.set_status(SharedString::from(format!("URL : {e:#}")));
        }
    });

    let a = app.clone();
    app.ui.on_library_add_file(move || {
        if let Err(e) = a.library_add_file() {
            a.set_status(SharedString::from(format!("Import impossible: {e:#}")));
        }
    });
    let a = app.clone();
    app.ui.on_library_delete(move || {
        let _ = a.library_delete();
    });
    let a = app.clone();
    app.ui.on_library_preview(move || {
        let _ = a.library_preview();
    });

    let a = app.clone();
    app.ui.on_slot_editor_save(move || {
        let _ = a.slot_editor_save();
    });
    let a = app.clone();
    app.ui.on_slot_editor_close(move || a.close_slot_editor());
    let a = app.clone();
    app.ui.on_slot_create_folder(move || {
        let _ = a.create_folder_page();
    });
    let a = app.clone();
    app.ui.on_slot_shortcut_capture_start(move || {
        a.start_shortcut_capture();
    });
    let a = app.clone();
    app.ui.on_slot_shortcut_reset_default(move || {
        if let Some((row, col)) = *a.editing_slot.borrow() {
            a.reset_editor_shortcut_to_default(row, col);
        }
    });
    let a = app.clone();
    app.ui
        .on_open_tools_menu(move || a.open_utility_modal(UtilityModal::Tools));
    let a = app.clone();
    app.ui
        .on_close_tools_menu(move || a.close_utility_modal(UtilityModal::Tools));
    let a = app.clone();
    app.ui.on_slot_set_kind_empty(move || {
        a.ui.set_slot_editor_kind("empty".into());
        a.ui.set_slot_editor_appearance("color".into());
        *a.slot_editor_custom_image.borrow_mut() = None;
        a.ui.set_slot_editor_color(NEUTRAL_SLOT_COLOR.into());
        a.sync_slot_editor_color_preview();
        a.ui.set_slot_editor_icon("".into());
        a.sync_slot_editor_icon_preview();
        a.resize_slot_editor_if_open();
    });
    let a = app.clone();
    app.ui.on_slot_set_kind_sound(move || {
        a.ui.set_slot_editor_kind("sound".into());
        a.ui
            .set_slot_editor_color(NEUTRAL_SLOT_COLOR.into());
        a.sync_slot_editor_color_preview();
        let sid = a.editing_slot_sound_id();
        let _ = a.sync_slot_editor_sound(sid);
        a.resize_slot_editor_if_open();
    });
    let a = app.clone();
    app.ui.on_slot_set_kind_folder(move || {
        a.ui.set_slot_editor_kind("folder".into());
        a.ui
            .set_slot_editor_color(default_color_for_kind(SlotKind::Folder).into());
        a.resize_slot_editor_if_open();
    });
    let a = app.clone();
    app.ui.on_slot_set_kind_script(move || {
        a.ui.set_slot_editor_kind("script".into());
        a.ui.set_slot_editor_color(NEUTRAL_SLOT_COLOR.into());
        a.sync_slot_editor_color_preview();
        a.ui
            .set_slot_editor_script_shell(default_script_shell().into());
        a.resize_slot_editor_if_open();
    });
    let a = app.clone();
    app.ui.on_slot_set_script_shell(move |shell| {
        a.ui.set_slot_editor_script_shell(shell);
    });
    let a = app.clone();
    app.ui.on_slot_browse_script(move || {
        if let Err(e) = a.browse_script_file() {
            a.set_status(SharedString::from(format!("Script : {e:#}")));
        }
    });
    let a = app.clone();
    app.ui.on_slot_set_kind_alarm(move || {
        a.ui.set_slot_editor_kind("alarm".into());
        a.ui.set_slot_editor_color(NEUTRAL_SLOT_COLOR.into());
        a.sync_slot_editor_color_preview();
        a.ui.set_slot_editor_icon("cloche".into());
        a.sync_slot_editor_icon_preview();
        a.ui.set_slot_editor_alarm_mode("timer".into());
        let sid = a.editing_slot_sound_id();
        let _ = a.sync_slot_editor_sound(sid);
        a.resize_slot_editor_if_open();
    });
    let a = app.clone();
    app.ui.on_slot_set_alarm_mode_clock(move || {
        a.ui.set_slot_editor_alarm_mode("clock".into());
        a.resize_slot_editor_if_open();
    });
    let a = app.clone();
    app.ui.on_slot_set_alarm_mode_timer(move || {
        a.ui.set_slot_editor_alarm_mode("timer".into());
        a.resize_slot_editor_if_open();
    });
    let a = app.clone();
    app.ui.on_slot_test_alarm(move || {
        let _ = a.test_alarm_from_editor();
    });
    let a = app.clone();
    app.ui.on_alarm_popup_dismiss(move || a.dismiss_alarm_popup());
    let a = app.clone();
    app.ui.on_slot_pick_icon(move |id| {
        a.ui.set_slot_editor_icon(id);
        a.sync_slot_editor_icon_preview();
        a.ui.set_show_slot_icon_picker(false);
    });
    let a = app.clone();
    app.ui.on_slot_pick_color(move |hex| {
        a.ui.set_slot_editor_color(hex);
        a.sync_slot_editor_color_preview();
        a.ui.set_show_slot_color_picker(false);
    });
    let a = app.clone();
    app.ui.on_slot_color_picker_rgb_changed(move || a.sync_slot_color_picker_preview());
    let a = app.clone();
    app.ui.on_slot_color_picker_apply(move || a.apply_slot_color_from_picker());
    let a = app.clone();
    app.ui.on_slot_open_color_picker(move || a.open_slot_color_picker());
    let a = app.clone();
    app.ui.on_slot_close_color_picker(move || a.ui.set_show_slot_color_picker(false));
    let a = app.clone();
    app.ui.on_slot_open_icon_picker(move || a.ui.set_show_slot_icon_picker(true));
    let a = app.clone();
    app.ui.on_slot_close_icon_picker(move || a.ui.set_show_slot_icon_picker(false));
    let a = app.clone();
    app.ui.on_slot_editor_sound_preview(move || {
        let _ = a.preview_editor_sound();
    });

    let a = app.clone();
    app.ui.on_slot_set_appearance_color(move || {
        a.ui.set_slot_editor_appearance("color".into());
        a.sync_slot_editor_icon_preview();
        a.resize_slot_editor_if_open();
    });
    let a = app.clone();
    app.ui.on_slot_set_appearance_image(move || {
        a.ui.set_slot_editor_appearance("image".into());
        a.clear_invalid_editor_image_inheritance();
        a.sync_slot_editor_photo_preview();
        a.resize_slot_editor_if_open();
    });
    let a = app.clone();
    app.ui.on_slot_import_image_file(move || {
        let _ = a.import_slot_image_from_dialog();
    });
    let a = app.clone();
    app.ui.on_slot_import_image_url(move |url| {
        if let Err(e) = a.import_slot_image_from_url_field(url.as_str()) {
            a.set_status(SharedString::from(format!("Image : {e:#}")));
        }
    });
    let a = app.clone();
    app.ui.on_slot_clear_custom_image(move || {
        a.clear_slot_editor_custom_image();
    });
}

fn folder_page_name(label: &str) -> String {
    let t = label.trim();
    if t.is_empty() {
        "dossier".to_string()
    } else {
        t.to_string()
    }
}

fn slugify_page_name(name: &str) -> String {
    let lower = name.to_lowercase();
    let mut out = String::new();
    for c in lower.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c);
        } else if c == ' ' || c == '-' || c == '_' {
            if !out.ends_with('_') && !out.is_empty() {
                out.push('_');
            }
        }
    }
    let trimmed = out.trim_matches('_');
    if trimmed.is_empty() {
        "page".to_string()
    } else {
        trimmed.to_string()
    }
}

fn sync_url_range_labels_ui(ui: &AppWindow) {
    let duration = ui.get_url_duration_secs() as f64;
    let (start, end) = range_to_seconds(
        duration,
        ui.get_url_range_start(),
        ui.get_url_range_end(),
    );
    ui.set_url_start_label(SharedString::from(format_clip_time(start)));
    ui.set_url_end_label(SharedString::from(format_clip_time(end)));
}

fn finish_url_load(
    ui: &AppWindow,
    url_preview: &Arc<Mutex<Option<PathBuf>>>,
    info: UrlProbeInfo,
    path: PathBuf,
    analyzed: ClipSourceInfo,
) {
    if let Ok(mut guard) = url_preview.lock() {
        *guard = Some(path);
    }
    let duration = info.duration_secs.max(0.0);
    let effective_duration = analyzed.duration_secs.max(duration).max(0.1);
    ui.set_url_duration_secs(effective_duration as f32);
    ui.set_url_waveform(ModelRc::new(VecModel::from(analyzed.peaks)));
    ui.set_url_range_start(0.0);
    let end_ratio = (5.0f64 / effective_duration).min(1.0) as f32;
    ui.set_url_range_end(end_ratio.max(0.05));
    sync_url_range_labels_ui(ui);

    if ui.get_url_title().trim().is_empty() {
        if let Some(title) = info.title.filter(|t| !t.trim().is_empty()) {
            ui.set_url_title(title.into());
        }
    }
    ui.set_url_status_msg("".into());
}

impl Application {
    fn utility_modal_size(modal: UtilityModal) -> (u32, u32) {
        match modal {
            UtilityModal::Settings => (SETTINGS_MODAL_W, SETTINGS_MODAL_H),
            UtilityModal::AppSettings => (APP_SETTINGS_MODAL_W, APP_SETTINGS_MODAL_H),
            UtilityModal::Tools => (TOOLS_MODAL_W, TOOLS_MODAL_H),
            UtilityModal::Library => (LIBRARY_MODAL_W, LIBRARY_MODAL_H),
            UtilityModal::ImageLibrary => (IMAGE_LIBRARY_MODAL_W, IMAGE_LIBRARY_MODAL_H),
            UtilityModal::Capture => (CAPTURE_MODAL_W, CAPTURE_MODAL_H),
            UtilityModal::ClipEditor => (CLIP_MODAL_W, CLIP_MODAL_H),
            UtilityModal::UrlClip => (URL_MODAL_W, URL_MODAL_H),
        }
    }

    fn show_utility_modal(&self, modal: UtilityModal) {
        match modal {
            UtilityModal::Settings => self.ui.set_show_settings(true),
            UtilityModal::AppSettings => self.ui.set_show_app_settings(true),
            UtilityModal::Tools => self.ui.set_show_tools_menu(true),
            UtilityModal::Library => self.ui.set_show_library(true),
            UtilityModal::ImageLibrary => self.ui.set_show_image_library(true),
            UtilityModal::Capture => self.ui.set_show_capture(true),
            UtilityModal::ClipEditor => self.ui.set_show_clip_editor(true),
            UtilityModal::UrlClip => self.ui.set_show_url_clip(true),
        }
    }

    fn hide_utility_modal(&self, modal: UtilityModal) {
        match modal {
            UtilityModal::Settings => self.ui.set_show_settings(false),
            UtilityModal::AppSettings => self.ui.set_show_app_settings(false),
            UtilityModal::Tools => self.ui.set_show_tools_menu(false),
            UtilityModal::Library => self.ui.set_show_library(false),
            UtilityModal::ImageLibrary => self.ui.set_show_image_library(false),
            UtilityModal::Capture => self.ui.set_show_capture(false),
            UtilityModal::ClipEditor => self.ui.set_show_clip_editor(false),
            UtilityModal::UrlClip => self.ui.set_show_url_clip(false),
        }
    }

    fn top_utility_modal(&self) -> Option<UtilityModal> {
        if self.ui.get_show_url_clip() {
            return Some(UtilityModal::UrlClip);
        }
        if self.ui.get_show_clip_editor() {
            return Some(UtilityModal::ClipEditor);
        }
        if self.ui.get_show_capture() {
            return Some(UtilityModal::Capture);
        }
        if self.ui.get_show_image_library() {
            return Some(UtilityModal::ImageLibrary);
        }
        if self.ui.get_show_library() {
            return Some(UtilityModal::Library);
        }
        if self.ui.get_show_tools_menu() {
            return Some(UtilityModal::Tools);
        }
        if self.ui.get_show_app_settings() {
            return Some(UtilityModal::AppSettings);
        }
        if self.ui.get_show_settings() {
            return Some(UtilityModal::Settings);
        }
        None
    }

    fn open_utility_modal(&self, modal: UtilityModal) {
        if let Some(current) = self.top_utility_modal() {
            if current != modal {
                self.hide_utility_modal(current);
                self.modal_back_stack.borrow_mut().push(current);
            }
        }
        let (w, h) = Self::utility_modal_size(modal);
        self.resize_window_for_overlay(w, h);
        self.show_utility_modal(modal);
    }

    fn close_utility_modal(&self, modal: UtilityModal) {
        self.hide_utility_modal(modal);
        if let Some(prev) = self.modal_back_stack.borrow_mut().pop() {
            let (w, h) = Self::utility_modal_size(prev);
            self.resize_window_for_overlay(w, h);
            self.show_utility_modal(prev);
        } else {
            self.restore_compact_window_after_overlay();
        }
    }

    fn page_breadcrumb(&self) -> Result<String> {
        let stack: Vec<i64> = self.nav.borrow().stack().to_vec();
        let mut parts = Vec::with_capacity(stack.len());
        for id in stack {
            let page = self.repo.get_page(id)?;
            parts.push(slugify_page_name(&page.name));
        }
        Ok(format!("/{}", parts.join("/")))
    }

    fn refresh_grid(&self) -> Result<()> {
        let page_id = self.nav.borrow().current();
        let slots = self.repo.list_slots(page_id)?;
        let at_root = self.nav.borrow().is_at_root();
        let models = build_grid_slots(&self.paths.base, &slots, at_root);
        self.ui.set_grid_slots(ModelRc::new(VecModel::from(models)));
        self.ui
            .set_page_path(self.page_breadcrumb()?.into());
        Ok(())
    }

    fn refresh_library(&self) -> Result<()> {
        let sounds = self.repo.list_sounds()?;
        let titles: Vec<SharedString> = sounds.iter().map(|s| s.title.clone().into()).collect();
        self.ui
            .set_library_titles(ModelRc::new(VecModel::from(titles)));
        Ok(())
    }

    fn open_image_library(&self) {
        self.ui.set_show_image_library(true);
        self.ui.set_image_library_selected(-1);
        self.ui.set_image_library_thumb(slint::Image::default());
        let _ = self.refresh_image_library();
        if *self.image_library_pick_mode.borrow() {
            self.ui.set_show_slot_editor(true);
        }
    }

    fn refresh_image_library(&self) -> Result<()> {
        let files = list_slot_images(&self.paths)?;
        *self.image_library_files.borrow_mut() = files.clone();
        let titles: Vec<SharedString> = files
            .iter()
            .map(|f| SharedString::from(image_library_label(f)))
            .collect();
        self.ui
            .set_image_library_titles(ModelRc::new(VecModel::from(titles)));
        Ok(())
    }

    fn selected_image_library_file(&self) -> Option<String> {
        let idx = self.ui.get_image_library_selected();
        if idx < 0 {
            return None;
        }
        self.image_library_files
            .borrow()
            .get(idx as usize)
            .cloned()
    }

    fn image_library_add_file(&self) -> Result<()> {
        let file = rfd::FileDialog::new()
            .set_title(crate::i18n::rfd_add_image())
            .add_filter(
                "Images",
                &["png", "jpg", "jpeg", "gif", "webp", "bmp", "PNG", "JPG", "GIF"],
            )
            .pick_file();
        let Some(path) = file else {
            return Ok(());
        };
        let _name = import_slot_image_from_file(&self.paths, &path)?;
        self.refresh_image_library()?;
        self.set_status(SharedString::from("Image ajoutée à la bibliothèque"));
        Ok(())
    }

    fn image_library_add_url(&self, url: &str) -> Result<()> {
        let custom_name = self.ui.get_image_library_name().to_string();
        let name = import_slot_image_from_url(
            &self.paths,
            url,
            if custom_name.trim().is_empty() {
                None
            } else {
                Some(custom_name.trim())
            },
        )?;
        self.ui.set_image_library_url("".into());
        self.ui.set_image_library_name("".into());
        self.refresh_image_library()?;
        self.set_status(SharedString::from(format!(
            "Image téléchargée : {}",
            image_library_label(&name)
        )));
        Ok(())
    }

    fn image_library_delete(&self) -> Result<()> {
        let Some(filename) = self.selected_image_library_file() else {
            return Ok(());
        };
        let n = self.repo.count_slots_using_image_file(&filename)?;
        if n > 0 {
            self.set_status(SharedString::from(format!(
                "Image utilisée par {n} bouton(s) — suppression annulée"
            )));
            return Ok(());
        }
        if self
            .slot_editor_custom_image
            .borrow()
            .as_deref()
            == Some(filename.as_str())
        {
            *self.slot_editor_custom_image.borrow_mut() = None;
            self.sync_slot_editor_photo_preview();
        }
        delete_slot_image_file(&self.paths, &filename);
        self.refresh_image_library()?;
        self.ui.set_image_library_thumb(slint::Image::default());
        self.set_status(SharedString::from("Image supprimée"));
        Ok(())
    }

    fn image_library_preview_thumb(&self) -> Result<()> {
        let Some(filename) = self.selected_image_library_file() else {
            return Ok(());
        };
        let img = load_slot_photo(&self.paths, &filename).unwrap_or_default();
        self.ui.set_image_library_thumb(img);
        Ok(())
    }

    fn image_library_choose(&self) -> Result<()> {
        if !*self.image_library_pick_mode.borrow() {
            return Ok(());
        }
        let Some(filename) = self.selected_image_library_file() else {
            self.set_status(SharedString::from("Selectionnez une image dans la liste"));
            return Ok(());
        };
        *self.slot_editor_custom_image.borrow_mut() = Some(filename);
        self.ui.set_slot_editor_appearance("image".into());
        self.sync_slot_editor_photo_preview();
        self.ui.set_show_image_library(false);
        *self.image_library_pick_mode.borrow_mut() = false;
        self.ui.set_image_library_pick_mode(false);
        // Garder l'editeur de touche ouvert
        self.ui.set_show_slot_editor(true);
        self.set_status(SharedString::from(
            "Image choisie — verifiez l apercu puis cliquez Sauver",
        ));
        Ok(())
    }

    fn handle_slot_click(&self, row: i32, col: i32) -> Result<()> {
        if is_home_slot(row, col) {
            self.nav.borrow_mut().home(self.root_id);
            self.refresh_grid()?;
            return Ok(());
        }
        if !self.nav.borrow().is_at_root() && is_back_slot(row, col) {
            self.nav.borrow_mut().pop();
            self.refresh_grid()?;
            return Ok(());
        }

        if *self.edit_mode.borrow() {
            *self.editing_slot.borrow_mut() = Some((row, col));
            let slots = self.repo.list_slots(self.nav.borrow().current())?;
            let slot = slots.iter().find(|s| s.row == row && s.col == col);
            self.ui.set_slot_editor_label(
                slot.and_then(|s| s.label.clone())
                    .unwrap_or_default()
                    .into(),
            );
            self.ui.set_slot_editor_kind(
                slot.map(|s| s.kind.as_str().to_string())
                    .unwrap_or_else(|| "empty".into())
                    .into(),
            );
            let kind_str = slot
                .map(|s| s.kind.as_str())
                .unwrap_or("empty");
            self.ui.set_slot_editor_color(
                hex_for_slot_editor(slot, kind_str).into(),
            );
            self.ui.set_slot_editor_script(
                slot.and_then(|s| s.script_command.clone())
                    .unwrap_or_default()
                    .into(),
            );
            self.ui.set_slot_editor_script_shell(
                slot.and_then(|s| s.script_shell.clone())
                    .unwrap_or_else(|| default_script_shell().to_string())
                    .into(),
            );
            *self.slot_editor_custom_image.borrow_mut() = None;
            if let Some(s) = slot {
                if s.kind == SlotKind::Empty {
                    self.ui.set_slot_editor_appearance("color".into());
                    self.ui.set_slot_editor_color(NEUTRAL_SLOT_COLOR.into());
                    self.ui.set_slot_editor_icon("".into());
                } else if is_image_appearance(&s.slot_appearance) {
                    self.ui.set_slot_editor_appearance("image".into());
                    *self.slot_editor_custom_image.borrow_mut() = s
                        .image_path
                        .clone()
                        .filter(|p| {
                            !is_catalog_icon_id(p) && self.paths.image_file(p).is_file()
                        });
                } else {
                    self.ui.set_slot_editor_appearance("color".into());
                    let icon = s
                        .image_path
                        .clone()
                        .filter(|i| !i.trim().is_empty())
                        .unwrap_or_else(|| DEFAULT_SLOT_ICON.to_string());
                    self.ui.set_slot_editor_icon(icon.into());
                }
            } else {
                self.ui.set_slot_editor_appearance("color".into());
                self.ui.set_slot_editor_color(NEUTRAL_SLOT_COLOR.into());
                self.ui.set_slot_editor_icon("".into());
            }
            self.ui.set_slot_editor_image_url("".into());
            let _ = self.prepare_slot_editor(slot);
            self.sync_slot_editor_shortcut_ui(row, col, slot);
            self.open_slot_editor();
            return Ok(());
        }

        let slots = self.repo.list_slots(self.nav.borrow().current())?;
        let Some(slot) = slots.iter().find(|s| s.row == row && s.col == col) else {
            return Ok(());
        };

        match slot.kind {
            SlotKind::Folder => {
                if let Some(child) = slot.child_page_id {
                    self.nav.borrow_mut().push(child);
                    self.refresh_grid()?;
                } else {
                    self.set_status(SharedString::from(
                        "Dossier sans page — Édition : type Dossier, nom (ex. toto), puis Sauver",
                    ));
                }
            }
            SlotKind::Sound => {
                if let Some(sid) = slot.sound_id {
                    let sound = self.repo.get_sound(sid)?;
                    self.audio.lock().unwrap().play_file(
                        &sound.file_path,
                        sound.volume_linear,
                        sound.loudness_gain_db,
                        slot.slot_volume,
                    )?;
                }
            }
            SlotKind::Alarm => {
                self.toggle_alarm_armed(slot)?;
            }
            SlotKind::Script => {
                if let Some(ref cmd) = slot.script_command {
                    let paths = self.shell_paths_from_settings();
                    match crate::script::run_slot_script(
                        cmd,
                        slot.script_shell.as_deref(),
                        &paths,
                    ) {
                        Ok(()) => {
                            let name = slot.label.as_deref().unwrap_or("Script");
                            self.set_status(SharedString::from(format!("Lancé : {name}")));
                        }
                        Err(e) => {
                            self.set_status(SharedString::from(format!("Script : {e:#}")));
                        }
                    }
                } else {
                    self.set_status(SharedString::from(
                        "Aucun script — mode Édition → Script",
                    ));
                }
            }
            SlotKind::Empty => {}
        }
        Ok(())
    }

    fn handle_volume(&self, v: f32) -> Result<()> {
        self.repo.set_setting("global_volume", &v.to_string())?;
        self.audio.lock().unwrap().set_global_volume(v);
        Ok(())
    }

    fn toggle_edit(&self) {
        let mut m = self.edit_mode.borrow_mut();
        *m = !*m;
        self.ui.set_edit_mode(*m);
        if *m {
            self.set_status(SharedString::from(
                "Mode édition — cliquez une touche (pas Accueil/Retour)",
            ));
        } else {
            self.set_status(SharedString::from("Mode lecture"));
        }
    }

    fn save_settings(&self) -> Result<()> {
        let policy = self.ui.get_settings_policy().to_string();
        let max_ch: u32 = self
            .ui
            .get_settings_max_channels()
            .parse()
            .unwrap_or(3);
        let lufs: f32 = self.ui.get_settings_lufs().parse().unwrap_or(-16.0);
        let cap_s: u64 = self
            .ui
            .get_settings_capture_max_s()
            .parse()
            .unwrap_or(60);

        self.repo.set_setting("audio_policy", &policy)?;
        self.repo.set_setting("max_channels", &max_ch.to_string())?;
        self.repo.set_setting("normalize_target_lufs", &lufs.to_string())?;
        self.pipeline.set_target_lufs(lufs);
        self.repo
            .set_setting("capture_max_duration_ms", &(cap_s * 1000).to_string())?;
        let always_on_top = self.ui.get_settings_always_on_top();
        self.repo
            .set_setting("always_on_top", if always_on_top { "1" } else { "0" })?;
        let grid_shortcuts = self.ui.get_settings_grid_shortcuts_enabled();
        self.repo.set_setting(
            "grid_shortcuts_enabled",
            if grid_shortcuts { "1" } else { "0" },
        )?;
        *self.grid_shortcuts_enabled.borrow_mut() = grid_shortcuts;
        let lang = crate::i18n::Lang::from_setting(&self.ui.get_settings_language());
        self.repo.set_setting("ui_language", lang.as_setting())?;
        crate::i18n::apply_slint(lang);
        self.apply_always_on_top(always_on_top);
        self.save_shell_path_settings()?;

        let pol = crate::db::AudioPolicy::from_setting(&policy);
        self.audio.lock().unwrap().set_policy(pol, max_ch);
        self.close_utility_modal(UtilityModal::AppSettings);
        Ok(())
    }

    fn library_add_file(&self) -> Result<()> {
        let file = rfd::FileDialog::new()
            .set_title(crate::i18n::rfd_add_sound())
            .add_filter(
                crate::i18n::rfd_audio_files(),
                &["wav", "WAV", "mp3", "MP3", "ogg", "flac", "m4a", "aac"],
            )
            .pick_file();
        let Some(path) = file else { return Ok(()) };
        let title = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| "Son".into());
        let (mut sound, _) = self
            .pipeline
            .import_file(&path, &title, None, None, "import")?;
        sound.id = self.repo.insert_sound(&sound)?;
        self.refresh_library()?;
        self.set_status(SharedString::from(format!("Ajouté: {}", sound.title)));
        Ok(())
    }

    fn library_delete(&self) -> Result<()> {
        let idx = self.ui.get_library_selected();
        if idx < 0 {
            return Ok(());
        }
        let sounds = self.repo.list_sounds()?;
        let Some(sound) = sounds.get(idx as usize) else {
            return Ok(());
        };
        let n = self.repo.count_slots_using_sound(sound.id)?;
        if n > 0 {
            self.set_status(SharedString::from(format!(
                "Son utilisé par {n} bouton(s) — suppression annulée"
            )));
            return Ok(());
        }
        self.repo.delete_sound(sound.id)?;
        let path = self.paths.sounds.join(&sound.file_path);
        let _ = std::fs::remove_file(path);
        self.refresh_library()?;
        Ok(())
    }

    fn library_preview(&self) -> Result<()> {
        let idx = self.ui.get_library_selected();
        if idx < 0 {
            return Ok(());
        }
        let sounds = self.repo.list_sounds()?;
        let Some(sound) = sounds.get(idx as usize) else {
            return Ok(());
        };
        self.audio.lock().unwrap().play_file(
            &sound.file_path,
            sound.volume_linear,
            sound.loudness_gain_db,
            1.0,
        )?;
        Ok(())
    }

    fn slot_editor_save(&self) -> Result<()> {
        let Some((row, col)) = *self.editing_slot.borrow() else {
            return Ok(());
        };
        let page_id = self.nav.borrow().current();
        let kind = SlotKind::from_str(self.ui.get_slot_editor_kind().as_str());
        let label = self.ui.get_slot_editor_label().to_string();
        let existing = self
            .repo
            .list_slots(page_id)?
            .into_iter()
            .find(|s| s.row == row && s.col == col);

        let sound_id = if kind == SlotKind::Sound || kind == SlotKind::Alarm {
            self.editor_sound_id()?
        } else {
            None
        };

        let child_page_id = if kind == SlotKind::Folder {
            let child_id = self.resolve_folder_child_id(page_id, &existing, &label)?;
            let page_name = folder_page_name(&label);
            self.repo.update_page_name(child_id, &page_name)?;
            Some(child_id)
        } else {
            None
        };

        let (script_command, script_shell) = if kind == SlotKind::Script {
            let cmd = self.ui.get_slot_editor_script().to_string();
            let shell = self.ui.get_slot_editor_script_shell().to_string();
            if cmd.trim().is_empty() {
                (None, None)
            } else {
                (
                    Some(cmd),
                    Some(if shell.trim().is_empty() {
                        default_script_shell().to_string()
                    } else {
                        shell
                    }),
                )
            }
        } else {
            (None, None)
        };

        let alarm_mode = if kind == SlotKind::Alarm {
            self.ui.get_slot_editor_alarm_mode().to_string()
        } else {
            "clock".to_string()
        };

        let (alarm_time, alarm_minutes, alarm_armed, alarm_armed_at_ms) = if kind == SlotKind::Alarm {
            if is_timer_mode(&alarm_mode) {
                let mins = self
                    .parse_editor_alarm_minutes()
                    .or_else(|| existing.as_ref().and_then(|s| s.alarm_minutes))
                    .unwrap_or(10);
                (
                    None,
                    Some(mins),
                    existing.as_ref().map(|s| s.alarm_armed).unwrap_or(false),
                    existing.as_ref().and_then(|s| s.alarm_armed_at_ms),
                )
            } else {
                let h = self.ui.get_slot_editor_alarm_hour_index();
                let m = self.ui.get_slot_editor_alarm_minute_index();
                let Some(at) = alarm_time_from_indices(h, m) else {
                    self.set_status(SharedString::from("Heure invalide"));
                    return Ok(());
                };
                (Some(at), None, false, None)
            }
        } else {
            (None, None, false, None)
        };
        let alarm_status_msg = if kind == SlotKind::Alarm {
            if is_timer_mode(&alarm_mode) {
                format!(
                    "Minuteur {} min — clic touche en lecture pour activer",
                    alarm_minutes.unwrap_or(0)
                )
            } else {
                format!(
                    "Alarme enregistrée — sonnera à {}",
                    alarm_time.as_deref().unwrap_or("?")
                )
            }
        } else {
            String::new()
        };

        let appearance = self.ui.get_slot_editor_appearance().to_string();
        let (slot_appearance, image_path, color_hex) = if kind == SlotKind::Empty {
            (
                "color".to_string(),
                None,
                NEUTRAL_SLOT_COLOR.to_string(),
            )
        } else if is_image_appearance(&appearance) {
            let Some(name) = self.resolve_slot_image_filename(&existing) else {
                self.set_status(SharedString::from(
                    "Mode image : fichier, bibliotheque ou URL puis Sauver",
                ));
                return Ok(());
            };
            (
                "image".to_string(),
                Some(name),
                NEUTRAL_SLOT_COLOR.to_string(),
            )
        } else {
            let icon_id = self.ui.get_slot_editor_icon().to_string();
            let path = if icon_id.trim().is_empty() {
                None
            } else {
                Some(icon_id)
            };
            let hex = normalize_color_hex(&self.ui.get_slot_editor_color().to_string());
            ("color".to_string(), path, hex)
        };

        let slot = Slot {
            page_id,
            row,
            col,
            kind,
            label: if label.is_empty() {
                None
            } else {
                Some(label.clone())
            },
            image_path,
            slot_appearance,
            sound_id: if kind == SlotKind::Sound || kind == SlotKind::Alarm {
                sound_id
            } else {
                None
            },
            child_page_id,
            slot_volume: existing.as_ref().map(|s| s.slot_volume).unwrap_or(1.0),
            color_hex: Some(color_hex),
            script_command,
            script_shell,
            alarm_time,
            alarm_mode,
            alarm_minutes,
            alarm_armed,
            alarm_armed_at_ms,
            shortcut_key: match self.editor_shortcut_key_for_save(row, col, existing.as_ref()) {
                Ok(key) => key,
                Err(conflict) => {
                    self.ui
                        .set_slot_editor_shortcut_error(conflict.message().into());
                    return Ok(());
                }
            },
        };
        self.repo.upsert_slot(&slot)?;
        self.close_slot_editor();
        self.refresh_grid()?;
        if kind == SlotKind::Folder {
            let name = folder_page_name(&label);
            self.set_status(SharedString::from(format!(
                "Dossier « {name} » — en Lecture, cliquez la touche pour ouvrir"
            )));
        } else if kind == SlotKind::Alarm {
            self.set_status(SharedString::from(alarm_status_msg));
        } else {
            self.set_status(SharedString::from("Bouton enregistré"));
        }
        Ok(())
    }

    fn resolve_folder_child_id(
        &self,
        page_id: i64,
        existing: &Option<Slot>,
        label: &str,
    ) -> Result<i64> {
        if let Some(s) = existing {
            if let Some(cid) = s.child_page_id {
                return Ok(cid);
            }
        }
        let name = folder_page_name(label);
        self.repo.create_child_page(page_id, &name)
    }

    fn create_folder_page(&self) -> Result<()> {
        let _ = self.slot_editor_save();
        Ok(())
    }

    fn prepare_clip_editor(&self) {
        self.ui.set_clip_source_path("".into());
        self.ui.set_clip_title("".into());
        self.ui.set_clip_duration_secs(0.0);
        self.ui.set_clip_range_start(0.0);
        self.ui.set_clip_range_end(1.0);
        self.ui.set_clip_waveform(ModelRc::new(VecModel::from(Vec::<f32>::new())));
        self.ui.set_clip_status_msg("".into());
        self.sync_clip_range_labels();
    }

    fn cleanup_url_preview(&self) {
        if let Some(path) = self.url_preview_wav.lock().ok().and_then(|mut g| g.take()) {
            let _ = std::fs::remove_file(&path);
        }
    }

    fn prepare_url_clip(&self) {
        self.cleanup_url_preview();
        self.ui.set_url_input("".into());
        self.ui.set_url_title("".into());
        self.ui.set_url_duration_secs(0.0);
        self.ui.set_url_range_start(0.0);
        self.ui.set_url_range_end(1.0);
        self.ui
            .set_url_waveform(ModelRc::new(VecModel::from(Vec::<f32>::new())));
        self.ui.set_url_bypass_limit(false);
        self.ui.set_url_loading(false);
        self.ui.set_url_status_msg("".into());
        self.ui.set_url_status_msg("".into());
        self.sync_url_range_labels();
    }

    fn sync_url_range_labels(&self) {
        sync_url_range_labels_ui(&self.ui);
    }

    fn url_load(self: &Rc<Self>) {
        if self.ui.get_url_loading() {
            return;
        }
        let url = self.ui.get_url_input().to_string();
        self.ui.set_url_status_msg("".into());
        if url.trim().is_empty() {
            self.ui
                .set_url_status_msg(crate::i18n::url_need_url().into());
            return;
        }
        let bypass = self.ui.get_url_bypass_limit();

        let info = match probe_url_info(&self.paths, url.trim()) {
            Ok(info) => info,
            Err(e) => {
                self.ui
                    .set_url_status_msg(format!("{e:#}").into());
                return;
            }
        };
        let duration = info.duration_secs.max(0.0);

        if duration > URL_DEFAULT_MAX_DURATION_SECS && !bypass {
            self.ui.set_url_status_msg(
                crate::i18n::url_too_long(&format_clip_time(duration)).into(),
            );
            return;
        }

        self.cleanup_url_preview();
        self.ui.set_url_duration_secs(0.0);
        self.ui
            .set_url_waveform(ModelRc::new(VecModel::from(Vec::<f32>::new())));
        self.ui.set_url_loading(true);
        if duration > URL_DEFAULT_MAX_DURATION_SECS {
            self.ui
                .set_url_status_msg(crate::i18n::url_downloading_long(&format_clip_time(duration)).into());
        } else {
            self.ui
                .set_url_status_msg(crate::i18n::url_downloading().into());
        }

        let weak_ui = self.ui.as_weak();
        let url_preview = self.url_preview_wav.clone();
        let paths = self.paths.clone();
        let url_trimmed = url.trim().to_string();
        let probe_info = info;

        std::thread::spawn(move || {
            let result: Result<(PathBuf, ClipSourceInfo), anyhow::Error> = (|| {
                let path = download_url_audio_full(&paths, &url_trimmed)?;
                let analyzed = analyze_clip_source(&paths, path.as_path())?;
                Ok((path, analyzed))
            })();

            let _ = slint::invoke_from_event_loop(move || {
                let Some(ui) = weak_ui.upgrade() else {
                    return;
                };
                ui.set_url_loading(false);
                match result {
                    Err(e) => {
                        ui.set_url_status_msg(format!("{e:#}").into());
                    }
                    Ok((path, analyzed)) => {
                        finish_url_load(&ui, &url_preview, probe_info, path, analyzed);
                    }
                }
            });
        });
    }

    fn url_preview_selection(&self) -> Result<()> {
        let path = self
            .url_preview_wav
            .lock()
            .map_err(|_| anyhow::anyhow!("verrou URL"))?
            .clone()
            .context("Chargez d'abord l'URL")?;
        let duration = self.ui.get_url_duration_secs() as f64;
        if duration <= 0.0 {
            return Ok(());
        }
        let (start, end) = range_to_seconds(
            duration,
            self.ui.get_url_range_start(),
            self.ui.get_url_range_end(),
        );
        self.audio.lock().unwrap().play_file_segment(
            path.as_path(),
            start,
            end,
        )?;
        self.set_status(SharedString::from(format!(
            "Lecture {start} → {end}",
            start = format_clip_time(start),
            end = format_clip_time(end)
        )));
        Ok(())
    }

    fn clip_browse(&self) -> Result<()> {
        let file = rfd::FileDialog::new()
            .set_title(crate::i18n::rfd_clip_source())
            .add_filter(
                "Audio",
                &["wav", "mp3", "ogg", "flac", "m4a", "aac", "WAV", "MP3"],
            )
            .pick_file();
        let Some(path) = file else {
            return Ok(());
        };
        self.ui
            .set_clip_source_path(path.display().to_string().into());
        if self.ui.get_clip_title().trim().is_empty() {
            let stem = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "Clip".into());
            self.ui.set_clip_title(stem.into());
        }
        self.load_clip_source(path.as_path())?;
        Ok(())
    }

    fn load_clip_source(&self, path: &std::path::Path) -> Result<()> {
        self.ui.set_clip_status_msg("".into());
        let info = analyze_clip_source(&self.paths, path)?;
        let duration = info.duration_secs.max(0.1);
        self.ui.set_clip_duration_secs(duration as f32);
        self.ui
            .set_clip_waveform(ModelRc::new(VecModel::from(info.peaks)));
        self.ui.set_clip_range_start(0.0);
        let end_ratio = (5.0f64 / duration).min(1.0) as f32;
        self.ui.set_clip_range_end(end_ratio.max(0.05));
        self.sync_clip_range_labels();
        Ok(())
    }

    fn sync_clip_range_labels(&self) {
        let duration = self.ui.get_clip_duration_secs() as f64;
        let (start, end) = range_to_seconds(
            duration,
            self.ui.get_clip_range_start(),
            self.ui.get_clip_range_end(),
        );
        self.ui
            .set_clip_start_label(SharedString::from(format_clip_time(start)));
        self.ui
            .set_clip_end_label(SharedString::from(format_clip_time(end)));
    }

    fn clip_preview_selection(&self) -> Result<()> {
        let path = self.ui.get_clip_source_path();
        if path.is_empty() {
            self.set_status(SharedString::from("Choisissez un fichier audio"));
            return Ok(());
        }
        let duration = self.ui.get_clip_duration_secs() as f64;
        if duration <= 0.0 {
            return Ok(());
        }
        let (start, end) = range_to_seconds(
            duration,
            self.ui.get_clip_range_start(),
            self.ui.get_clip_range_end(),
        );
        self.audio.lock().unwrap().play_file_segment(
            PathBuf::from(path.as_str()).as_path(),
            start,
            end,
        )?;
        self.set_status(SharedString::from(format!(
            "Lecture {start} → {end}",
            start = format_clip_time(start),
            end = format_clip_time(end)
        )));
        Ok(())
    }

    fn clip_export(&self) -> Result<()> {
        let path = self.ui.get_clip_source_path();
        if path.is_empty() {
            self.set_status(SharedString::from("Choisissez un fichier audio"));
            return Ok(());
        }
        let mut title = self.ui.get_clip_title().to_string();
        if title.trim().is_empty() {
            self.set_status(SharedString::from("Donnez un nom au son"));
            return Ok(());
        }
        title = title.trim().to_string();
        let duration = self.ui.get_clip_duration_secs() as f64;
        let (start, end) = range_to_seconds(
            duration.max(0.1),
            self.ui.get_clip_range_start(),
            self.ui.get_clip_range_end(),
        );
        self.set_status(SharedString::from("Découpe en cours…"));
        let (mut sound, _) = self.pipeline.import_file(
            PathBuf::from(path.as_str()).as_path(),
            &title,
            Some(start),
            Some(end),
            "clip_local",
        )?;
        sound.id = self.repo.insert_sound(&sound)?;
        self.refresh_library()?;
        self.close_utility_modal(UtilityModal::ClipEditor);
        self.set_status(SharedString::from(format!("Ajouté à la bibliothèque : {title}")));
        Ok(())
    }

    fn url_extract(&self) -> Result<()> {
        let path = self
            .url_preview_wav
            .lock()
            .map_err(|_| anyhow::anyhow!("verrou URL"))?
            .clone()
            .context("Chargez d'abord l'URL (bouton Charger)")?;
        let mut title = self.ui.get_url_title().to_string();
        if title.trim().is_empty() {
            self.set_status(SharedString::from("Donnez un nom au son"));
            return Ok(());
        }
        title = title.trim().to_string();
        let duration = self.ui.get_url_duration_secs() as f64;
        if duration <= 0.0 {
            self.set_status(SharedString::from("Chargez d'abord l'URL"));
            return Ok(());
        }
        let (start, end) = range_to_seconds(
            duration,
            self.ui.get_url_range_start(),
            self.ui.get_url_range_end(),
        );
        self.set_status(SharedString::from("Découpe et import en cours…"));
        let (mut sound, _) = self.pipeline.import_file(
            path.as_path(),
            &title,
            Some(start),
            Some(end),
            "clip_url",
        )?;
        sound.id = self.repo.insert_sound(&sound)?;
        self.cleanup_url_preview();
        self.ui.set_url_duration_secs(0.0);
        self.refresh_library()?;
        self.close_utility_modal(UtilityModal::UrlClip);
        self.set_status(SharedString::from(format!("Ajouté à la bibliothèque : {title}")));
        Ok(())
    }

    fn capture_toggle(self: &Rc<Self>) -> Result<()> {
        let state = self.ui.get_capture_state();
        if state.as_str() == "recording" {
            self.stop_recording()?;
        } else if state.as_str() == "ready" {
            self.discard_capture_file();
            self.reset_capture_ui();
            self.start_recording()?;
        } else {
            self.start_recording()?;
        }
        Ok(())
    }

    fn reset_capture_ui(&self) {
        self.stop_capture_poll();
        *self.recording.borrow_mut() = None;
        self.ui.set_capture_state("idle".into());
        self.ui.set_capture_timer("00:00".into());
        self.ui
            .set_capture_duration_label(SharedString::from(crate::i18n::duration_placeholder()));
        self.ui.set_capture_level(0.0);
        self.ui.set_capture_can_save(false);
    }

    fn start_capture_poll(self: &Rc<Self>) {
        self.stop_capture_poll();
        let weak = Rc::downgrade(self);
        let timer = Timer::default();
        timer.start(TimerMode::Repeated, Duration::from_millis(200), move || {
            let Some(app) = weak.upgrade() else {
                return;
            };
            let (ms, level) = {
                let guard = app.recording.borrow();
                let Some(session) = guard.as_ref() else {
                    return;
                };
                (session.elapsed_ms(), session.level_rms())
            };
            let max_ms = app.repo.capture_max_duration_ms().unwrap_or(60_000);
            app.ui
                .set_capture_timer(SharedString::from(format_mm_ss(ms)));
            app.ui.set_capture_level(level);
            app.ui.set_capture_duration_label(SharedString::from(
                crate::i18n::duration_label(&format_mm_ss(ms)),
            ));
            if ms >= max_ms {
                let _ = app.stop_recording();
            }
        });
        *self.capture_poll.borrow_mut() = Some(timer);
    }

    fn stop_capture_poll(&self) {
        if let Some(timer) = self.capture_poll.borrow_mut().take() {
            timer.stop();
        }
    }

    fn start_recording(self: &Rc<Self>) -> Result<()> {
        self.discard_capture_file();
        let source = CaptureSource::from_setting(&self.ui.get_capture_source().to_string());
        if source == CaptureSource::SystemLoopback && !CaptureSource::loopback_available() {
            self.set_status(SharedString::from(
                "Capture PC indisponible — Windows ou Linux (monitor Pulse/PipeWire)",
            ));
            return Ok(());
        }
        let device = self
            .repo
            .get_setting("capture_input_device")?
            .unwrap_or_default();
        let session = RecordingSession::start(
            source,
            if source == CaptureSource::Microphone {
                Some(device.as_str())
            } else {
                None
            },
        )?;
        *self.recording.borrow_mut() = Some(session);
        self.ui.set_capture_state("recording".into());
        self.ui.set_capture_can_save(false);
        self.ui.set_capture_timer("00:00".into());
        self.ui
            .set_capture_duration_label(SharedString::from(crate::i18n::duration_zero()));
        let msg = if source == CaptureSource::SystemLoopback {
            "Enregistrement sortie PC…"
        } else {
            "Enregistrement micro…"
        };
        self.set_status(SharedString::from(msg));
        self.start_capture_poll();
        Ok(())
    }

    fn set_capture_source(&self, raw: &str) {
        let source = CaptureSource::from_setting(raw);
        if source == CaptureSource::SystemLoopback && !CaptureSource::loopback_available() {
            self.set_status(SharedString::from(
                "Capture PC indisponible sur cette machine",
            ));
            return;
        }
        self.ui
            .set_capture_source(SharedString::from(source.as_setting()));
        let _ = self
            .repo
            .set_setting("capture_source", source.as_setting());
        self.ui
            .set_capture_device_hint(SharedString::from(capture_source_label(source)));
    }

    fn sync_capture_source_ui(&self) -> Result<()> {
        self.ui
            .set_capture_loopback_available(CaptureSource::loopback_available());
        let stored = self
            .repo
            .get_setting("capture_source")?
            .unwrap_or_else(|| "microphone".to_string());
        let source = CaptureSource::from_setting(&stored);
        self.ui
            .set_capture_source(SharedString::from(source.as_setting()));
        self.ui
            .set_capture_device_hint(SharedString::from(capture_source_label(source)));
        Ok(())
    }

    fn stop_recording(&self) -> Result<()> {
        self.stop_capture_poll();
        let session = self.recording.borrow_mut().take();
        let Some(session) = session else {
            return Ok(());
        };
        let min_ms = self.repo.capture_min_duration_ms()?;
        let elapsed = session.elapsed_ms();
        if elapsed < min_ms {
            self.reset_capture_ui();
            self.set_status(SharedString::from(format!(
                "Trop court (min. {} s) — recommencez",
                (min_ms + 999) / 1000
            )));
            return Ok(());
        }
        let captured = session.stop()?;
        let dur = captured.duration_ms();
        let wav = self.paths.temp.join(format!("cap_{}.wav", uuid::Uuid::new_v4()));
        captured.write_wav(&wav)?;
        *self.last_capture_wav.borrow_mut() = Some(wav);
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        self.ui
            .set_capture_title(SharedString::from(format!("capture_{ts}")));
        self.ui.set_capture_state("ready".into());
        self.ui
            .set_capture_timer(SharedString::from(format_mm_ss(dur)));
        self.ui.set_capture_duration_label(SharedString::from(
            crate::i18n::duration_label(&format_mm_ss(dur)),
        ));
        self.ui.set_capture_can_save(true);
        self.set_status(SharedString::from(
            "Enregistrement terminé — vérifiez le nom puis Sauver",
        ));
        Ok(())
    }

    fn discard_capture_file(&self) {
        if let Some(path) = self.last_capture_wav.borrow_mut().take() {
            let _ = std::fs::remove_file(path);
        }
    }

    fn cancel_capture(&self) {
        self.stop_capture_poll();
        *self.recording.borrow_mut() = None;
        self.discard_capture_file();
        self.reset_capture_ui();
        self.ui.set_capture_title("".into());
        self.set_status(SharedString::from("Enregistrement annulé"));
    }

    fn capture_save(&self) -> Result<()> {
        if self.ui.get_capture_state().as_str() != "ready" {
            self.set_status(SharedString::from(
                "REC puis Stop avant de sauver",
            ));
            return Ok(());
        }
        let wav = self.last_capture_wav.borrow().clone();
        let Some(wav) = wav else {
            self.set_status(SharedString::from("Aucun enregistrement — REC puis Stop"));
            return Ok(());
        };
        let mut title = self.ui.get_capture_title().to_string();
        if title.trim().is_empty() {
            self.set_status(SharedString::from("Donnez un nom à l'enregistrement"));
            return Ok(());
        }
        title = title.trim().to_string();
        self.set_status(SharedString::from("Normalisation en cours…"));
        let (mut sound, _) = self
            .pipeline
            .import_file(&wav, &title, None, None, "capture")?;
        sound.id = self.repo.insert_sound(&sound)?;
        let _ = std::fs::remove_file(&wav);
        *self.last_capture_wav.borrow_mut() = None;
        self.refresh_library()?;
        self.close_utility_modal(UtilityModal::Capture);
        self.reset_capture_ui();
        self.set_status(SharedString::from(format!("Ajouté à la bibliothèque : {title}")));
        Ok(())
    }

    fn capture_refine(&self) -> Result<()> {
        let wav = self.last_capture_wav.borrow().clone();
        let Some(wav) = wav else {
            return Ok(());
        };
        self.ui
            .set_clip_source_path(wav.display().to_string().into());
        let _ = self.load_clip_source(wav.as_path());
        self.open_utility_modal(UtilityModal::ClipEditor);
        Ok(())
    }

    fn set_status(&self, _msg: SharedString) {
        // Messages volontairement non affichés (évite de décaler la barre de volume).
    }

    fn current_window_logical_size(&self) -> (u32, u32) {
        let window = self.ui.window();
        let scale = window.scale_factor();
        let size = window.size();
        let w = ((size.width as f32) / scale).round().max(480.0) as u32;
        let h = ((size.height as f32) / scale).round().max(420.0) as u32;
        (w, h)
    }

    fn apply_window_logical_size(&self, w: u32, h: u32) {
        let window = self.ui.window();
        let scale = window.scale_factor();
        window.set_size(slint::PhysicalSize::new(
            (w as f32 * scale).round() as u32,
            (h as f32 * scale).round() as u32,
        ));
    }

    fn editing_slot_sound_id(&self) -> Option<i64> {
        let (row, col) = (*self.editing_slot.borrow())?;
        let page_id = self.nav.borrow().current();
        self.repo
            .list_slots(page_id)
            .ok()?
            .into_iter()
            .find(|s| s.row == row && s.col == col)
            .and_then(|s| s.sound_id)
    }

    fn resolve_slot_image_filename(&self, existing: &Option<Slot>) -> Option<String> {
        let file = self
            .slot_editor_custom_image
            .borrow()
            .clone()
            .or_else(|| existing.as_ref().and_then(|s| s.image_path.clone()));
        file.filter(|f| {
            let t = f.trim();
            !t.is_empty() && !is_catalog_icon_id(t) && self.paths.image_file(t).is_file()
        })
    }

    fn clear_invalid_editor_image_inheritance(&self) {
        let mut guard = self.slot_editor_custom_image.borrow_mut();
        if let Some(ref name) = *guard {
            if is_catalog_icon_id(name) || !self.paths.image_file(name).is_file() {
                *guard = None;
            }
            return;
        }
        let Some((row, col)) = *self.editing_slot.borrow() else {
            return;
        };
        let page_id = self.nav.borrow().current();
        let Ok(slots) = self.repo.list_slots(page_id) else {
            return;
        };
        let Some(s) = slots.iter().find(|s| s.row == row && s.col == col) else {
            return;
        };
        if !is_image_appearance(&s.slot_appearance) {
            return;
        }
        if let Some(ref p) = s.image_path {
            if !is_catalog_icon_id(p) && self.paths.image_file(p).is_file() {
                *guard = Some(p.clone());
            }
        }
    }

    fn slot_editor_modal_height(ui: &AppWindow) -> u32 {
        let kind = ui.get_slot_editor_kind();
        let appearance = ui.get_slot_editor_appearance();
        let alarm_mode = ui.get_slot_editor_alarm_mode();

        let mut h = 270u32;
        match kind.as_str() {
            "empty" => h += 40,
            "folder" => h += 32,
            _ => {}
        }
        if kind.as_str() != "empty" {
            h += if appearance.as_str() == "image" { 330 } else { 120 };
            match kind.as_str() {
                "sound" => h += 150,
                "script" => h += 200,
                "alarm" => {
                    h += if alarm_mode.as_str() == "clock" {
                        310
                    } else {
                        400
                    };
                }
                _ => {}
            }
        }
        h += 24;
        h.clamp(SLOT_EDITOR_MODAL_H_MIN, SLOT_EDITOR_MODAL_H_MAX)
    }

    fn resize_slot_editor_if_open(&self) {
        if self.ui.get_show_slot_editor() {
            let h = Self::slot_editor_modal_height(&self.ui);
            self.resize_window_for_overlay(SLOT_EDITOR_MODAL_W, h);
        }
    }

    fn open_slot_editor(&self) {
        let h = Self::slot_editor_modal_height(&self.ui);
        self.resize_window_for_overlay(SLOT_EDITOR_MODAL_W, h);
        self.ui.set_show_slot_editor(true);
    }

    fn close_slot_editor(&self) {
        self.ui.set_show_slot_editor(false);
        self.ui.set_show_image_library(false);
        self.ui.set_show_slot_color_picker(false);
        self.ui.set_show_slot_icon_picker(false);
        self.ui.set_show_library(false);
        *self.image_library_pick_mode.borrow_mut() = false;
        self.ui.set_image_library_pick_mode(false);
        *self.library_pick_mode.borrow_mut() = false;
        self.ui.set_library_pick_mode(false);
        *self.slot_editor_selected_sound_id.borrow_mut() = None;
        self.ui.set_slot_editor_sound_title("".into());
        self.modal_back_stack.borrow_mut().clear();
        *self.slot_editor_custom_image.borrow_mut() = None;
        self.restore_compact_window_after_overlay();
    }

    fn init_slot_editor_lists(&self) {
        let hours: Vec<SharedString> = (0..24)
            .map(|h| format!("{h:02}").into())
            .collect();
        let minutes: Vec<SharedString> = (0..60)
            .map(|m| format!("{m:02}").into())
            .collect();
        self.ui.set_slot_editor_hour_options(ModelRc::new(VecModel::from(hours)));
        self.ui
            .set_slot_editor_minute_options(ModelRc::new(VecModel::from(minutes)));
    }

    fn refresh_alarm_presets_ui(&self) -> Result<()> {
        let presets = self.repo.alarm_preset_minutes()?;
        self.ui
            .set_alarm_presets(ModelRc::new(VecModel::from(presets)));
        Ok(())
    }

    fn sync_slot_editor_sound(&self, sound_id: Option<i64>) -> Result<()> {
        *self.slot_editor_selected_sound_id.borrow_mut() = sound_id;
        if let Some(id) = sound_id {
            let sound = self.repo.get_sound(id)?;
            self.ui
                .set_slot_editor_sound_title(SharedString::from(sound.title.as_str()));
        } else {
            self.ui.set_slot_editor_sound_title("".into());
        }
        Ok(())
    }

    fn open_sound_library_picker(&self) {
        *self.library_pick_mode.borrow_mut() = true;
        self.ui.set_library_pick_mode(true);
        let current = *self.slot_editor_selected_sound_id.borrow();
        let _ = self.refresh_library();
        if let Ok(sounds) = self.repo.list_sounds() {
            let sel = current
                .and_then(|id| sounds.iter().position(|s| s.id == id))
                .map(|i| i as i32)
                .unwrap_or(-1);
            self.ui.set_library_selected(sel);
        } else {
            self.ui.set_library_selected(-1);
        }
        self.ui.set_show_library(true);
        self.ui.set_show_slot_editor(true);
    }

    fn library_choose(&self) -> Result<()> {
        if !*self.library_pick_mode.borrow() {
            return Ok(());
        }
        let idx = self.ui.get_library_selected();
        if idx < 0 {
            self.set_status(SharedString::from("Sélectionnez un son dans la liste"));
            return Ok(());
        }
        let sound = self
            .repo
            .list_sounds()?
            .get(idx as usize)
            .cloned()
            .context("son introuvable")?;
        *self.slot_editor_selected_sound_id.borrow_mut() = Some(sound.id);
        self.ui
            .set_slot_editor_sound_title(SharedString::from(sound.title.as_str()));
        self.ui.set_show_library(false);
        *self.library_pick_mode.borrow_mut() = false;
        self.ui.set_library_pick_mode(false);
        self.ui.set_show_slot_editor(true);
        self.set_status(SharedString::from(
            "Son choisi — vérifiez puis cliquez Sauver",
        ));
        Ok(())
    }

    fn prepare_slot_editor(&self, slot: Option<&Slot>) -> Result<()> {
        self.sync_slot_editor_color_preview();
        self.sync_slot_editor_icon_preview();
        self.sync_slot_editor_photo_preview();
        self.sync_slot_editor_sound(slot.and_then(|s| s.sound_id))?;
        if let Some(s) = slot {
            if s.kind == SlotKind::Alarm {
                let mode = if is_timer_mode(&s.alarm_mode) {
                    "timer"
                } else {
                    "clock"
                };
                self.ui.set_slot_editor_alarm_mode(mode.into());
                if is_timer_mode(&s.alarm_mode) {
                    let mins = s.alarm_minutes.unwrap_or(10);
                    self.ui.set_slot_editor_alarm_minutes(mins);
                    self.ui
                        .set_slot_editor_alarm_minutes_text(mins.to_string().into());
                } else {
                    let hm = s.alarm_time.as_deref().unwrap_or("12:00");
                    let (h, m) = indices_from_alarm_time(hm);
                    self.ui.set_slot_editor_alarm_hour_index(h);
                    self.ui.set_slot_editor_alarm_minute_index(m);
                }
            }
        }
        Ok(())
    }

    fn sync_slot_editor_color_preview(&self) {
        let hex = self.ui.get_slot_editor_color().to_string();
        let brush = parse_hex(&hex)
            .or_else(|| parse_hex(DEFAULT_SLOT_COLOR))
            .unwrap_or_else(|| parse_hex(DEFAULT_SLOT_COLOR).unwrap());
        self.ui.set_slot_editor_color_brush(brush);
    }

    fn open_slot_color_picker(&self) {
        let hex = self.ui.get_slot_editor_color().to_string();
        let (r, g, b) = hex_to_rgb(&hex).unwrap_or((255, 152, 0));
        self.ui.set_slot_color_picker_r(r as i32);
        self.ui.set_slot_color_picker_g(g as i32);
        self.ui.set_slot_color_picker_b(b as i32);
        self.sync_slot_color_picker_preview();
        self.ui.set_show_slot_color_picker(true);
    }

    fn sync_slot_color_picker_preview(&self) {
        let r = self.ui.get_slot_color_picker_r().clamp(0, 255) as u8;
        let g = self.ui.get_slot_color_picker_g().clamp(0, 255) as u8;
        let b = self.ui.get_slot_color_picker_b().clamp(0, 255) as u8;
        self.ui.set_slot_color_picker_preview(brush_from_rgb(r, g, b));
    }

    fn apply_slot_color_from_picker(&self) {
        let r = self.ui.get_slot_color_picker_r().clamp(0, 255) as u8;
        let g = self.ui.get_slot_color_picker_g().clamp(0, 255) as u8;
        let b = self.ui.get_slot_color_picker_b().clamp(0, 255) as u8;
        let hex = rgb_to_hex(r, g, b);
        self.ui.set_slot_editor_color(hex.clone().into());
        self.sync_slot_editor_color_preview();
        self.ui.set_show_slot_color_picker(false);
    }

    fn sync_slot_editor_icon_preview(&self) {
        let id = self.ui.get_slot_editor_icon().to_string();
        let img = if id.trim().is_empty() {
            slint::Image::default()
        } else {
            load_icon(&self.paths.base, &id).unwrap_or_default()
        };
        self.ui.set_slot_editor_icon_preview(img);
    }

    fn sync_slot_editor_photo_preview(&self) {
        let img = self
            .slot_editor_custom_image
            .borrow()
            .as_ref()
            .and_then(|f| load_slot_photo(&self.paths, f))
            .unwrap_or_default();
        self.ui.set_slot_editor_photo_preview(img);
    }

    fn import_slot_image_from_dialog(&self) -> Result<()> {
        let file = rfd::FileDialog::new()
            .set_title(crate::i18n::rfd_slot_image())
            .add_filter(
                "Images",
                &["png", "jpg", "jpeg", "gif", "webp", "bmp", "PNG", "JPG", "GIF"],
            )
            .pick_file();
        let Some(path) = file else {
            return Ok(());
        };
        let name = import_slot_image_from_file(&self.paths, &path)?;
        *self.slot_editor_custom_image.borrow_mut() = Some(name);
        self.ui.set_slot_editor_appearance("image".into());
        self.sync_slot_editor_photo_preview();
        self.set_status(SharedString::from(
            "Image importee — cliquez Sauver pour l appliquer sur la touche",
        ));
        Ok(())
    }

    fn import_slot_image_from_url_field(&self, url: &str) -> Result<()> {
        let name = import_slot_image_from_url(&self.paths, url, None)?;
        *self.slot_editor_custom_image.borrow_mut() = Some(name);
        self.ui.set_slot_editor_appearance("image".into());
        self.sync_slot_editor_photo_preview();
        self.set_status(SharedString::from(
            "Image telechargee — cliquez Sauver pour l appliquer sur la touche",
        ));
        Ok(())
    }

    fn clear_slot_editor_custom_image(&self) {
        *self.slot_editor_custom_image.borrow_mut() = None;
        self.ui.set_slot_editor_image_url("".into());
        self.sync_slot_editor_photo_preview();
        self.set_status(SharedString::from("Image retirée (Sauver pour confirmer)"));
    }

    fn editor_sound_id(&self) -> Result<Option<i64>> {
        Ok(*self.slot_editor_selected_sound_id.borrow())
    }

    fn preview_editor_sound(&self) -> Result<()> {
        let Some(sid) = self.editor_sound_id()? else {
            self.set_status(SharedString::from(
                "Choisissez un son via Bibliothèque",
            ));
            return Ok(());
        };
        let sound = self.repo.get_sound(sid)?;
        self.audio.lock().unwrap().play_file(
            &sound.file_path,
            sound.volume_linear,
            sound.loudness_gain_db,
            1.0,
        )?;
        Ok(())
    }

    fn parse_editor_alarm_minutes(&self) -> Option<i32> {
        let text = self.ui.get_slot_editor_alarm_minutes_text().to_string();
        if let Ok(v) = text.trim().parse::<i32>() {
            if v > 0 && v <= 24 * 60 {
                return Some(v);
            }
        }
        let m = self.ui.get_slot_editor_alarm_minutes();
        if m > 0 { Some(m) } else { None }
    }

    fn toggle_alarm_armed(&self, slot: &Slot) -> Result<()> {
        let mut updated = slot.clone();
        if updated.alarm_armed {
            updated.alarm_armed = false;
            updated.alarm_armed_at_ms = None;
        } else {
            if updated.sound_id.is_none() {
                return Ok(());
            }
            if is_timer_mode(&updated.alarm_mode) {
                let mins = updated.alarm_minutes.unwrap_or(0);
                if mins <= 0 {
                    return Ok(());
                }
                updated.alarm_armed = true;
                updated.alarm_armed_at_ms = Some(now_ms());
            } else {
                let has_time = updated
                    .alarm_time
                    .as_deref()
                    .map(str::trim)
                    .filter(|t| !t.is_empty())
                    .is_some();
                if !has_time {
                    return Ok(());
                }
                updated.alarm_armed = true;
                updated.alarm_armed_at_ms = None;
            }
        }
        self.repo.upsert_slot(&updated)?;
        self.refresh_grid()?;
        Ok(())
    }

    fn disarm_timer_slot(&self, slot: &Slot) {
        let mut updated = slot.clone();
        updated.alarm_armed = false;
        updated.alarm_armed_at_ms = None;
        let _ = self.repo.upsert_slot(&updated);
        let _ = self.refresh_grid();
    }

    fn overlay_window_size(modal_w: u32, modal_h: u32) -> (u32, u32) {
        (
            modal_w.max(COMPACT_WINDOW_W) + 28,
            modal_h + 52,
        )
    }

    fn resize_window_for_overlay(&self, modal_w: u32, modal_h: u32) {
        let (need_w, need_h) = Self::overlay_window_size(modal_w, modal_h);
        let (w, h) = self.current_window_logical_size();
        if self.window_size_before_overlay.borrow().is_none() {
            *self.window_size_before_overlay.borrow_mut() = Some((w, h));
        }
        if w != need_w || h != need_h {
            self.apply_window_logical_size(need_w, need_h);
        }
    }

    fn restore_compact_window_after_overlay(&self) {
        self.window_size_before_overlay.borrow_mut().take();
        self.apply_compact_window_size();
    }

    fn apply_compact_window_size(&self) {
        let w = self
            .repo
            .get_setting("window_width")
            .ok()
            .flatten()
            .and_then(|v| v.parse().ok())
            .unwrap_or(COMPACT_WINDOW_W)
            .max(480);
        self.apply_window_logical_size(w, COMPACT_WINDOW_H);
    }

    fn apply_native_window_chrome(&self) {
        let weak = self.ui.as_weak();
        let _ = slint::invoke_from_event_loop(move || {
            let Some(ui) = weak.upgrade() else {
                return;
            };
            #[cfg(windows)]
            apply_windows_rounded_corners(&ui.window());
        });
    }

    fn populate_settings_ui(&self) {
        let policy = self
            .repo
            .get_setting("audio_policy")
            .ok()
            .flatten()
            .unwrap_or_else(|| "stop_previous".into());
        let max_ch = self
            .repo
            .get_setting("max_channels")
            .ok()
            .flatten()
            .unwrap_or_else(|| "3".into());
        let lufs = self
            .repo
            .get_setting("normalize_target_lufs")
            .ok()
            .flatten()
            .unwrap_or_else(|| "-16".into());
        let cap_max = self
            .repo
            .capture_max_duration_ms()
            .unwrap_or(60_000)
            / 1000;
        self.ui.set_settings_policy(policy.into());
        self.ui
            .set_settings_max_channels(SharedString::from(max_ch));
        self.ui.set_settings_lufs(SharedString::from(lufs));
        self.ui
            .set_settings_capture_max_s(SharedString::from(cap_max.to_string()));
        self.ui
            .set_settings_always_on_top(self.always_on_top_enabled());
        self.load_grid_shortcuts_setting();
        self.load_shell_path_settings_ui();
        let lang = self
            .repo
            .get_setting("ui_language")
            .ok()
            .flatten()
            .unwrap_or_else(|| "en".into());
        self.ui
            .set_settings_language(SharedString::from(crate::i18n::Lang::from_setting(&lang).as_setting()));
    }

    fn load_settings_ui(&self) {
        self.populate_settings_ui();
        self.open_utility_modal(UtilityModal::Settings);
    }

    fn open_app_settings_ui(&self) {
        self.populate_settings_ui();
        self.open_utility_modal(UtilityModal::AppSettings);
    }

    fn shell_paths_from_settings(&self) -> crate::script::ShellPaths {
        crate::script::ShellPaths {
            powershell: self.repo.get_setting("shell_path_powershell").ok().flatten(),
            cmd: self.repo.get_setting("shell_path_cmd").ok().flatten(),
            bash: self.repo.get_setting("shell_path_bash").ok().flatten(),
            python: self.repo.get_setting("shell_path_python").ok().flatten(),
        }
    }

    fn load_shell_path_settings_ui(&self) {
        self.ui.set_settings_shell_powershell(
            self.repo
                .get_setting("shell_path_powershell")
                .ok()
                .flatten()
                .unwrap_or_default()
                .into(),
        );
        self.ui.set_settings_shell_cmd(
            self.repo
                .get_setting("shell_path_cmd")
                .ok()
                .flatten()
                .unwrap_or_default()
                .into(),
        );
        self.ui.set_settings_shell_bash(
            self.repo
                .get_setting("shell_path_bash")
                .ok()
                .flatten()
                .unwrap_or_default()
                .into(),
        );
        self.ui.set_settings_shell_python(
            self.repo
                .get_setting("shell_path_python")
                .ok()
                .flatten()
                .unwrap_or_default()
                .into(),
        );
    }

    fn save_shell_path_settings(&self) -> Result<()> {
        self.repo.set_setting(
            "shell_path_powershell",
            &self.ui.get_settings_shell_powershell().to_string(),
        )?;
        self.repo
            .set_setting("shell_path_cmd", &self.ui.get_settings_shell_cmd().to_string())?;
        self.repo
            .set_setting("shell_path_bash", &self.ui.get_settings_shell_bash().to_string())?;
        self.repo.set_setting(
            "shell_path_python",
            &self.ui.get_settings_shell_python().to_string(),
        )?;
        Ok(())
    }

    fn browse_script_file(&self) -> Result<()> {
        let file = rfd::FileDialog::new()
            .set_title(crate::i18n::rfd_pick_script())
            .add_filter(
                "Scripts",
                &[
                    "ps1", "bat", "cmd", "sh", "py", "pyw", "exe", "PS1", "BAT", "CMD", "SH",
                    "PY",
                ],
            )
            .pick_file();
        let Some(path) = file else {
            return Ok(());
        };
        self.ui
            .set_slot_editor_script(path.display().to_string().into());
        let shell = crate::script::infer_shell_from_path(&path);
        self.ui
            .set_slot_editor_script_shell(SharedString::from(shell.as_str()));
        Ok(())
    }

    fn restore_window_size(&self) -> Result<()> {
        self.apply_compact_window_size();
        Ok(())
    }

    fn save_window_size(&self) -> Result<()> {
        let window = self.ui.window();
        let scale = window.scale_factor();
        let size = window.size();
        let w = ((size.width as f32) / scale).round().max(480.0) as u32;
        self.repo.set_setting("window_width", &w.to_string())?;
        self.repo
            .set_setting("window_height", &COMPACT_WINDOW_H.to_string())?;
        Ok(())
    }

    fn start_alarm_poll(self: &Rc<Self>) {
        let weak = Rc::downgrade(self);
        let timer = Timer::default();
        timer.start(TimerMode::Repeated, Duration::from_secs(1), move || {
            let Some(app) = weak.upgrade() else {
                return;
            };
            app.poll_alarms();
        });
        *self.alarm_poll.borrow_mut() = Some(timer);
    }

    fn poll_alarms(&self) {
        self.alarm_tracker.borrow_mut().reset_if_new_day();
        let Ok(slots) = self.repo.list_alarm_slots() else {
            return;
        };
        for slot in slots {
            if is_timer_mode(&slot.alarm_mode) {
                if self.alarm_tracker.borrow_mut().should_fire_timer(&slot) {
                    self.trigger_alarm(&slot);
                    self.disarm_timer_slot(&slot);
                }
            } else if let Some(ref hm) = slot.alarm_time {
                if slot.alarm_armed
                    && self
                        .alarm_tracker
                        .borrow_mut()
                        .should_fire_clock(&slot, hm)
                {
                    self.trigger_alarm(&slot);
                }
            }
        }
    }

    fn trigger_alarm(&self, slot: &Slot) {
        let label = slot
            .label
            .clone()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Alarme".into());
        let time = if is_timer_mode(&slot.alarm_mode) {
            format!(
                "⏱ {} min",
                slot.alarm_minutes
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| "?".into())
            )
        } else {
            slot.alarm_time.clone().unwrap_or_default()
        };
        self.ui.set_alarm_popup_label(label.into());
        self.ui.set_alarm_popup_time(time.clone().into());
        self.ui.set_show_alarm_popup(true);
        self.ui.set_window_always_on_top(true);
        let _ = self.ui.window().show();

        if let Some(sid) = slot.sound_id {
            if let Ok(sound) = self.repo.get_sound(sid) {
                let _ = self.audio.lock().unwrap().play_file(
                    &sound.file_path,
                    sound.volume_linear,
                    sound.loudness_gain_db,
                    slot.slot_volume,
                );
            }
        }
        self.set_status(SharedString::from(format!("Alarme {time}")));
    }

    fn dismiss_alarm_popup(&self) {
        self.ui.set_show_alarm_popup(false);
        self.apply_always_on_top_setting();
        self.audio.lock().unwrap().stop_all();
    }

    fn always_on_top_enabled(&self) -> bool {
        self.repo
            .get_setting("always_on_top")
            .ok()
            .flatten()
            .map(|v| parse_bool_setting(&v))
            .unwrap_or(false)
    }

    fn apply_always_on_top_setting(&self) {
        self.apply_always_on_top(self.always_on_top_enabled());
    }

    fn apply_always_on_top(&self, enabled: bool) {
        self.ui.set_window_always_on_top(enabled);
    }

    fn test_alarm_from_editor(&self) -> Result<()> {
        let Some((row, col)) = *self.editing_slot.borrow() else {
            return Ok(());
        };
        let page_id = self.nav.borrow().current();
        let label = self.ui.get_slot_editor_label().to_string();
        let mode = self.ui.get_slot_editor_alarm_mode().to_string();
        let (alarm_time, alarm_minutes) = if is_timer_mode(&mode) {
            let Some(mins) = self.parse_editor_alarm_minutes() else {
                self.set_status(SharedString::from("Durée invalide"));
                return Ok(());
            };
            (None, Some(mins))
        } else {
            let h = self.ui.get_slot_editor_alarm_hour_index();
            let m = self.ui.get_slot_editor_alarm_minute_index();
            let Some(hm) = alarm_time_from_indices(h, m) else {
                self.set_status(SharedString::from("Heure invalide"));
                return Ok(());
            };
            (Some(hm), None)
        };
        let sound_id = self.editor_sound_id()?;
        let slot = Slot {
            page_id,
            row,
            col,
            kind: SlotKind::Alarm,
            label: if label.is_empty() {
                Some("Test alarme".into())
            } else {
                Some(label)
            },
            image_path: None,
            slot_appearance: "color".to_string(),
            sound_id,
            child_page_id: None,
            slot_volume: 1.0,
            color_hex: None,
            script_command: None,
            script_shell: None,
            alarm_time,
            alarm_mode: mode,
            alarm_minutes,
            alarm_armed: false,
            alarm_armed_at_ms: None,
            shortcut_key: None,
        };
        self.trigger_alarm(&slot);
        Ok(())
    }
}

fn format_mm_ss(ms: u64) -> String {
    let total_sec = ms / 1000;
    format!("{:02}:{:02}", total_sec / 60, total_sec % 60)
}

fn parse_bool_setting(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn default_script_shell() -> &'static str {
    crate::script::ScriptShell::default_for_platform().as_str()
}

fn normalize_color_hex(input: &str) -> String {
    let t = input.trim();
    if t.is_empty() {
        return DEFAULT_SLOT_COLOR.into();
    }
    if t.starts_with('#') && t.len() == 7 {
        return t.to_ascii_lowercase();
    }
    if t.len() == 6 && t.chars().all(|c| c.is_ascii_hexdigit()) {
        return format!("#{t}");
    }
    DEFAULT_SLOT_COLOR.into()
}

#[cfg(windows)]
fn apply_windows_rounded_corners(window: &slint::Window) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    let _ = window.with_winit_window(|winit_win| {
        let Ok(handle) = winit_win.window_handle() else {
            return;
        };
        let RawWindowHandle::Win32(win32) = handle.as_raw() else {
            return;
        };
        let hwnd = win32.hwnd.get() as *mut core::ffi::c_void;
        unsafe {
            const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
            const DWMWCP_ROUND: u32 = 2;
            let preference = DWMWCP_ROUND;
            let _ = windows_sys::Win32::Graphics::Dwm::DwmSetWindowAttribute(
                hwnd,
                DWMWA_WINDOW_CORNER_PREFERENCE,
                &preference as *const u32 as *const _,
                core::mem::size_of::<u32>() as u32,
            );
        }
    });
}

fn wire_window_chrome(app: Rc<Application>) {
    let a = app.clone();
    app.ui.on_window_quit(move || {
        let _ = a.ui.window().hide();
        slint::quit_event_loop().ok();
    });

    let a = app.clone();
    app.ui.on_window_drag(move || {
        let win = a.ui.window();
        let _ = win.with_winit_window(|winit_win| winit_win.drag_window());
    });
}
