use crate::models::{
    BackupPayload, Flashcard, FeynmanNote, ImplementationIntention, MoodCheckIn, Specialty,
    StudySessionRecord, UserGoalsAndSettings, VignetteQuestion, WeeklyReflection,
};
use chrono::Utc;
use rusqlite::{params, Connection};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("SQLite error: {0}")]
    Sql(#[from] rusqlite::Error),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

pub struct OfflineStorage {
    conn: Connection,
}

impl OfflineStorage {
    pub fn open_in_memory() -> Result<Self, StorageError> {
        let conn = Connection::open_in_memory()?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    pub fn open_path(path: &str) -> Result<Self, StorageError> {
        let conn = Connection::open(path)?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> Result<(), StorageError> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS kv_store (
                key TEXT PRIMARY KEY,
                json_value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS flashcards (
                id TEXT PRIMARY KEY,
                specialty_id TEXT NOT NULL,
                topic_id TEXT NOT NULL,
                due_epoch_secs INTEGER NOT NULL,
                json_payload TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS study_sessions (
                id TEXT PRIMARY KEY,
                specialty_id TEXT NOT NULL,
                started_epoch_secs INTEGER NOT NULL,
                duration_minutes INTEGER NOT NULL,
                json_payload TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS feynman_notes (
                id TEXT PRIMARY KEY,
                topic_id TEXT NOT NULL,
                json_payload TEXT NOT NULL
            );
            ",
        )?;
        Ok(())
    }

    pub fn save_settings(&self, settings: &UserGoalsAndSettings) -> Result<(), StorageError> {
        let raw = serde_json::to_string(settings)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO kv_store (key, json_value) VALUES ('settings', ?1)",
            params![raw],
        )?;
        Ok(())
    }

    pub fn load_settings(&self) -> Result<UserGoalsAndSettings, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT json_value FROM kv_store WHERE key = 'settings'")?;
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            let raw: String = row.get(0)?;
            Ok(serde_json::from_str(&raw)?)
        } else {
            Ok(UserGoalsAndSettings::default())
        }
    }

    pub fn upsert_flashcard(&self, card: &Flashcard) -> Result<(), StorageError> {
        let raw = serde_json::to_string(card)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO flashcards (id, specialty_id, topic_id, due_epoch_secs, json_payload)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![card.id, card.specialty_id, card.topic_id, card.due_epoch_secs, raw],
        )?;
        Ok(())
    }

    pub fn list_flashcards(&self) -> Result<Vec<Flashcard>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT json_payload FROM flashcards ORDER BY due_epoch_secs ASC")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut out = Vec::new();
        for r in rows {
            out.push(serde_json::from_str(&r?)?);
        }
        Ok(out)
    }

    pub fn record_session(&self, session: &StudySessionRecord) -> Result<(), StorageError> {
        let raw = serde_json::to_string(session)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO study_sessions (id, specialty_id, started_epoch_secs, duration_minutes, json_payload)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                session.id,
                session.specialty_id,
                session.started_epoch_secs,
                session.duration_minutes,
                raw
            ],
        )?;
        Ok(())
    }

    pub fn list_sessions(&self) -> Result<Vec<StudySessionRecord>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT json_payload FROM study_sessions ORDER BY started_epoch_secs DESC")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut out = Vec::new();
        for r in rows {
            out.push(serde_json::from_str(&r?)?);
        }
        Ok(out)
    }

    pub fn upsert_feynman_note(&self, note: &FeynmanNote) -> Result<(), StorageError> {
        let raw = serde_json::to_string(note)?;
        self.conn.execute(
            "INSERT OR REPLACE INTO feynman_notes (id, topic_id, json_payload) VALUES (?1, ?2, ?3)",
            params![note.id, note.topic_id, raw],
        )?;
        Ok(())
    }

    pub fn list_feynman_notes(&self) -> Result<Vec<FeynmanNote>, StorageError> {
        let mut stmt = self
            .conn
            .prepare("SELECT json_payload FROM feynman_notes ORDER BY id ASC")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let mut out = Vec::new();
        for r in rows {
            out.push(serde_json::from_str(&r?)?);
        }
        Ok(out)
    }

    pub fn export_backup_json(
        &self,
        custom_specialties: Vec<Specialty>,
        questions: Vec<VignetteQuestion>,
        mood_logs: Vec<MoodCheckIn>,
        intentions: Vec<ImplementationIntention>,
        reflections: Vec<WeeklyReflection>,
        favorite_quote_ids: Vec<String>,
        seen_quote_ids: Vec<String>,
    ) -> Result<String, StorageError> {
        let payload = BackupPayload {
            schema_version: 1,
            exported_at: Utc::now(),
            settings: self.load_settings()?,
            custom_specialties,
            flashcards: self.list_flashcards()?,
            questions,
            sessions: self.list_sessions()?,
            notes: self.list_feynman_notes()?,
            mood_logs,
            intentions,
            reflections,
            favorite_quote_ids,
            seen_quote_ids,
        };
        Ok(serde_json::to_string_pretty(&payload)?)
    }

    pub fn import_backup_json(&self, json_str: &str) -> Result<BackupPayload, StorageError> {
        let payload: BackupPayload = serde_json::from_str(json_str)?;
        self.save_settings(&payload.settings)?;
        for card in &payload.flashcards {
            self.upsert_flashcard(card)?;
        }
        for session in &payload.sessions {
            self.record_session(session)?;
        }
        for note in &payload.notes {
            self.upsert_feynman_note(note)?;
        }
        Ok(payload)
    }
}
