pub fn db_to_linear(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}

pub fn effective_volume(global: f32, sound_vol: f32, loudness_gain_db: f32, slot_vol: f32) -> f32 {
    (global * sound_vol * slot_vol * db_to_linear(loudness_gain_db)).clamp(0.0, 2.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_volume_calc() {
        let v = effective_volume(0.7, 1.0, 0.0, 1.0);
        assert!((v - 0.7).abs() < 0.001);
    }
}
