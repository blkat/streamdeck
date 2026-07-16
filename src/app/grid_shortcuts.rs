use crate::shortcuts::{
    effective_label, find_conflict_on_page, key_event_to_stored_code, parse_key_code,
    resolve_cell, ShortcutConflict,
};
use slint::winit_030::{EventResult, WinitWindowAccessor};
use slint::ComponentHandle;
use std::rc::{Rc, Weak};
use winit::event::{ElementState, KeyEvent, WindowEvent};

use super::Application;

pub fn wire_grid_shortcuts(app: Rc<Application>) {
    let weak: Weak<Application> = Rc::downgrade(&app);
    app.ui.window().on_winit_window_event(move |_slint_window, event| {
        let Some(app) = weak.upgrade() else {
            return EventResult::Propagate;
        };
        if app.handle_grid_key_event(event) {
            return EventResult::PreventDefault;
        }
        EventResult::Propagate
    });
}

impl Application {
    pub fn load_grid_shortcuts_setting(&self) {
        let enabled = self
            .repo
            .get_setting("grid_shortcuts_enabled")
            .ok()
            .flatten()
            .map(|v| parse_bool_setting(&v))
            .unwrap_or(true);
        *self.grid_shortcuts_enabled.borrow_mut() = enabled;
        self.ui.set_settings_grid_shortcuts_enabled(enabled);
    }

    pub fn sync_slot_editor_shortcut_ui(
        &self,
        row: i32,
        col: i32,
        slot: Option<&crate::db::Slot>,
    ) {
        let label = effective_label(slot, row, col);
        self.ui.set_slot_editor_shortcut_label(label.into());
        self.ui.set_slot_editor_shortcut_error("".into());
        self.ui.set_slot_editor_shortcut_capturing(false);
        *self.editor_shortcut_use_default.borrow_mut() = slot
            .and_then(|s| s.shortcut_key.as_ref())
            .is_none();
        *self.editor_shortcut_override.borrow_mut() =
            slot.and_then(|s| s.shortcut_key.clone());
    }

    pub fn handle_grid_key_event(self: &Rc<Self>, event: &WindowEvent) -> bool {
        if let WindowEvent::ModifiersChanged(modifiers) = event {
            *self.modifiers_down.borrow_mut() = modifiers.state();
            return false;
        }

        let WindowEvent::KeyboardInput {
            event:
                KeyEvent {
                    physical_key,
                    logical_key,
                    state,
                    repeat,
                    ..
                },
            ..
        } = event
        else {
            return false;
        };

        if *state != ElementState::Pressed || *repeat {
            return false;
        }

        if self.modifiers_down.borrow().intersects(
            winit::keyboard::ModifiersState::CONTROL
                | winit::keyboard::ModifiersState::ALT
                | winit::keyboard::ModifiersState::SUPER,
        ) {
            return false;
        }

        let Some(stored) = key_event_to_stored_code(*physical_key, logical_key) else {
            return false;
        };

        if *self.shortcut_capture_active.borrow() {
            return self.handle_shortcut_capture(&stored);
        }

        if self.grid_shortcuts_blocked() {
            return false;
        }

        let page_id = self.nav.borrow().current();
        let Ok(slots) = self.repo.list_slots(page_id) else {
            return false;
        };
        let Some(code) = parse_key_code(&stored) else {
            return false;
        };
        let Some((row, col)) = resolve_cell(&slots, code) else {
            return false;
        };

        let _ = self.handle_slot_click(row, col);
        true
    }

    fn handle_shortcut_capture(&self, stored: &str) -> bool {
        let Some((row, col)) = *self.editing_slot.borrow() else {
            *self.shortcut_capture_active.borrow_mut() = false;
            self.ui.set_slot_editor_shortcut_capturing(false);
            return true;
        };
        let page_id = self.nav.borrow().current();
        let slots = self.repo.list_slots(page_id).ok().unwrap_or_default();
        if let Some(conflict) = find_conflict_on_page(&slots, row, col, stored) {
            self.ui
                .set_slot_editor_shortcut_error(conflict.message().into());
        } else {
            *self.editor_shortcut_override.borrow_mut() = Some(stored.to_string());
            *self.editor_shortcut_use_default.borrow_mut() = false;
            let label = if let Some(code) = parse_key_code(stored) {
                crate::shortcuts::label_for_key_code(code)
            } else {
                stored.to_string()
            };
            self.ui.set_slot_editor_shortcut_label(label.into());
            self.ui.set_slot_editor_shortcut_error("".into());
        }
        *self.shortcut_capture_active.borrow_mut() = false;
        self.ui.set_slot_editor_shortcut_capturing(false);
        true
    }

    fn grid_shortcuts_blocked(&self) -> bool {
        if *self.edit_mode.borrow() {
            return true;
        }
        if !*self.grid_shortcuts_enabled.borrow() {
            return true;
        }
        if self.ui.get_show_slot_editor() {
            return true;
        }
        if self.ui.get_show_slot_color_picker() {
            return true;
        }
        if self.top_utility_modal().is_some() {
            return true;
        }
        false
    }

    pub fn start_shortcut_capture(self: &Rc<Self>) {
        *self.shortcut_capture_active.borrow_mut() = true;
        self.ui.set_slot_editor_shortcut_capturing(true);
        self.ui.set_slot_editor_shortcut_error("".into());
    }

    pub fn reset_editor_shortcut_to_default(&self, row: i32, col: i32) {
        *self.editor_shortcut_use_default.borrow_mut() = true;
        *self.editor_shortcut_override.borrow_mut() = None;
        self.ui.set_slot_editor_shortcut_label(
            crate::shortcuts::shortcut_label_for_cell(row, col).into(),
        );
        self.ui.set_slot_editor_shortcut_error("".into());
    }

    pub fn editor_shortcut_key_for_save(
        &self,
        row: i32,
        col: i32,
        existing: Option<&crate::db::Slot>,
    ) -> Result<Option<String>, ShortcutConflict> {
        if *self.editor_shortcut_use_default.borrow() {
            return Ok(None);
        }
        if let Some(ref key) = *self.editor_shortcut_override.borrow() {
            let page_id = self.nav.borrow().current();
            let slots = self.repo.list_slots(page_id).unwrap_or_default();
            if let Some(conflict) = find_conflict_on_page(&slots, row, col, key) {
                return Err(conflict);
            }
            return Ok(Some(key.clone()));
        }
        Ok(existing.and_then(|s| s.shortcut_key.clone()))
    }
}

fn parse_bool_setting(value: &str) -> bool {
    matches!(
        value.trim().to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}
