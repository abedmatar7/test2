use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimerPreset {
    Pomodoro25_5,
    Deep50_10,
    Custom { focus_minutes: u32, break_minutes: u32 },
}

impl TimerPreset {
    pub fn durations_seconds(&self) -> (u32, u32) {
        match *self {
            TimerPreset::Pomodoro25_5 => (25 * 60, 5 * 60),
            TimerPreset::Deep50_10 => (50 * 60, 10 * 60),
            TimerPreset::Custom {
                focus_minutes,
                break_minutes,
            } => (focus_minutes.max(1) * 60, break_minutes.max(1) * 60),
        }
    }

    pub fn label(&self) -> String {
        match *self {
            TimerPreset::Pomodoro25_5 => "25 / 5 Pomodoro".to_string(),
            TimerPreset::Deep50_10 => "50 / 10 Deep Focus".to_string(),
            TimerPreset::Custom {
                focus_minutes,
                break_minutes,
            } => format!("{} / {} Custom", focus_minutes, break_minutes),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionPhase {
    Idle,
    PreSessionQuote,
    RunningFocus,
    PausedFocus,
    RunningBreak,
    PostSessionOutcome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AmbientAccent {
    MutedTeal, // #2A8B82 (low luminance for OLED)
    WarmAmber, // #B8863B (zero blue-light night shift)
}

impl AmbientAccent {
    pub fn hex_color(&self) -> &'static str {
        match self {
            AmbientAccent::MutedTeal => "#2A8B82",
            AmbientAccent::WarmAmber => "#B8863B",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnatomicalStructureIcon {
    pub id: &'static str,
    pub name: &'static str,
    pub svg_filename: &'static str,
    pub short_fact: &'static str,
}

pub const ANATOMICAL_ICONS: &[AnatomicalStructureIcon] = &[
    AnatomicalStructureIcon {
        id: "heart",
        name: "Heart",
        svg_filename: "heart.svg",
        short_fact: "Heart: ~100,000 beats per day pumping ~7,200 L of blood",
    },
    AnatomicalStructureIcon {
        id: "lungs",
        name: "Lungs",
        svg_filename: "lungs.svg",
        short_fact: "Lungs: ~480 million alveoli providing ~70 m² of gas-exchange surface",
    },
    AnatomicalStructureIcon {
        id: "brain",
        name: "Brain",
        svg_filename: "brain.svg",
        short_fact: "Brain: ~86 billion neurons consuming 20% of resting body oxygen",
    },
    AnatomicalStructureIcon {
        id: "liver",
        name: "Liver",
        svg_filename: "liver.svg",
        short_fact: "Liver: Dual blood supply (75% portal vein, 25% hepatic artery) across 500+ metabolic functions",
    },
    AnatomicalStructureIcon {
        id: "kidneys",
        name: "Kidneys",
        svg_filename: "kidneys.svg",
        short_fact: "Kidneys: ~1 million nephrons per kidney filtering ~180 L of plasma daily",
    },
    AnatomicalStructureIcon {
        id: "stomach",
        name: "Stomach",
        svg_filename: "stomach.svg",
        short_fact: "Stomach: Parietal cells secrete HCl to maintain a lumen pH of 1.5–3.5",
    },
    AnatomicalStructureIcon {
        id: "intestines",
        name: "Intestines",
        svg_filename: "intestines.svg",
        short_fact: "Small Intestine: Plicae circulares, villi, and microvilli amplify absorptive area 600-fold",
    },
    AnatomicalStructureIcon {
        id: "spleen",
        name: "Spleen",
        svg_filename: "spleen.svg",
        short_fact: "Spleen: Red pulp cords of Billroth cull senescent RBCs with <3 µm slits",
    },
    AnatomicalStructureIcon {
        id: "pancreas",
        name: "Pancreas",
        svg_filename: "pancreas.svg",
        short_fact: "Pancreas: ~1 million Islets of Langerhans balance endocrine insulin and glucagon",
    },
    AnatomicalStructureIcon {
        id: "thyroid",
        name: "Thyroid",
        svg_filename: "thyroid.svg",
        short_fact: "Thyroid: Only endocrine gland storing hormone extracellularly in thyroglobulin colloid",
    },
    AnatomicalStructureIcon {
        id: "bladder",
        name: "Urinary Bladder",
        svg_filename: "bladder.svg",
        short_fact: "Bladder: Urothelial umbrella cells and plaques unfold to hold 400–600 mL at low pressure",
    },
    AnatomicalStructureIcon {
        id: "eye",
        name: "Eye & Retina",
        svg_filename: "eye.svg",
        short_fact: "Eye: ~120 million rods for scotopic vision and ~6 million cones concentrated at the fovea",
    },
    AnatomicalStructureIcon {
        id: "ear",
        name: "Inner Ear & Cochlea",
        svg_filename: "ear.svg",
        short_fact: "Ear: Ossicular chain amplifies tympanic membrane force ~22-fold at the oval window",
    },
    AnatomicalStructureIcon {
        id: "spine",
        name: "Vertebral Column",
        svg_filename: "spine.svg",
        short_fact: "Spine: 33 vertebrae and fibrocartilaginous annulus fibrosus discs absorb axial load",
    },
    AnatomicalStructureIcon {
        id: "skull",
        name: "Cranium",
        svg_filename: "skull.svg",
        short_fact: "Skull: 22 cranial and facial bones fused via fibrous Sharpey-fiber sutures",
    },
    AnatomicalStructureIcon {
        id: "femur",
        name: "Femur & Cortical Bone",
        svg_filename: "femur.svg",
        short_fact: "Femur: Trabecular arches in the femoral neck remodel along Wolff's law stress lines",
    },
    AnatomicalStructureIcon {
        id: "hand",
        name: "Hand & Carpus",
        svg_filename: "hand.svg",
        short_fact: "Hand: 27 bones innervated by median, ulnar, and radial nerves for fine motor precision",
    },
    AnatomicalStructureIcon {
        id: "neuron",
        name: "Multipolar Neuron",
        svg_filename: "neuron.svg",
        short_fact: "Neuron: Saltatory conduction at Nodes of Ranvier accelerates impulses up to 120 m/s",
    },
    AnatomicalStructureIcon {
        id: "rbc",
        name: "Red Blood Cell",
        svg_filename: "rbc.svg",
        short_fact: "Erythrocyte: Anucleate 7.5 µm biconcave disc carrying ~270 million hemoglobin molecules",
    },
    AnatomicalStructureIcon {
        id: "dna",
        name: "DNA Double Helix",
        svg_filename: "dna.svg",
        short_fact: "DNA: ~3.2 billion base pairs per haploid genome compacted ~10,000-fold into chromatin",
    },
    AnatomicalStructureIcon {
        id: "nephron",
        name: "Nephron & Glomerulus",
        svg_filename: "nephron.svg",
        short_fact: "Nephron: Countercurrent multiplication in the Loop of Henle builds a 1,200 mOsm/L gradient",
    },
    AnatomicalStructureIcon {
        id: "alveolus",
        name: "Alveolar-Capillary Unit",
        svg_filename: "alveolus.svg",
        short_fact: "Alveolus: Type II pneumocytes secrete dipalmitoylphosphatidylcholine surfactant after 28 weeks",
    },
];

/// deterministic 60-second burn-in protection pixel offset bounded within [-6, +6] px.
/// Shifts along a 16-point non-repeating diamond/Lissajous path so no OLED subpixel remains static.
pub fn compute_burn_in_pixel_shift(elapsed_minutes: u32) -> (i32, i32) {
    const PATH: [(i32, i32); 16] = [
        (0, 0),
        (2, -3),
        (4, -1),
        (6, 2),
        (3, 5),
        (0, 6),
        (-3, 4),
        (-6, 2),
        (-5, -2),
        (-3, -5),
        (0, -6),
        (3, -4),
        (5, 0),
        (2, 3),
        (-2, 1),
        (-4, -3),
    ];
    PATH[(elapsed_minutes as usize) % PATH.len()]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FocusTimerEngine {
    pub preset: TimerPreset,
    pub phase: SessionPhase,
    pub total_seconds: u32,
    pub remaining_seconds: u32,
    pub elapsed_seconds: u32,
    pub ambient_mode_active: bool,
    pub controls_visible: bool,
    pub controls_auto_hide_countdown_secs: u8,
    pub accent: AmbientAccent,
    pub target_screen_brightness: f32, // 0.05 (5%) in OLED ambient mode
    pub keep_screen_on_flag: bool,
    pub foreground_service_active: bool,
    pub started_wall_epoch_secs: Option<i64>,
    pub specialty_id: String,
    pub topic_id: String,
    pub pre_session_quote_id: String,
    pub post_session_why_it_matters: String,
}

impl FocusTimerEngine {
    pub fn new(preset: TimerPreset, specialty_id: &str, topic_id: &str) -> Self {
        let (focus_secs, _) = preset.durations_seconds();
        Self {
            preset,
            phase: SessionPhase::Idle,
            total_seconds: focus_secs,
            remaining_seconds: focus_secs,
            elapsed_seconds: 0,
            ambient_mode_active: true,
            controls_visible: false,
            controls_auto_hide_countdown_secs: 0,
            accent: AmbientAccent::MutedTeal,
            target_screen_brightness: 0.05,
            keep_screen_on_flag: false,
            foreground_service_active: false,
            started_wall_epoch_secs: None,
            specialty_id: specialty_id.to_string(),
            topic_id: topic_id.to_string(),
            pre_session_quote_id: String::new(),
            post_session_why_it_matters: String::new(),
        }
    }

    pub fn begin_pre_session_quote(&mut self, quote_id: &str, why_it_matters: &str) {
        self.pre_session_quote_id = quote_id.to_string();
        self.post_session_why_it_matters = why_it_matters.to_string();
        self.phase = SessionPhase::PreSessionQuote;
    }

    pub fn start_focus(&mut self, now_epoch_secs: i64) {
        let (focus_secs, _) = self.preset.durations_seconds();
        self.total_seconds = focus_secs;
        self.remaining_seconds = focus_secs;
        self.elapsed_seconds = 0;
        self.phase = SessionPhase::RunningFocus;
        self.ambient_mode_active = true;
        self.controls_visible = false;
        self.keep_screen_on_flag = true;
        self.foreground_service_active = true;
        self.target_screen_brightness = 0.05;
        self.started_wall_epoch_secs = Some(now_epoch_secs);
    }

    pub fn tick_one_second(&mut self) -> bool {
        if self.controls_visible && self.controls_auto_hide_countdown_secs > 0 {
            self.controls_auto_hide_countdown_secs -= 1;
            if self.controls_auto_hide_countdown_secs == 0 {
                self.controls_visible = false;
            }
        }

        match self.phase {
            SessionPhase::RunningFocus => {
                if self.remaining_seconds > 0 {
                    self.remaining_seconds -= 1;
                    self.elapsed_seconds += 1;
                }
                if self.remaining_seconds == 0 {
                    self.phase = SessionPhase::PostSessionOutcome;
                    self.keep_screen_on_flag = false;
                    self.foreground_service_active = false;
                    self.target_screen_brightness = -1.0; // restore system brightness
                    return true; // session finished!
                }
            }
            SessionPhase::RunningBreak => {
                if self.remaining_seconds > 0 {
                    self.remaining_seconds -= 1;
                    self.elapsed_seconds += 1;
                }
                if self.remaining_seconds == 0 {
                    self.phase = SessionPhase::Idle;
                    self.keep_screen_on_flag = false;
                    self.foreground_service_active = false;
                    return true;
                }
            }
            _ => {}
        }
        false
    }

    /// Screen-off-safe fallback reconciliation: if the user turns off the physical display or
    /// backgrounds the app while the Android ForegroundService holds a partial wake lock,
    /// reconcile elapsed time from the monotonic/wall clock timestamp on resume.
    pub fn reconcile_from_wall_clock(&mut self, current_epoch_secs: i64) -> bool {
        if self.phase != SessionPhase::RunningFocus && self.phase != SessionPhase::RunningBreak {
            return false;
        }
        if let Some(start) = self.started_wall_epoch_secs {
            let wall_elapsed = (current_epoch_secs - start).max(0) as u32;
            self.elapsed_seconds = wall_elapsed.min(self.total_seconds);
            self.remaining_seconds = self.total_seconds.saturating_sub(self.elapsed_seconds);
            if self.remaining_seconds == 0 {
                self.phase = SessionPhase::PostSessionOutcome;
                self.keep_screen_on_flag = false;
                self.foreground_service_active = false;
                return true;
            }
        }
        false
    }

    pub fn on_ambient_screen_tapped(&mut self) {
        self.controls_visible = !self.controls_visible;
        if self.controls_visible {
            self.controls_auto_hide_countdown_secs = 4;
        }
    }

    pub fn pause(&mut self) {
        if self.phase == SessionPhase::RunningFocus {
            self.phase = SessionPhase::PausedFocus;
        }
    }

    pub fn resume(&mut self, now_epoch_secs: i64) {
        if self.phase == SessionPhase::PausedFocus {
            self.phase = SessionPhase::RunningFocus;
            self.started_wall_epoch_secs = Some(now_epoch_secs - (self.elapsed_seconds as i64));
        }
    }

    pub fn stop_early(&mut self) {
        self.phase = SessionPhase::PostSessionOutcome;
        self.keep_screen_on_flag = false;
        self.foreground_service_active = false;
        self.target_screen_brightness = -1.0;
    }

    pub fn start_break(&mut self, now_epoch_secs: i64) {
        let (_, break_secs) = self.preset.durations_seconds();
        self.total_seconds = break_secs;
        self.remaining_seconds = break_secs;
        self.elapsed_seconds = 0;
        self.phase = SessionPhase::RunningBreak;
        self.started_wall_epoch_secs = Some(now_epoch_secs);
    }

    /// Rotates anatomical structure icon every 5 minutes (300 seconds) with smooth crossfade progress.
    pub fn current_anatomical_icon(&self) -> &'static AnatomicalStructureIcon {
        let index = ((self.elapsed_seconds / 300) as usize) % ANATOMICAL_ICONS.len();
        &ANATOMICAL_ICONS[index]
    }

    /// Returns crossfade opacity (0.15..=0.55) so line-art stays dim on OLED and gently crossfades
    /// during the 3 seconds before and after each 5-minute transition.
    pub fn icon_dim_opacity(&self) -> f32 {
        let mod_300 = self.elapsed_seconds % 300;
        if mod_300 < 3 {
            0.18 + (mod_300 as f32) * 0.10
        } else if mod_300 > 297 {
            0.18 + ((300 - mod_300) as f32) * 0.10
        } else {
            0.48
        }
    }

    pub fn current_burn_in_shift(&self) -> (i32, i32) {
        compute_burn_in_pixel_shift(self.elapsed_seconds / 60)
    }

    pub fn formatted_countdown(&self) -> String {
        let mins = self.remaining_seconds / 60;
        let secs = self.remaining_seconds % 60;
        format!("{:02}:{:02}", mins, secs)
    }
}
