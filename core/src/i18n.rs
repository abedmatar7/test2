use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Centralized localization catalog so additional languages can be added without modifying UI code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiStrings {
    pub locale: String,
    pub entries: HashMap<String, String>,
}

impl Default for UiStrings {
    fn default() -> Self {
        let mut entries = HashMap::new();
        let defaults = [
            ("app.title", "MedPulse Offline Study"),
            ("app.subtitle", "OLED Focus Timer & Clinical Mastery"),
            ("nav.timer", "Focus & OLED"),
            ("nav.flashcards", "Flashcards"),
            ("nav.qbank", "Q-Bank"),
            ("nav.cases", "Cases & OSCE"),
            ("nav.notes", "Feynman Notes"),
            ("nav.dashboard", "Dashboard"),
            ("timer.preset.25_5", "Pomodoro 25 / 5"),
            ("timer.preset.50_10", "Deep Block 50 / 10"),
            ("timer.preset.custom", "Custom Block"),
            ("timer.start", "Start Focus Session"),
            ("timer.ambient.enter", "Enter Pure-Black OLED Ambient Mode"),
            ("timer.ambient.tap_hint", "Tap anywhere to reveal controls"),
            ("timer.ambient.pause", "Pause"),
            ("timer.ambient.resume", "Resume"),
            ("timer.ambient.stop", "Finish & Log"),
            ("timer.ambient.exit", "Exit Ambient"),
            ("timer.aod.disclaimer", "In-app OLED Ambient Mode keeps the display awake at ultra-low luminance with 60s pixel shifting. If you lock the screen with the hardware power button, Android's Foreground Service continues timing and triggers a high-priority alarm notification at completion."),
            ("motivation.pre_session_header", "Before You Begin"),
            ("motivation.favorite_toggle", "Save to Favorite Quotes"),
            ("motivation.post_session_header", "Why This Matters at the Bedside"),
            ("motivation.streak_rest_day_badge", "Rest-Day Grace Protected (1 recovery day/week preserves streak)"),
            ("motivation.sleep_reminder", "Consolidation Alert: Synaptic replay of today's pathology & clinical schemas peaks during slow-wave and REM sleep. Begin wind-down soon."),
            ("srs.show_answer", "Reveal Answer"),
            ("srs.grade.again", "Again (<10m)"),
            ("srs.grade.hard", "Hard"),
            ("srs.grade.good", "Good"),
            ("srs.grade.easy", "Easy"),
            ("notes.feynman.prompt", "Explain this mechanism in plain bedside language (no jargon):"),
            ("notes.mnemonic.prompt", "Personal Mnemonic / Diagnostic Key:"),
            ("backup.export", "Export Offline JSON Backup"),
            ("backup.import", "Import Offline JSON Backup"),
        ];
        for (k, v) in defaults {
            entries.insert(k.to_string(), v.to_string());
        }
        Self {
            locale: "en".to_string(),
            entries,
        }
    }
}

impl UiStrings {
    pub fn from_json_str(raw: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(raw)
    }

    pub fn get<'a>(&'a self, key: &'a str) -> &'a str {
        self.entries.get(key).map(|s| s.as_str()).unwrap_or(key)
    }
}
