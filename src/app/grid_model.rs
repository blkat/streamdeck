use crate::alarm::is_timer_mode;
use crate::color::brush_from_hex_or_default;
use crate::db::{Slot, SlotKind};
use crate::icons::slot_display_icon;
use crate::slot_image::{is_image_appearance, load_slot_photo_at_base};
use crate::navigation::{is_back_slot, is_home_slot, is_reserved_slot};
use crate::shortcuts::{effective_label, shortcut_label_for_cell};
use crate::SlotViewModel;
use slint::Image;
use std::path::Path;

const DEFAULT_BTN: (u8, u8, u8) = (0xff, 0x98, 0x00);
const EMPTY_BTN: (u8, u8, u8) = (0x1a, 0x1a, 0x1a);

pub const DEFAULT_SLOT_COLOR: &str = "#ff9800";
pub const NEUTRAL_SLOT_COLOR: &str = "#1a1a1a";
pub const DEFAULT_SLOT_ICON: &str = "aide";

pub fn default_color_for_kind(kind: SlotKind) -> &'static str {
    match kind {
        SlotKind::Sound | SlotKind::Script | SlotKind::Alarm => NEUTRAL_SLOT_COLOR,
        SlotKind::Folder => "#1565c0",
        SlotKind::Empty => NEUTRAL_SLOT_COLOR,
    }
}

pub fn build_grid_slots(
    base: &Path,
    db_slots: &[Slot],
    at_root: bool,
) -> Vec<SlotViewModel> {
    let mut models = Vec::with_capacity(15);
    for row in 0..3 {
        for col in 0..5 {
            if is_home_slot(row, col) {
                models.push(SlotViewModel {
                    row,
                    col,
                    label: "".into(),
                    kind: "home".into(),
                    kind_icon: "".into(),
                    bg_color: brush_from_hex_or_default(Some("#2ecc71"), (0x2e, 0xcc, 0x71)),
                    is_system: true,
                    use_slot_photo: false,
                    slot_photo: Image::default(),
                    show_slot_icon: false,
                    slot_icon: Image::default(),
                    alarm_armed: false,
                    shortcut_label: shortcut_label_for_cell(row, col).into(),
                });
                continue;
            }
            if !at_root && is_back_slot(row, col) {
                models.push(SlotViewModel {
                    row,
                    col,
                    label: "".into(),
                    kind: "back".into(),
                    kind_icon: "".into(),
                    bg_color: brush_from_hex_or_default(Some("#1565c0"), (0x15, 0x65, 0xc0)),
                    is_system: true,
                    use_slot_photo: false,
                    slot_photo: Image::default(),
                    show_slot_icon: false,
                    slot_icon: Image::default(),
                    alarm_armed: false,
                    shortcut_label: shortcut_label_for_cell(row, col).into(),
                });
                continue;
            }
            if is_reserved_slot(row, col, at_root) {
                continue;
            }

            let slot = db_slots.iter().find(|s| s.row == row && s.col == col);

            let (label, kind, kind_icon, bg, use_photo, photo, show_icon, icon_img, alarm_armed) =
                match slot {
                Some(s) => {
                    let label = s
                        .label
                        .clone()
                        .unwrap_or_else(|| default_label(s));
                    let default_hex = default_color_for_kind(s.kind);
                    let hex: &str = if s.kind == SlotKind::Alarm && s.alarm_armed {
                        "#00c853"
                    } else {
                        s.color_hex.as_deref().unwrap_or(default_hex)
                    };
                    let armed = s.kind == SlotKind::Alarm && s.alarm_armed;
                    if s.kind == SlotKind::Empty {
                        let bg = brush_from_hex_or_default(Some(NEUTRAL_SLOT_COLOR), EMPTY_BTN);
                        (
                            label,
                            kind_str(s),
                            grid_kind_icon(s),
                            bg,
                            false,
                            Image::default(),
                            false,
                            Image::default(),
                            false,
                        )
                    } else if is_image_appearance(&s.slot_appearance) {
                        let photo = s
                            .image_path
                            .as_ref()
                            .and_then(|f| load_slot_photo_at_base(base, f))
                            .unwrap_or_default();
                        let has_photo = s.image_path.as_ref().is_some_and(|f| !f.is_empty());
                        let bg = brush_from_hex_or_default(
                            if armed { Some("#00c853") } else { Some("#0d0d0d") },
                            EMPTY_BTN,
                        );
                        (
                            label,
                            kind_str(s),
                            grid_kind_icon(s),
                            bg,
                            has_photo,
                            photo,
                            false,
                            Image::default(),
                            armed,
                        )
                    } else {
                        let bg = brush_from_hex_or_default(Some(hex), DEFAULT_BTN);
                        let icon_img =
                            slot_display_icon(base, &s.slot_appearance, &s.image_path)
                                .unwrap_or_default();
                        let show_icon = s
                            .image_path
                            .as_ref()
                            .is_some_and(|id| !id.trim().is_empty());
                        (
                            label,
                            kind_str(s),
                            grid_kind_icon(s),
                            bg,
                            false,
                            Image::default(),
                            show_icon,
                            icon_img,
                            armed,
                        )
                    }
                }
                None => (
                    "".into(),
                    "empty".into(),
                    "".into(),
                    brush_from_hex_or_default(None, EMPTY_BTN),
                    false,
                    Image::default(),
                    false,
                    Image::default(),
                    false,
                ),
            };

            models.push(SlotViewModel {
                row,
                col,
                label: label.into(),
                kind: kind.into(),
                kind_icon: kind_icon.into(),
                bg_color: bg,
                is_system: false,
                use_slot_photo: use_photo,
                slot_photo: photo,
                show_slot_icon: show_icon,
                slot_icon: icon_img,
                alarm_armed,
                shortcut_label: effective_label(slot, row, col).into(),
            });
        }
    }
    models
}

fn grid_kind_icon(s: &Slot) -> &'static str {
    match s.kind {
        SlotKind::Folder => "📁",
        SlotKind::Sound => "♪",
        SlotKind::Script => "▶",
        SlotKind::Alarm => {
            if s.alarm_armed {
                "▶"
            } else if is_timer_mode(&s.alarm_mode) {
                "⏱"
            } else {
                "⏰"
            }
        }
        SlotKind::Empty => "",
    }
}

fn kind_str(s: &Slot) -> &'static str {
    match s.kind {
        SlotKind::Sound => "sound",
        SlotKind::Folder => "folder",
        SlotKind::Script => "script",
        SlotKind::Alarm => "alarm",
        SlotKind::Empty => "empty",
    }
}

fn default_label(s: &Slot) -> String {
    match s.kind {
        SlotKind::Folder => s
            .label
            .clone()
            .filter(|l| !l.is_empty())
            .unwrap_or_else(|| crate::i18n::default_kind_label("folder")),
        SlotKind::Sound => s
            .label
            .clone()
            .filter(|l| !l.is_empty())
            .unwrap_or_else(|| crate::i18n::default_kind_label("sound")),
        SlotKind::Script => s
            .label
            .clone()
            .filter(|l| !l.is_empty())
            .unwrap_or_else(|| crate::i18n::default_kind_label("script")),
        SlotKind::Alarm => {
            if is_timer_mode(&s.alarm_mode) {
                let mins = s.alarm_minutes.unwrap_or(0);
                let prefix = if s.alarm_armed { "▶" } else { "⏱" };
                if let Some(lbl) = s.label.clone().filter(|l| !l.is_empty()) {
                    format!("{prefix} {lbl}")
                } else {
                    format!("{prefix} {mins} min")
                }
            } else if let Some(lbl) = s.label.clone().filter(|l| !l.is_empty()) {
                if s.alarm_armed {
                    format!("▶ {lbl}")
                } else {
                    lbl
                }
            } else if s.alarm_armed {
                s.alarm_time
                    .as_deref()
                    .map(|t| format!("▶ {t}"))
                    .unwrap_or_else(|| format!("▶ {}", crate::i18n::alarm_default_label()))
            } else {
                s.alarm_time
                    .as_deref()
                    .map(|t| format!("⏰ {t}"))
                    .unwrap_or_else(|| crate::i18n::alarm_default_label())
            }
        }
        SlotKind::Empty => String::new(),
    }
}

pub fn hex_for_slot_editor(slot: Option<&Slot>, kind: &str) -> String {
    if let Some(s) = slot {
        if let Some(ref c) = s.color_hex {
            return c.clone();
        }
        return default_color_for_kind(s.kind).to_string();
    }
    match kind {
        "sound" | "script" | "alarm" | "empty" => NEUTRAL_SLOT_COLOR.into(),
        "folder" => "#1565c0".into(),
        _ => DEFAULT_SLOT_COLOR.into(),
    }
}
