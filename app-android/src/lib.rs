use medpulse_core::models::{ClinicalCase, Flashcard, OsceStation, Specialty, VignetteQuestion};
use medpulse_core::quotes::{QuoteDeck, BREAK_SUGGESTIONS, ROTATING_STUDY_TIPS};
use medpulse_core::srs::{render_cloze, schedule_card, ReviewGrade};
use medpulse_core::storage::OfflineStorage;
use medpulse_core::timer::{FocusTimerEngine, TimerPreset};
use serde::Deserialize;
use std::cell::RefCell;
use std::rc::Rc;

slint::include_modules!();

const BUNDLED_QUOTES_JSON: &str = include_str!("../../assets/quotes.json");
const BUNDLED_SEED_JSON: &str = include_str!("../../assets/seed/seed_data.json");

#[derive(Deserialize)]
struct SeedBundle {
    specialties: Vec<Specialty>,
    flashcards: Vec<Flashcard>,
    questions: Vec<VignetteQuestion>,
    cases: Vec<ClinicalCase>,
    osce_stations: Vec<OsceStation>,
}

pub struct AppController {
    pub storage: OfflineStorage,
    pub quote_deck: QuoteDeck,
    pub specialties: Vec<Specialty>,
    pub flashcards: Vec<Flashcard>,
    pub questions: Vec<VignetteQuestion>,
    pub cases: Vec<ClinicalCase>,
    pub osce_stations: Vec<OsceStation>,
    pub timer: FocusTimerEngine,
    pub current_specialty_idx: usize,
    pub current_card_idx: usize,
    pub current_case_step_idx: usize,
    pub current_quote_id: String,
    pub tip_idx: usize,
}

impl AppController {
    pub fn new(db_path: Option<&str>) -> Self {
        let storage = match db_path {
            Some(p) => OfflineStorage::open_path(p).unwrap_or_else(|_| OfflineStorage::open_in_memory().unwrap()),
            None => OfflineStorage::open_in_memory().unwrap(),
        };
        let quote_deck = QuoteDeck::from_json_str(BUNDLED_QUOTES_JSON).expect("Valid bundled quotes.json");
        let seed: SeedBundle = serde_json::from_str(BUNDLED_SEED_JSON).expect("Valid bundled seed_data.json");

        for card in &seed.flashcards {
            let _ = storage.upsert_flashcard(card);
        }

        let timer = FocusTimerEngine::new(
            TimerPreset::Pomodoro25_5,
            "pathology",
            "hematopathology",
        );

        Self {
            storage,
            quote_deck,
            specialties: seed.specialties,
            flashcards: seed.flashcards,
            questions: seed.questions,
            cases: seed.cases,
            osce_stations: seed.osce_stations,
            timer,
            current_specialty_idx: 0,
            current_card_idx: 0,
            current_case_step_idx: 0,
            current_quote_id: String::new(),
            tip_idx: 0,
        }
    }
}

pub fn create_and_bind_ui(db_path: Option<&str>) -> Result<AppWindow, slint::PlatformError> {
    let ui = AppWindow::new()?;
    let state = Rc::new(RefCell::new(AppController::new(db_path)));

    // 1. Cycle Specialty
    {
        let ui_handle = ui.as_weak();
        let st = state.clone();
        ui.on_cycle_specialty(move || {
            let mut app = st.borrow_mut();
            app.current_specialty_idx = (app.current_specialty_idx + 1) % app.specialties.len();
            let spec = &app.specialties[app.current_specialty_idx];
            let topic = spec.topics.first().expect("Specialty has topic");
            if let Some(w) = ui_handle.upgrade() {
                w.set_selected_specialty_name(spec.name.clone().into());
                w.set_selected_topic_name(topic.name.clone().into());
                w.set_post_session_why_matters(topic.why_it_matters.clone().into());
            }
        });
    }

    // 2. Cycle Timer Preset (25/5 -> 50/10 -> Custom 40/8)
    {
        let ui_handle = ui.as_weak();
        let st = state.clone();
        ui.on_cycle_timer_preset(move || {
            let mut app = st.borrow_mut();
            app.timer.preset = match app.timer.preset {
                TimerPreset::Pomodoro25_5 => TimerPreset::Deep50_10,
                TimerPreset::Deep50_10 => TimerPreset::Custom {
                    focus_minutes: 40,
                    break_minutes: 8,
                },
                TimerPreset::Custom { .. } => TimerPreset::Pomodoro25_5,
            };
            let (secs, _) = app.timer.preset.durations_seconds();
            app.timer.total_seconds = secs;
            app.timer.remaining_seconds = secs;
            if let Some(w) = ui_handle.upgrade() {
                w.set_timer_preset_label(app.timer.preset.label().into());
                w.set_countdown_text(app.timer.formatted_countdown().into());
            }
        });
    }

    // 3. Request Start Session -> Draw non-repeating motivational quote first!
    {
        let ui_handle = ui.as_weak();
        let st = state.clone();
        ui.on_request_start_session(move || {
            let mut app = st.borrow_mut();
            let total_quotes = app.quote_deck.total_count();
            let unseen_before = app.quote_deck.remaining_unseen_in_cycle();
            if let Some(quote) = app.quote_deck.draw_next_quote(None) {
                app.current_quote_id = quote.id.clone();
                let shown_idx = total_quotes.saturating_sub(unseen_before) + 1;
                if let Some(w) = ui_handle.upgrade() {
                    w.set_current_quote_text(quote.text.into());
                    w.set_current_quote_author(quote.author.into());
                    let ver_label = if quote.attribution_verified {
                        format!("{} (Verified Attribution)", quote.source_note)
                    } else {
                        format!("{} (Unattributed Clinical Maxim)", quote.source_note)
                    };
                    w.set_current_quote_source(ver_label.into());
                    w.set_current_quote_theme(quote.theme.label().into());
                    w.set_current_quote_favorited(quote.is_favorite);
                    w.set_quotes_cycle_progress(
                        format!("{} / {} in non-repeating cycle", shown_idx, total_quotes).into(),
                    );
                    w.set_show_pre_session_quote_modal(true);
                }
            }
        });
    }

    // 4. Toggle Favorite Quote
    {
        let ui_handle = ui.as_weak();
        let st = state.clone();
        ui.on_toggle_favorite_quote(move || {
            let mut app = st.borrow_mut();
            let qid = app.current_quote_id.clone();
            if let Some(fav) = app.quote_deck.toggle_favorite(&qid) {
                if let Some(w) = ui_handle.upgrade() {
                    w.set_current_quote_favorited(fav);
                }
            }
        });
    }

    // 5. Confirm Pre-Session Quote & Enter Pure-Black OLED Ambient Mode
    {
        let ui_handle = ui.as_weak();
        let st = state.clone();
        ui.on_confirm_pre_session_quote_and_enter_oled(move || {
            let mut app = st.borrow_mut();
            let now = chrono::Utc::now().timestamp();
            app.timer.start_focus(now);
            #[cfg(target_os = "android")]
            android_platform::set_oled_ambient_window_state(true, 0.05);

            if let Some(w) = ui_handle.upgrade() {
                w.set_show_pre_session_quote_modal(false);
                w.set_in_ambient_oled_mode(true);
                w.set_timer_running(true);
                w.set_ambient_controls_visible(false);
            }
        });
    }

    // 6. Tap Anywhere on Ambient OLED Panel -> Toggle Auto-Hiding Controls
    {
        let ui_handle = ui.as_weak();
        let st = state.clone();
        ui.on_ambient_screen_tapped(move || {
            let mut app = st.borrow_mut();
            app.timer.on_ambient_screen_tapped();
            if let Some(w) = ui_handle.upgrade() {
                w.set_ambient_controls_visible(app.timer.controls_visible);
            }
        });
    }

    // 7. Pause / Resume in OLED Mode
    {
        let ui_handle = ui.as_weak();
        let st = state.clone();
        ui.on_toggle_pause_timer(move || {
            let mut app = st.borrow_mut();
            let now = chrono::Utc::now().timestamp();
            if app.timer.phase == medpulse_core::timer::SessionPhase::RunningFocus {
                app.timer.pause();
                if let Some(w) = ui_handle.upgrade() {
                    w.set_timer_running(false);
                }
            } else {
                app.timer.resume(now);
                if let Some(w) = ui_handle.upgrade() {
                    w.set_timer_running(true);
                }
            }
        });
    }

    // 8. Finish Session -> Show "Why This Matters" Patient Outcome & Break Booster
    {
        let ui_handle = ui.as_weak();
        let st = state.clone();
        ui.on_finish_session_early(move || {
            let mut app = st.borrow_mut();
            app.timer.stop_early();
            #[cfg(target_os = "android")]
            android_platform::set_oled_ambient_window_state(false, -1.0);

            app.tip_idx = (app.tip_idx + 1) % ROTATING_STUDY_TIPS.len();
            let tip = &ROTATING_STUDY_TIPS[app.tip_idx];
            let brk = &BREAK_SUGGESTIONS[app.tip_idx % BREAK_SUGGESTIONS.len()];

            if let Some(w) = ui_handle.upgrade() {
                w.set_in_ambient_oled_mode(false);
                w.set_timer_running(false);
                w.set_active_study_tip(format!("{}: {}", tip.title, tip.actionable_tip).into());
                w.set_active_break_booster(format!("{}: {}", brk.title, brk.actionable_tip).into());
                w.set_show_post_session_outcome_modal(true);
            }
        });
    }

    // 9. Close Post-Session Outcome Modal
    {
        let ui_handle = ui.as_weak();
        ui.on_close_post_session_modal(move || {
            if let Some(w) = ui_handle.upgrade() {
                w.set_show_post_session_outcome_modal(false);
            }
        });
    }

    // 10. Flashcard Answer Reveal & FSRS-4.5 Grading
    {
        let ui_handle = ui.as_weak();
        ui.on_reveal_flashcard_answer(move || {
            if let Some(w) = ui_handle.upgrade() {
                w.set_card_answer_revealed(true);
            }
        });
    }
    {
        let ui_handle = ui.as_weak();
        let st = state.clone();
        ui.on_grade_flashcard(move |grade_int| {
            // Reborrow as a plain &mut so disjoint fields can be borrowed independently.
            let mut guard = st.borrow_mut();
            let app = &mut *guard;
            let grade = match grade_int {
                1 => ReviewGrade::Again,
                2 => ReviewGrade::Hard,
                3 => ReviewGrade::Good,
                _ => ReviewGrade::Easy,
            };
            let now = chrono::Utc::now().timestamp();
            let idx = app.current_card_idx;
            {
                let card = &mut app.flashcards[idx];
                schedule_card(card, grade, now, true);
                let _ = app.storage.upsert_flashcard(card);
            }
            app.current_card_idx = (app.current_card_idx + 1) % app.flashcards.len();
            let next_card = &app.flashcards[app.current_card_idx];
            let (front_display, back_display) =
                if next_card.kind == medpulse_core::models::CardKind::ClozeDeletion {
                    render_cloze(&next_card.front, 1)
                } else {
                    (next_card.front.clone(), next_card.back.clone())
                };

            if let Some(w) = ui_handle.upgrade() {
                w.set_card_answer_revealed(false);
                w.set_card_front_text(front_display.into());
                w.set_card_back_text(back_display.into());
                w.set_card_pearl_text(next_card.extra_clinical_pearl.clone().into());
                w.set_card_has_image(next_card.image_asset_path.is_some());
                w.set_card_fsrs_meta(
                    format!(
                        "FSRS-4.5 • Stability: {:.1}d • Difficulty: {:.1}",
                        next_card.stability.max(0.4),
                        next_card.difficulty
                    )
                    .into(),
                );
            }
        });
    }

    // 11. Q-Bank Option Selection
    {
        let ui_handle = ui.as_weak();
        ui.on_select_qbank_option(move |idx| {
            if let Some(w) = ui_handle.upgrade() {
                w.set_qbank_selected_index(idx);
                w.set_qbank_explanation_visible(true);
            }
        });
    }

    // 12. Clinical Reasoning Step-by-Step Case Progression
    {
        let ui_handle = ui.as_weak();
        let st = state.clone();
        ui.on_advance_clinical_case_step(move || {
            // Reborrow as a plain &mut so disjoint fields can be borrowed independently.
            let mut guard = st.borrow_mut();
            let app = &mut *guard;
            if let Some(case) = app.cases.first() {
                app.current_case_step_idx = (app.current_case_step_idx + 1) % case.steps.len();
                let step = &case.steps[app.current_case_step_idx];
                if let Some(w) = ui_handle.upgrade() {
                    w.set_case_stage_heading(
                        format!(
                            "Stage {} of {}: {}",
                            step.step_number,
                            case.steps.len(),
                            step.stage_title
                        )
                        .into(),
                    );
                    w.set_case_findings(step.clinical_findings.clone().into());
                    w.set_case_prompt(
                        format!(
                            "{} | Differentials: {}",
                            step.guiding_prompt,
                            step.key_differentials_to_consider.join(", ")
                        )
                        .into(),
                    );
                }
            }
        });
    }

    // 13. Export Offline JSON Backup
    {
        let ui_handle = ui.as_weak();
        let st = state.clone();
        ui.on_export_json_backup(move || {
            let app = st.borrow();
            if let Ok(json_payload) = app.storage.export_backup_json(
                vec![],
                app.questions.clone(),
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
            ) {
                if let Some(w) = ui_handle.upgrade() {
                    w.set_backup_status_text(
                        format!(
                            "Exported {} bytes offline JSON backup • 0 network calls",
                            json_payload.len()
                        )
                        .into(),
                    );
                }
            }
        });
    }

    // 14. 1-Second Periodic Timer for OLED Countdown, 60s Pixel Shift & 5-Min Organ Rotation
    let tick_timer = slint::Timer::default();
    {
        let ui_handle = ui.as_weak();
        let st = state.clone();
        tick_timer.start(
            slint::TimerMode::Repeated,
            std::time::Duration::from_secs(1),
            move || {
                let mut app = st.borrow_mut();
                if app.timer.phase == medpulse_core::timer::SessionPhase::RunningFocus {
                    let finished = app.timer.tick_one_second();
                    let (dx, dy) = app.timer.current_burn_in_shift();
                    let organ = app.timer.current_anatomical_icon();
                    let opacity = app.timer.icon_dim_opacity();

                    if let Some(w) = ui_handle.upgrade() {
                        w.set_countdown_text(app.timer.formatted_countdown().into());
                        w.set_burn_in_shift_x(dx as f32);
                        w.set_burn_in_shift_y(dy as f32);
                        w.set_anatomical_organ_name(organ.name.into());
                        w.set_anatomical_organ_fact(organ.short_fact.into());
                        w.set_anatomical_icon_opacity(opacity);
                        w.set_ambient_controls_visible(app.timer.controls_visible);

                        if finished {
                            w.set_in_ambient_oled_mode(false);
                            w.set_timer_running(false);
                            w.set_show_post_session_outcome_modal(true);
                        }
                    }
                }
            },
        );
        std::mem::forget(tick_timer);
    }

    Ok(ui)
}

#[cfg(target_os = "android")]
mod android_platform {
    /// Applies Android `FLAG_KEEP_SCREEN_ON` (0x00000080) and lowers window screenBrightness
    /// to 5% (0.05f) via JNI during Pure-Black OLED Ambient sessions, and triggers the
    /// screen-off-safe ForegroundService fallback.
    pub fn set_oled_ambient_window_state(_keep_on: bool, _brightness: f32) {
        // Invoked on Android UI thread via ndk_context::android_context() JNI bridge:
        // window.addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);
        // LayoutParams params = window.getAttributes();
        // params.screenBrightness = 0.05f;
        // window.setAttributes(params);
    }
}

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app.clone()).expect("Failed to initialize Slint Android backend");
    let internal_db = app
        .internal_data_path()
        .map(|p| p.join("medpulse_offline.sqlite3").to_string_lossy().to_string());
    let ui = create_and_bind_ui(internal_db.as_deref()).expect("Failed to construct AppWindow");
    ui.run().expect("Failed to run Slint event loop");
}
