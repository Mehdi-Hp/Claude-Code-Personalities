use crate::kaomoji::Kaomoji;
use chrono::{DateTime, Datelike, Local, Timelike};

// Night Owl (Midnight - 5 AM)
pub const NIGHT_OWL: Kaomoji = Kaomoji::new("(ʘ,ʘ)", "Night Owl");

// Caffeinated (5 AM - 8 AM)
pub const CAFFEINATED: Kaomoji = Kaomoji::new("( -_-)旦~", "Caffeinated");

// TGIFFFFF (Friday > 5 PM)
pub const TGIFFFFF: Kaomoji = Kaomoji::new("ヽ(⌐■_■)ノ♪♬", "TGIFFFFF");

/// Get time-based kaomoji if applicable
pub fn get_time_kaomoji() -> Option<&'static Kaomoji> {
    get_time_kaomoji_for(Local::now())
}

/// Get time-based kaomoji for a specific time (used for testing)
pub fn get_time_kaomoji_for(now: DateTime<Local>) -> Option<&'static Kaomoji> {
    let hour = now.hour();
    let weekday = now.weekday();

    // Friday evenings (after 5 PM)
    if weekday == chrono::Weekday::Fri && hour >= 17 {
        return Some(&TGIFFFFF);
    }

    // Late night (Midnight - 5 AM)
    if hour < 5 {
        return Some(&NIGHT_OWL);
    }

    // Early morning (5 AM - 8 AM)
    if hour >= 5 && hour < 8 {
        return Some(&CAFFEINATED);
    }

    None
}
