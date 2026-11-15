use crate::models::{Flashcard, Specialty, StudySessionRecord, VignetteQuestion};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPlanItem {
    pub order: usize,
    pub specialty_id: String,
    pub topic_id: String,
    pub activity_type: String, // "Due Flashcards", "Weak-Topic Remediation", "Interleaved Block"
    pub title: String,
    pub recommended_minutes: u32,
    pub clinical_rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeakTopicMetric {
    pub specialty_id: String,
    pub topic_id: String,
    pub accuracy_percent: f64,
    pub card_lapses: u32,
    pub heat_severity: u8, // 0 = strong, 1 = moderate, 2 = high priority weak topic
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardSnapshot {
    pub active_streak_days: u32,
    pub rest_days_used_last_7: u32,
    pub streak_grace_active: bool,
    pub minutes_by_specialty: HashMap<String, u32>,
    pub overall_retention_percent: f64,
    pub due_cards_today: usize,
    pub weak_topic_heatmap: Vec<WeakTopicMetric>,
    pub weekly_minutes_completed: u32,
    pub weekly_goal_minutes: u32,
    pub exam_days_remaining: i64,
}

/// Computes study streak in days with evidence-based Rest-Day Grace:
/// Up to 1 rest day (0 study minutes) in any rolling 7-day window is forgiven without breaking the streak.
pub fn compute_streak_with_rest_grace(
    active_study_dates: &BTreeSet<NaiveDate>,
    today: NaiveDate,
) -> (u32, u32, bool) {
    let mut streak = 0u32;
    let mut cursor = today;
    let mut rest_days_in_last_7 = 0u32;
    let mut grace_used = false;
    let mut days_checked = 0u32;
    let mut last_grace_day_index: Option<u32> = None;

    loop {
        if active_study_dates.contains(&cursor) {
            streak += 1;
        } else {
            // Allow 1 rest day per rolling 7-day window if the immediately preceding day was studied
            let prev_day_studied = cursor
                .checked_sub_days(chrono::Days::new(1))
                .map(|prev| active_study_dates.contains(&prev))
                .unwrap_or(false);

            let can_use_grace = match last_grace_day_index {
                Some(last_idx) => days_checked.saturating_sub(last_idx) >= 7,
                None => true,
            };

            if prev_day_studied && can_use_grace {
                grace_used = true;
                last_grace_day_index = Some(days_checked);
                if days_checked < 7 {
                    rest_days_in_last_7 += 1;
                }
            } else {
                break;
            }
        }

        days_checked += 1;
        if let Some(prev) = cursor.checked_sub_days(chrono::Days::new(1)) {
            cursor = prev;
        } else {
            break;
        }
        if days_checked > 3650 {
            break;
        }
    }

    (streak, rest_days_in_last_7, grace_used)
}

/// Builds a Smart Daily Study Plan combining:
/// 1. Due SRS Flashcards
/// 2. Weak topics identified from Q-Bank accuracy (<75%) or SRS lapses
/// 3. Interleaved foundational + clinical subjects so adjacent blocks contrast mechanisms
pub fn build_smart_daily_plan(
    specialties: &[Specialty],
    cards: &[Flashcard],
    questions: &[VignetteQuestion],
    now_epoch_secs: i64,
) -> Vec<DailyPlanItem> {
    let mut plan = Vec::new();
    let mut order = 1;

    // 1. Due Flashcards grouped by specialty
    let due_cards: Vec<&Flashcard> = cards
        .iter()
        .filter(|c| c.due_epoch_secs <= now_epoch_secs)
        .collect();

    if !due_cards.is_empty() {
        let est_minutes = ((due_cards.len() as u32) / 2).clamp(15, 25);
        plan.push(DailyPlanItem {
            order,
            specialty_id: due_cards[0].specialty_id.clone(),
            topic_id: due_cards[0].topic_id.clone(),
            activity_type: "Due Flashcards".to_string(),
            title: format!("Clear {} Due Active-Recall & Cloze Cards", due_cards.len()),
            recommended_minutes: est_minutes,
            clinical_rationale: "Completing due FSRS reviews at ~90% retrievability doubles stability before synaptic decay.".to_string(),
        });
        order += 1;
    }

    // 2. Weak-Topic Remediation
    let mut weak_topics: Vec<(&VignetteQuestion, f64)> = questions
        .iter()
        .filter(|q| q.attempts > 0 && q.accuracy() < 0.75)
        .map(|q| (q, q.accuracy()))
        .collect();
    weak_topics.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

    if let Some((weak_q, acc)) = weak_topics.first() {
        plan.push(DailyPlanItem {
            order,
            specialty_id: weak_q.specialty_id.clone(),
            topic_id: weak_q.topic_id.clone(),
            activity_type: "Weak-Topic Remediation".to_string(),
            title: format!(
                "Targeted Vignette & Feynman Drill: {} ({:.0}% accuracy)",
                weak_q.topic_id,
                acc * 100.0
            ),
            recommended_minutes: 25,
            clinical_rationale: "Prioritizes highest-error clinical discriminators before adding new material.".to_string(),
        });
        order += 1;
    }

    // 3. Interleaved Foundational + Clinical Pairing
    let foundational = specialties
        .iter()
        .find(|s| s.category == "Foundational Science")
        .or_else(|| specialties.first());
    let clinical = specialties
        .iter()
        .find(|s| s.category == "Clinical Clerkship")
        .or_else(|| specialties.last());

    if let Some(f_spec) = foundational {
        if let Some(topic) = f_spec.topics.first() {
            plan.push(DailyPlanItem {
                order,
                specialty_id: f_spec.id.clone(),
                topic_id: topic.id.clone(),
                activity_type: "Interleaved Block A (Mechanism)".to_string(),
                title: format!("{}: {}", f_spec.name, topic.name),
                recommended_minutes: 25,
                clinical_rationale: topic.why_it_matters.clone(),
            });
            order += 1;
        }
    }

    if let Some(c_spec) = clinical {
        if let Some(topic) = c_spec.topics.first() {
            plan.push(DailyPlanItem {
                order,
                specialty_id: c_spec.id.clone(),
                topic_id: topic.id.clone(),
                activity_type: "Interleaved Block B (Bedside Application)".to_string(),
                title: format!("{}: {}", c_spec.name, topic.name),
                recommended_minutes: 25,
                clinical_rationale: topic.why_it_matters.clone(),
            });
        }
    }

    plan
}

pub fn compute_weak_topic_heatmap(
    cards: &[Flashcard],
    questions: &[VignetteQuestion],
) -> Vec<WeakTopicMetric> {
    let mut map: HashMap<(String, String), (u32, u32, u32)> = HashMap::new(); // (attempts, correct, lapses)

    for q in questions {
        let entry = map
            .entry((q.specialty_id.clone(), q.topic_id.clone()))
            .or_insert((0, 0, 0));
        entry.0 += q.attempts;
        entry.1 += q.correct_count;
    }
    for c in cards {
        let entry = map
            .entry((c.specialty_id.clone(), c.topic_id.clone()))
            .or_insert((0, 0, 0));
        entry.2 += c.lapses;
    }

    let mut metrics: Vec<WeakTopicMetric> = map
        .into_iter()
        .map(|((specialty_id, topic_id), (attempts, correct, lapses))| {
            let acc = if attempts == 0 {
                100.0
            } else {
                (correct as f64 / attempts as f64) * 100.0
            };
            let severity = if acc < 65.0 || lapses >= 3 {
                2
            } else if acc < 80.0 || lapses >= 1 {
                1
            } else {
                0
            };
            WeakTopicMetric {
                specialty_id,
                topic_id,
                accuracy_percent: acc,
                card_lapses: lapses,
                heat_severity: severity,
            }
        })
        .collect();

    metrics.sort_by(|a, b| b.heat_severity.cmp(&a.heat_severity));
    metrics
}

pub fn aggregate_minutes_by_specialty(sessions: &[StudySessionRecord]) -> HashMap<String, u32> {
    let mut totals = HashMap::new();
    for s in sessions.iter().filter(|s| s.completed) {
        *totals.entry(s.specialty_id.clone()).or_insert(0) += s.duration_minutes;
    }
    totals
}
