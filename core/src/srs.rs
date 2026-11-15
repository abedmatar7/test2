use crate::models::Flashcard;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewGrade {
    Again = 1,
    Hard = 2,
    Good = 3,
    Easy = 4,
}

/// Review a flashcard using either FSRS-4.5 (default) or classic SM-2.
pub fn schedule_card(
    card: &mut Flashcard,
    grade: ReviewGrade,
    now_epoch_secs: i64,
    prefer_fsrs: bool,
) {
    let elapsed_days = if card.reps == 0 {
        0.0
    } else {
        ((now_epoch_secs - (card.due_epoch_secs - (card.interval_days * 86400.0) as i64)).max(0)
            as f64)
            / 86400.0
    };

    if prefer_fsrs {
        schedule_fsrs(card, grade, elapsed_days, now_epoch_secs);
    } else {
        schedule_sm2(card, grade, now_epoch_secs);
    }
}

fn schedule_fsrs(
    card: &mut Flashcard,
    grade: ReviewGrade,
    elapsed_days: f64,
    now_epoch_secs: i64,
) {
    let g = grade as i32 as f64; // 1.0..=4.0
    if card.reps == 0 {
        // Initial FSRS-4.5 stability and difficulty weights
        let init_stabilities = [0.4, 1.2, 3.1, 8.5];
        card.stability = init_stabilities[(grade as usize) - 1];
        card.difficulty = (6.8 - 0.9 * (g - 3.0)).clamp(1.0, 10.0);
    } else {
        let s = card.stability.max(0.2);
        let d = card.difficulty.clamp(1.0, 10.0);
        // FSRS-4.5 power-law forgetting curve R(t, S) = (1 + 19/81 * t/S)^(-0.5)
        let retrievability = (1.0 + (19.0 / 81.0) * (elapsed_days / s)).powf(-0.5);

        let next_d = (d - 0.75 * (g - 3.0)).clamp(1.0, 10.0);
        card.difficulty = 0.2 * 5.5 + 0.8 * next_d; // Mean reversion

        if grade == ReviewGrade::Again {
            card.lapses += 1;
            card.stability = (0.35 * d.powf(-0.2) * ((s + 1.0).powf(0.2) - 1.0)).max(0.25);
        } else {
            let hard_penalty = if grade == ReviewGrade::Hard { 0.82 } else { 1.0 };
            let easy_bonus = if grade == ReviewGrade::Easy { 1.35 } else { 1.0 };
            let growth = 1.0
                + (1.95_f64.exp())
                    * (11.0 - card.difficulty)
                    * s.powf(-0.18)
                    * (((1.0 - retrievability) * 0.9).exp_m1().max(0.12))
                    * hard_penalty
                    * easy_bonus;
            card.stability = (s * growth).clamp(0.5, 3650.0);
        }
    }

    card.reps += 1;
    // Target 90% desired retention => interval == stability S
    card.interval_days = if grade == ReviewGrade::Again {
        0.01 // ~15 minutes intra-day relearn
    } else {
        card.stability.round().max(1.0)
    };
    card.due_epoch_secs = now_epoch_secs + (card.interval_days * 86400.0) as i64;
}

fn schedule_sm2(card: &mut Flashcard, grade: ReviewGrade, now_epoch_secs: i64) {
    let q = match grade {
        ReviewGrade::Again => 1.0,
        ReviewGrade::Hard => 3.0,
        ReviewGrade::Good => 4.0,
        ReviewGrade::Easy => 5.0,
    };

    if grade == ReviewGrade::Again {
        card.lapses += 1;
        card.reps = 0;
        card.interval_days = 0.01;
    } else {
        card.reps += 1;
        card.interval_days = match card.reps {
            1 => 1.0,
            2 => 6.0,
            _ => (card.interval_days * card.ease_factor).round().max(1.0),
        };
    }

    let delta = 0.1 - (5.0 - q) * (0.08 + (5.0 - q) * 0.02);
    card.ease_factor = (card.ease_factor + delta).max(1.3);
    card.due_epoch_secs = now_epoch_secs + (card.interval_days * 86400.0) as i64;
}

/// Renders a Cloze deletion template such as:
/// `"Auer rods are {{c1::peroxidase-positive::stain}} inclusions seen in {{c2::Acute Promyelocytic Leukemia (M3)}}."`
/// For `active_cloze_index = 1`:
/// - `question` -> `"Auer rods are [stain] inclusions seen in Acute Promyelocytic Leukemia (M3)."`
/// - `answer`   -> `"Auer rods are peroxidase-positive inclusions seen in Acute Promyelocytic Leukemia (M3)."`
pub fn render_cloze(raw_text: &str, active_cloze_index: u8) -> (String, String) {
    let mut question = String::new();
    let mut answer = String::new();
    let mut rest = raw_text;

    while let Some(start) = rest.find("{{c") {
        question.push_str(&rest[..start]);
        answer.push_str(&rest[..start]);
        let after_prefix = &rest[start + 3..];
        if let Some(end) = after_prefix.find("}}") {
            let inside = &after_prefix[..end];
            if let Some(colon_pos) = inside.find("::") {
                let idx_str = &inside[..colon_pos];
                let idx: u8 = idx_str.parse().unwrap_or(1);
                let payload = &inside[colon_pos + 2..];
                let (secret, hint) = if let Some(h_pos) = payload.find("::") {
                    (&payload[..h_pos], Some(&payload[h_pos + 2..]))
                } else {
                    (payload, None)
                };

                if idx == active_cloze_index {
                    if let Some(h) = hint {
                        question.push_str(&format!("[...{}...]", h));
                    } else {
                        question.push_str("[...]");
                    }
                } else {
                    question.push_str(secret);
                }
                answer.push_str(secret);
            }
            rest = &after_prefix[end + 2..];
        } else {
            question.push_str(&rest[start..]);
            answer.push_str(&rest[start..]);
            break;
        }
    }
    question.push_str(rest);
    answer.push_str(rest);
    (question, answer)
}
