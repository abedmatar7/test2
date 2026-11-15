use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuoteTheme {
    Perseverance,
    Compassion,
    Patients,
    Resilience,
    Curiosity,
    LifelongLearning,
}

impl QuoteTheme {
    pub fn label(&self) -> &'static str {
        match self {
            QuoteTheme::Perseverance => "Perseverance",
            QuoteTheme::Compassion => "Compassion",
            QuoteTheme::Patients => "Patients First",
            QuoteTheme::Resilience => "Resilience",
            QuoteTheme::Curiosity => "Scientific Curiosity",
            QuoteTheme::LifelongLearning => "Lifelong Learning",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quote {
    pub id: String,
    pub text: String,
    pub author: String,
    pub source_note: String,
    pub attribution_verified: bool,
    pub theme: QuoteTheme,
    #[serde(default)]
    pub is_favorite: bool,
    #[serde(default)]
    pub shown_in_current_cycle: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Specialty {
    pub id: String,
    pub name: String,
    pub is_custom: bool,
    pub category: String, // "Foundational Science" or "Clinical Clerkship"
    pub default_why_it_matters: String,
    pub topics: Vec<Topic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Topic {
    pub id: String,
    pub specialty_id: String,
    pub name: String,
    pub why_it_matters: String,
    pub target_minutes_weekly: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CardKind {
    ActiveRecall,
    ClozeDeletion,
    ImageCard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageModality {
    None,
    Ecg,
    Histology,
    XRay,
    AnatomyDiagram,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Flashcard {
    pub id: String,
    pub specialty_id: String,
    pub topic_id: String,
    pub kind: CardKind,
    pub front: String,
    pub back: String,
    pub extra_clinical_pearl: String,
    pub image_asset_path: Option<String>,
    pub image_modality: ImageModality,
    // SRS state (supports both SM-2 and FSRS-4.5)
    pub due_epoch_secs: i64,
    pub interval_days: f64,
    pub ease_factor: f64,      // SM-2 ease (default 2.5)
    pub stability: f64,        // FSRS stability S in days
    pub difficulty: f64,       // FSRS difficulty D in [1.0, 10.0]
    pub reps: u32,
    pub lapses: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VignetteQuestion {
    pub id: String,
    pub specialty_id: String,
    pub topic_id: String,
    pub stem: String,
    pub lead_in: String,
    pub options: Vec<String>,
    pub correct_index: usize,
    pub explanation: String,
    pub distractor_rationales: Vec<String>,
    pub attempts: u32,
    pub correct_count: u32,
}

impl VignetteQuestion {
    pub fn accuracy(&self) -> f64 {
        if self.attempts == 0 {
            1.0
        } else {
            self.correct_count as f64 / self.attempts as f64
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseStep {
    pub step_number: u8,
    pub stage_title: String, // e.g., "1. Chief Complaint & Triage", "2. Focused History", "3. Physical Exam", "4. Labs & Pathology", "5. Definitive Diagnosis"
    pub clinical_findings: String,
    pub guiding_prompt: String,
    pub key_differentials_to_consider: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClinicalCase {
    pub id: String,
    pub specialty_id: String,
    pub topic_id: String,
    pub title: String,
    pub patient_summary: String,
    pub steps: Vec<CaseStep>,
    pub final_diagnosis: String,
    pub diagnostic_criteria_pearl: String,
    pub why_it_matters: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OsceStationType {
    HistoryTaking,
    PhysicalExamination,
    PatientCounseling,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsceChecklistItem {
    pub item_id: String,
    pub description: String,
    pub critical_action: bool,
    #[serde(default)]
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsceStation {
    pub id: String,
    pub specialty_id: String,
    pub topic_id: String,
    pub title: String,
    pub station_type: OsceStationType,
    pub duration_seconds: u32,
    pub candidate_instructions: String,
    pub patient_script_notes: String,
    pub checklist: Vec<OsceChecklistItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeynmanNote {
    pub id: String,
    pub specialty_id: String,
    pub topic_id: String,
    pub title: String,
    pub plain_language_summary: String, // Explain it to a patient / 12-year-old
    pub knowledge_gaps_identified: String,
    pub mnemonic: String,
    pub updated_epoch_secs: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudySessionRecord {
    pub id: String,
    pub specialty_id: String,
    pub topic_id: String,
    pub started_epoch_secs: i64,
    pub ended_epoch_secs: i64,
    pub duration_minutes: u32,
    pub mode: String, // "Pomodoro 25/5", "Deep 50/10", "Custom"
    pub quote_id_shown: String,
    pub why_this_matters_shown: String,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoodCheckIn {
    pub timestamp_epoch_secs: i64,
    pub mood_score: u8,         // 1..=5
    pub energy_score: u8,       // 1..=5
    pub self_compassion_prompt: String,
    pub reflection_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationIntention {
    pub id: String,
    pub specialty_id: String,
    pub topic_name: String,
    pub planned_time_str: String, // e.g. "19:30"
    pub location_cue: String,     // e.g. "at my desk with water bottle ready"
    pub completed: bool,
}

impl ImplementationIntention {
    pub fn formatted_statement(&self) -> String {
        format!(
            "I will study {} at {} {}.",
            self.topic_name, self.planned_time_str, self.location_cue
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeeklyReflection {
    pub week_start_iso: String,
    pub wins_summary: String,
    pub hardest_concept: String,
    pub adjustment_for_next_week: String,
    pub sleep_hygiene_target_met_days: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserGoalsAndSettings {
    pub daily_goal_minutes: u32,
    pub weekly_goal_minutes: u32,
    pub exam_name: String,
    pub exam_date_iso: String, // YYYY-MM-DD
    pub sleep_reminder_hour: u8,
    pub sleep_reminder_minute: u8,
    pub prefer_fsrs: bool,
    pub pure_black_dark_mode: bool,
    pub ambient_accent: String, // "muted_teal" or "warm_amber"
}

impl Default for UserGoalsAndSettings {
    fn default() -> Self {
        Self {
            daily_goal_minutes: 100,
            weekly_goal_minutes: 600,
            exam_name: "Pathology & Clinical Medicine Boards".to_string(),
            exam_date_iso: "2027-01-15".to_string(),
            sleep_reminder_hour: 22,
            sleep_reminder_minute: 45,
            prefer_fsrs: true,
            pure_black_dark_mode: true,
            ambient_accent: "muted_teal".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPayload {
    pub schema_version: u32,
    pub exported_at: DateTime<Utc>,
    pub settings: UserGoalsAndSettings,
    pub custom_specialties: Vec<Specialty>,
    pub flashcards: Vec<Flashcard>,
    pub questions: Vec<VignetteQuestion>,
    pub sessions: Vec<StudySessionRecord>,
    pub notes: Vec<FeynmanNote>,
    pub mood_logs: Vec<MoodCheckIn>,
    pub intentions: Vec<ImplementationIntention>,
    pub reflections: Vec<WeeklyReflection>,
    pub favorite_quote_ids: Vec<String>,
    pub seen_quote_ids: Vec<String>,
}

pub fn days_until_exam(exam_date_iso: &str, today: NaiveDate) -> i64 {
    if let Ok(target) = NaiveDate::parse_from_str(exam_date_iso, "%Y-%m-%d") {
        (target - today).num_days().max(0)
    } else {
        0
    }
}
