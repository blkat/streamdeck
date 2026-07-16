use crate::db::Slot;

use chrono::{Local, Timelike};

use std::collections::HashMap;

use std::time::{SystemTime, UNIX_EPOCH};



pub fn parse_alarm_hm(raw: &str) -> Option<(u32, u32)> {

    let t = raw.trim();

    if t.is_empty() {

        return None;

    }

    let mut parts = t.split(':');

    let h: u32 = parts.next()?.trim().parse().ok()?;

    let m: u32 = parts.next().unwrap_or("0").trim().parse().ok()?;

    if h < 24 && m < 60 {

        Some((h, m))

    } else {

        None

    }

}



pub fn normalize_alarm_time(raw: &str) -> Option<String> {

    let (h, m) = parse_alarm_hm(raw)?;

    Some(format!("{h:02}:{m:02}"))

}



pub fn alarm_time_from_indices(hour: i32, minute: i32) -> Option<String> {

    if !(0..24).contains(&hour) || !(0..60).contains(&minute) {

        return None;

    }

    Some(format!("{hour:02}:{minute:02}"))

}



pub fn indices_from_alarm_time(hm: &str) -> (i32, i32) {

    parse_alarm_hm(hm).map(|(h, m)| (h as i32, m as i32)).unwrap_or((12, 0))

}



pub fn is_timer_mode(mode: &str) -> bool {

    mode == "timer"

}



pub fn now_matches_alarm(alarm_hm: &str) -> bool {

    let Some((ah, am)) = parse_alarm_hm(alarm_hm) else {

        return false;

    };

    let now = Local::now();

    now.hour() == ah && now.minute() == am

}



pub fn now_ms() -> i64 {

    SystemTime::now()

        .duration_since(UNIX_EPOCH)

        .map(|d| d.as_millis() as i64)

        .unwrap_or(0)

}



pub fn timer_deadline_reached(slot: &Slot) -> bool {

    if !is_timer_mode(&slot.alarm_mode) || !slot.alarm_armed {

        return false;

    }

    let Some(armed_at) = slot.alarm_armed_at_ms else {

        return false;

    };

    let Some(mins) = slot.alarm_minutes.filter(|m| *m > 0) else {

        return false;

    };

    let deadline = armed_at + i64::from(mins) * 60_000;

    now_ms() >= deadline

}



pub fn slot_alarm_key(slot: &Slot) -> String {

    format!("{}:{}:{}", slot.page_id, slot.row, slot.col)

}



pub struct AlarmFireTracker {

    fired_today: HashMap<String, String>,

    fired_timer: HashMap<String, i64>,

}



impl AlarmFireTracker {

    pub fn new() -> Self {

        Self {

            fired_today: HashMap::new(),

            fired_timer: HashMap::new(),

        }

    }



    pub fn should_fire_clock(&mut self, slot: &Slot, alarm_hm: &str) -> bool {

        if !now_matches_alarm(alarm_hm) {

            return false;

        }

        let today = Local::now().format("%Y-%m-%d").to_string();

        let key = slot_alarm_key(slot);

        if self.fired_today.get(&key) == Some(&today) {

            return false;

        }

        self.fired_today.insert(key, today);

        true

    }



    pub fn should_fire_timer(&mut self, slot: &Slot) -> bool {

        if !timer_deadline_reached(slot) {

            return false;

        }

        let key = slot_alarm_key(slot);

        let armed_at = slot.alarm_armed_at_ms.unwrap_or(0);

        if self.fired_timer.get(&key) == Some(&armed_at) {

            return false;

        }

        self.fired_timer.insert(key, armed_at);

        true

    }



    pub fn reset_if_new_day(&mut self) {

        let today = Local::now().format("%Y-%m-%d").to_string();

        self.fired_today.retain(|_, d| d == &today);

    }

}



#[cfg(test)]

mod tests {

    use super::*;

    use crate::db::{Slot, SlotKind};



    #[test]

    fn timer_deadline() {

        let mut slot = Slot {

            page_id: 1,

            row: 0,

            col: 0,

            kind: SlotKind::Alarm,

            label: None,

            image_path: None,
            slot_appearance: "color".to_string(),

            sound_id: None,

            child_page_id: None,

            slot_volume: 1.0,

            color_hex: None,

            script_command: None,
            script_shell: None,

            alarm_time: None,

            alarm_mode: "timer".into(),

            alarm_minutes: Some(5),

            alarm_armed: true,

            alarm_armed_at_ms: Some(now_ms() - 6 * 60_000),

            shortcut_key: None,

        };

        assert!(timer_deadline_reached(&slot));

        slot.alarm_armed_at_ms = Some(now_ms());

        assert!(!timer_deadline_reached(&slot));

    }

}


