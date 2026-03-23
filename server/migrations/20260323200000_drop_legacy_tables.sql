-- Phase 4: Drop legacy per-modality tables now that all data lives in the unified frames table.

DROP TABLE IF EXISTS screen_records;
DROP TABLE IF EXISTS browser_records;
DROP TABLE IF EXISTS ocr_records;
DROP TABLE IF EXISTS audio_records;
DROP TABLE IF EXISTS clipboard_records;
DROP TABLE IF EXISTS shell_history_records;
DROP TABLE IF EXISTS keystroke_records;
DROP TABLE IF EXISTS transcription_records;
