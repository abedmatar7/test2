use chrono::NaiveDate;
use medpulse_core::models::{
    CardKind, Flashcard, ImageModality, Specialty, StudySessionRecord, Topic, VignetteQuestion,
};
use medpulse_core::planner::{build_smart_daily_plan, compute_streak_with_rest_grace};
use medpulse_core::quotes::QuoteDeck;
use medpulse_core::srs::{render_cloze, schedule_card, ReviewGrade};
use medpulse_core::storage::OfflineStorage;
use medpulse_core::timer::{compute_burn_in_pixel_shift, FocusTimerEngine, SessionPhase, TimerPreset};
use std::collections::BTreeSet;

#[test]
fn test_oled_timer_and_burn_in_shift_and_icon_rotation() {
    let mut timer = FocusTimerEngine::new(TimerPreset::Pomodoro25_5, "pathology", "cellular_injury");
    timer.begin_pre_session_quote("q_001", "Early recognition of reversible ischemia saves myocardium.");
    assert_eq!(timer.phase, SessionPhase::PreSessionQuote);

    timer.start_focus(1_700_000_000);
    assert_eq!(timer.phase, SessionPhase::RunningFocus);
    assert!(timer.keep_screen_on_flag);
    assert!(timer.foreground_service_active);
    assert_eq!(timer.current_anatomical_icon().id, "heart");

    // Advance 5 minutes (300 seconds) -> icon should crossfade to Lungs and pixel shift should change
    for _ in 0..300 {
        assert!(!timer.tick_one_second());
    }
    assert_eq!(timer.current_anatomical_icon().id, "lungs");
    let shift_at_5m = compute_burn_in_pixel_shift(5);
    assert!(shift_at_5m.0.abs() <= 6 && shift_at_5m.1.abs() <= 6);
    assert_ne!(shift_at_5m, (0, 0));

    // Screen-off wall clock reconciliation to end of 25-minute block
    let finished = timer.reconcile_from_wall_clock(1_700_000_000 + 25 * 60);
    assert!(finished);
    assert_eq!(timer.phase, SessionPhase::PostSessionOutcome);
    assert!(!timer.keep_screen_on_flag);
}

#[test]
fn test_fsrs_and_cloze_deletion() {
    let (q, a) = render_cloze(
        "Reed-Sternberg cells are {{c1::CD15 and CD30::immunophenotype}} positive in {{c2::Classic Hodgkin Lymphoma}}.",
        1,
    );
    assert_eq!(
        q,
        "Reed-Sternberg cells are [...immunophenotype...] positive in Classic Hodgkin Lymphoma."
    );
    assert_eq!(
        a,
        "Reed-Sternberg cells are CD15 and CD30 positive in Classic Hodgkin Lymphoma."
    );

    let mut card = Flashcard {
        id: "card_1".to_string(),
        specialty_id: "pathology".to_string(),
        topic_id: "hematopathology".to_string(),
        kind: CardKind::ClozeDeletion,
        front: q,
        back: a,
        extra_clinical_pearl: "Owl-eye bilobed nuclei with eosinophilic nucleoli.".to_string(),
        image_asset_path: None,
        image_modality: ImageModality::Histology,
        due_epoch_secs: 1_700_000_000,
        interval_days: 0.0,
        ease_factor: 2.5,
        stability: 0.0,
        difficulty: 5.0,
        reps: 0,
        lapses: 0,
    };

    schedule_card(&mut card, ReviewGrade::Good, 1_700_000_000, true);
    assert_eq!(card.reps, 1);
    assert!(card.stability >= 3.0);
    let first_stability = card.stability;

    // Second review after 3 days -> stability increases
    schedule_card(
        &mut card,
        ReviewGrade::Good,
        1_700_000_000 + 3 * 86400,
        true,
    );
    assert_eq!(card.reps, 2);
    assert!(card.stability > first_stability);
}

#[test]
fn test_quote_deck_never_repeats_until_cycle_exhausted() {
    let raw = std::fs::read_to_string("../assets/quotes.json").expect("quotes.json must exist");
    let mut deck = QuoteDeck::from_json_str(&raw).expect("valid quotes JSON");
    let total = deck.total_count();
    assert!(total >= 150, "Expected at least 150 quotes, found {}", total);

    let mut seen_ids = BTreeSet::new();
    for _ in 0..total {
        let q = deck.draw_next_quote(None).expect("quote available");
        assert!(
            seen_ids.insert(q.id.clone()),
            "Quote repeated before all quotes were shown!"
        );
    }
    assert_eq!(seen_ids.len(), total);

    // Drawing one more automatically resets the cycle
    let next_cycle_q = deck.draw_next_quote(None).expect("cycle resets");
    assert!(seen_ids.contains(&next_cycle_q.id));
}

#[test]
fn test_streak_with_rest_day_grace() {
    let mut dates = BTreeSet::new();
    let today = NaiveDate::from_ymd_opt(2026, 11, 15).unwrap();
    // Studied Nov 15, Nov 14, REST on Nov 13, Studied Nov 12, Nov 11, Nov 10
    dates.insert(NaiveDate::from_ymd_opt(2026, 11, 15).unwrap());
    dates.insert(NaiveDate::from_ymd_opt(2026, 11, 14).unwrap());
    dates.insert(NaiveDate::from_ymd_opt(2026, 11, 12).unwrap());
    dates.insert(NaiveDate::from_ymd_opt(2026, 11, 11).unwrap());
    dates.insert(NaiveDate::from_ymd_opt(2026, 11, 10).unwrap());

    let (streak, rest_days, grace_used) = compute_streak_with_rest_grace(&dates, today);
    assert_eq!(streak, 5);
    assert_eq!(rest_days, 1);
    assert!(grace_used);
}

#[test]
fn test_sqlite_storage_and_json_backup_roundtrip() {
    let store = OfflineStorage::open_in_memory().unwrap();
    let session = StudySessionRecord {
        id: "sess_01".to_string(),
        specialty_id: "pathology".to_string(),
        topic_id: "neoplasia".to_string(),
        started_epoch_secs: 1_700_000_000,
        ended_epoch_secs: 1_700_001_500,
        duration_minutes: 25,
        mode: "Pomodoro 25/5".to_string(),
        quote_id_shown: "q_001".to_string(),
        why_this_matters_shown: "Grading and staging directly dictate surgical margins vs. adjuvant chemotherapy.".to_string(),
        completed: true,
    };
    store.record_session(&session).unwrap();

    let backup_json = store
        .export_backup_json(vec![], vec![], vec![], vec![], vec![], vec![], vec![])
        .unwrap();

    let store2 = OfflineStorage::open_in_memory().unwrap();
    let restored = store2.import_backup_json(&backup_json).unwrap();
    assert_eq!(restored.sessions.len(), 1);
    assert_eq!(restored.sessions[0].topic_id, "neoplasia");
}
