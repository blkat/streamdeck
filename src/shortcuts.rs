use crate::db::Slot;
use winit::keyboard::{KeyCode, NamedKey, PhysicalKey};

pub const GRID_ROWS: i32 = 3;
pub const GRID_COLS: i32 = 5;

const DEFAULT_KEYS: [[KeyCode; 5]; 3] = [
    [
        KeyCode::KeyQ,
        KeyCode::KeyW,
        KeyCode::KeyE,
        KeyCode::KeyR,
        KeyCode::KeyT,
    ],
    [
        KeyCode::KeyA,
        KeyCode::KeyS,
        KeyCode::KeyD,
        KeyCode::KeyF,
        KeyCode::KeyG,
    ],
    [
        KeyCode::KeyZ,
        KeyCode::KeyX,
        KeyCode::KeyC,
        KeyCode::KeyV,
        KeyCode::KeyB,
    ],
];

const DEFAULT_LABELS: [[&str; 5]; 3] = [
    ["A", "Z", "E", "R", "T"],
    ["Q", "S", "D", "F", "G"],
    ["W", "X", "C", "V", "B"],
];

pub fn default_key_for_cell(row: i32, col: i32) -> Option<KeyCode> {
    if !(0..GRID_ROWS).contains(&row) || !(0..GRID_COLS).contains(&col) {
        return None;
    }
    Some(DEFAULT_KEYS[row as usize][col as usize])
}

pub fn default_label_for_cell(row: i32, col: i32) -> &'static str {
    if !(0..GRID_ROWS).contains(&row) || !(0..GRID_COLS).contains(&col) {
        return "";
    }
    DEFAULT_LABELS[row as usize][col as usize]
}

pub fn format_key_code(code: KeyCode) -> String {
    format!("{code:?}")
}

pub fn parse_key_code(s: &str) -> Option<KeyCode> {
    let trimmed = s.trim();
    if trimmed.is_empty() {
        return None;
    }
    match trimmed {
        "KeyA" => Some(KeyCode::KeyA),
        "KeyB" => Some(KeyCode::KeyB),
        "KeyC" => Some(KeyCode::KeyC),
        "KeyD" => Some(KeyCode::KeyD),
        "KeyE" => Some(KeyCode::KeyE),
        "KeyF" => Some(KeyCode::KeyF),
        "KeyG" => Some(KeyCode::KeyG),
        "KeyH" => Some(KeyCode::KeyH),
        "KeyI" => Some(KeyCode::KeyI),
        "KeyJ" => Some(KeyCode::KeyJ),
        "KeyK" => Some(KeyCode::KeyK),
        "KeyL" => Some(KeyCode::KeyL),
        "KeyM" => Some(KeyCode::KeyM),
        "KeyN" => Some(KeyCode::KeyN),
        "KeyO" => Some(KeyCode::KeyO),
        "KeyP" => Some(KeyCode::KeyP),
        "KeyQ" => Some(KeyCode::KeyQ),
        "KeyR" => Some(KeyCode::KeyR),
        "KeyS" => Some(KeyCode::KeyS),
        "KeyT" => Some(KeyCode::KeyT),
        "KeyU" => Some(KeyCode::KeyU),
        "KeyV" => Some(KeyCode::KeyV),
        "KeyW" => Some(KeyCode::KeyW),
        "KeyX" => Some(KeyCode::KeyX),
        "KeyY" => Some(KeyCode::KeyY),
        "KeyZ" => Some(KeyCode::KeyZ),
        "Digit0" => Some(KeyCode::Digit0),
        "Digit1" => Some(KeyCode::Digit1),
        "Digit2" => Some(KeyCode::Digit2),
        "Digit3" => Some(KeyCode::Digit3),
        "Digit4" => Some(KeyCode::Digit4),
        "Digit5" => Some(KeyCode::Digit5),
        "Digit6" => Some(KeyCode::Digit6),
        "Digit7" => Some(KeyCode::Digit7),
        "Digit8" => Some(KeyCode::Digit8),
        "Digit9" => Some(KeyCode::Digit9),
        "F1" => Some(KeyCode::F1),
        "F2" => Some(KeyCode::F2),
        "F3" => Some(KeyCode::F3),
        "F4" => Some(KeyCode::F4),
        "F5" => Some(KeyCode::F5),
        "F6" => Some(KeyCode::F6),
        "F7" => Some(KeyCode::F7),
        "F8" => Some(KeyCode::F8),
        "F9" => Some(KeyCode::F9),
        "F10" => Some(KeyCode::F10),
        "F11" => Some(KeyCode::F11),
        "F12" => Some(KeyCode::F12),
        "Space" => Some(KeyCode::Space),
        _ => None,
    }
}

pub fn label_for_key_code(code: KeyCode) -> String {
    for row in 0..GRID_ROWS {
        for col in 0..GRID_COLS {
            if DEFAULT_KEYS[row as usize][col as usize] == code {
                return DEFAULT_LABELS[row as usize][col as usize].to_string();
            }
        }
    }
    format_key_code(code)
        .strip_prefix("Key")
        .map(str::to_string)
        .or_else(|| {
            format_key_code(code)
                .strip_prefix("Digit")
                .map(str::to_string)
        })
        .unwrap_or_else(|| format_key_code(code))
}

pub fn effective_key_code(slot: Option<&Slot>, row: i32, col: i32) -> String {
    if let Some(s) = slot {
        if let Some(ref custom) = s.shortcut_key {
            if !custom.trim().is_empty() {
                return custom.clone();
            }
        }
    }
    default_key_for_cell(row, col)
        .map(format_key_code)
        .unwrap_or_default()
}

pub fn effective_label(slot: Option<&Slot>, row: i32, col: i32) -> String {
    if let Some(s) = slot {
        if let Some(ref custom) = s.shortcut_key {
            if let Some(code) = parse_key_code(custom) {
                return label_for_key_code(code);
            }
            if custom == "Space" {
                return "Space".into();
            }
        }
    }
    default_label_for_cell(row, col).to_string()
}

pub fn shortcut_label_for_cell(row: i32, col: i32) -> String {
    default_label_for_cell(row, col).to_string()
}

pub fn resolve_cell(slots: &[Slot], code: KeyCode) -> Option<(i32, i32)> {
    let key_str = format_key_code(code);
    for slot in slots {
        if effective_key_code(Some(slot), slot.row, slot.col) == key_str {
            return Some((slot.row, slot.col));
        }
    }
    for row in 0..GRID_ROWS {
        for col in 0..GRID_COLS {
            if slots.iter().any(|s| s.row == row && s.col == col) {
                continue;
            }
            if default_key_for_cell(row, col) == Some(code) {
                return Some((row, col));
            }
        }
    }
    None
}

pub fn physical_key_to_code(key: PhysicalKey) -> Option<KeyCode> {
    match key {
        PhysicalKey::Code(code) => Some(code),
        PhysicalKey::Unidentified(_) => None,
    }
}

pub fn named_key_label(key: NamedKey) -> Option<String> {
    match key {
        NamedKey::Space => Some("Space".into()),
        NamedKey::F1 => Some("F1".into()),
        NamedKey::F2 => Some("F2".into()),
        NamedKey::F3 => Some("F3".into()),
        NamedKey::F4 => Some("F4".into()),
        NamedKey::F5 => Some("F5".into()),
        NamedKey::F6 => Some("F6".into()),
        NamedKey::F7 => Some("F7".into()),
        NamedKey::F8 => Some("F8".into()),
        NamedKey::F9 => Some("F9".into()),
        NamedKey::F10 => Some("F10".into()),
        NamedKey::F11 => Some("F11".into()),
        NamedKey::F12 => Some("F12".into()),
        _ => None,
    }
}

pub fn key_event_to_stored_code(
    physical: PhysicalKey,
    logical: &winit::keyboard::Key,
) -> Option<String> {
    if let Some(code) = physical_key_to_code(physical) {
        return Some(format_key_code(code));
    }
    if let winit::keyboard::Key::Named(named) = logical {
        return named_key_label(*named);
    }
    None
}

#[derive(Debug, Clone)]
pub struct ShortcutConflict {
    pub row: i32,
    pub col: i32,
}

impl ShortcutConflict {
    pub fn message(&self) -> String {
        crate::i18n::shortcut_already_used(&format!("({}, {})", self.row + 1, self.col + 1))
    }
}

pub fn find_conflict_on_page(
    slots: &[Slot],
    exclude_row: i32,
    exclude_col: i32,
    key_code: &str,
) -> Option<ShortcutConflict> {
    for slot in slots {
        if slot.row == exclude_row && slot.col == exclude_col {
            continue;
        }
        if effective_key_code(Some(slot), slot.row, slot.col) == key_code {
            return Some(ShortcutConflict {
                row: slot.row,
                col: slot.col,
            });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{Slot, SlotKind};

    fn empty_slot(row: i32, col: i32) -> Slot {
        Slot {
            page_id: 1,
            row,
            col,
            kind: SlotKind::Empty,
            label: None,
            image_path: None,
            slot_appearance: "color".into(),
            sound_id: None,
            child_page_id: None,
            slot_volume: 1.0,
            color_hex: None,
            script_command: None,
            script_shell: None,
            alarm_time: None,
            alarm_mode: "clock".into(),
            alarm_minutes: None,
            alarm_armed: false,
            alarm_armed_at_ms: None,
            shortcut_key: None,
        }
    }

    #[test]
    fn default_mapping_fifteen_cells() {
        assert_eq!(default_label_for_cell(0, 0), "A");
        assert_eq!(default_label_for_cell(2, 4), "B");
        assert_eq!(format_key_code(default_key_for_cell(1, 2).unwrap()), "KeyD");
    }

    #[test]
    fn parse_round_trip() {
        let code = KeyCode::KeyW;
        let s = format_key_code(code);
        assert_eq!(parse_key_code(&s), Some(code));
    }

    #[test]
    fn resolve_custom_over_default() {
        let mut slot = empty_slot(0, 0);
        slot.shortcut_key = Some("KeyF".into());
        let resolved = resolve_cell(&[slot], KeyCode::KeyF);
        assert_eq!(resolved, Some((0, 0)));
    }

    #[test]
    fn conflict_detection() {
        let mut a = empty_slot(0, 0);
        a.shortcut_key = Some("KeyF".into());
        let b = empty_slot(0, 1);
        let conflict = find_conflict_on_page(&[a, b], 0, 1, "KeyF");
        assert!(conflict.is_some());
    }
}
