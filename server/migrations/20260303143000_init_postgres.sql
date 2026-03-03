CREATE TABLE IF NOT EXISTS upload_chunks (
    id TEXT PRIMARY KEY,
    collector_id TEXT NOT NULL,
    stream_id TEXT NOT NULL,
    session_id BIGINT NOT NULL,
    "offset" BIGINT NOT NULL,
    length INTEGER NOT NULL,
    hash TEXT NOT NULL,
    indexed BOOLEAN NOT NULL DEFAULT FALSE,
    frame_uuid TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (collector_id, stream_id, session_id, "offset")
);

CREATE INDEX IF NOT EXISTS idx_upload_chunks_resume
    ON upload_chunks (collector_id, stream_id, session_id, "offset" DESC);
CREATE INDEX IF NOT EXISTS idx_upload_chunks_frame_uuid
    ON upload_chunks (frame_uuid);

CREATE TABLE IF NOT EXISTS catalog (
    origin TEXT PRIMARY KEY,
    collector_id TEXT NOT NULL,
    modality TEXT NOT NULL,
    stream_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_catalog_collector_modality
    ON catalog (collector_id, modality);

CREATE TABLE IF NOT EXISTS transform_watermarks (
    transform_id TEXT PRIMARY KEY,
    cursor_value TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS screen_records (
    id UUID PRIMARY KEY,
    collector_id TEXT NOT NULL,
    stream_id TEXT NOT NULL,
    time_range TSTZRANGE NOT NULL,
    t_device TIMESTAMPTZ NOT NULL,
    t_ingest TIMESTAMPTZ,
    t_canonical TIMESTAMPTZ,
    t_end TIMESTAMPTZ,
    time_quality TEXT,
    width BIGINT NOT NULL,
    height BIGINT NOT NULL,
    blob_hash TEXT NOT NULL,
    blob_size BIGINT NOT NULL,
    mime_type TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_screen_records_time_range_gist
    ON screen_records USING GIST (time_range);

CREATE TABLE IF NOT EXISTS browser_records (
    id UUID PRIMARY KEY,
    collector_id TEXT NOT NULL,
    stream_id TEXT NOT NULL,
    time_range TSTZRANGE NOT NULL,
    t_device TIMESTAMPTZ NOT NULL,
    t_ingest TIMESTAMPTZ,
    t_canonical TIMESTAMPTZ,
    t_end TIMESTAMPTZ,
    time_quality TEXT,
    url TEXT NOT NULL,
    title TEXT NOT NULL,
    visit_count BIGINT NOT NULL DEFAULT 0,
    search_document TSVECTOR GENERATED ALWAYS AS (
        setweight(to_tsvector('english', coalesce(url, '')), 'A') ||
        setweight(to_tsvector('english', coalesce(title, '')), 'B')
    ) STORED
);

CREATE INDEX IF NOT EXISTS idx_browser_records_time_range_gist
    ON browser_records USING GIST (time_range);
CREATE INDEX IF NOT EXISTS idx_browser_records_search_gin
    ON browser_records USING GIN (search_document);

CREATE TABLE IF NOT EXISTS ocr_records (
    id UUID PRIMARY KEY,
    collector_id TEXT NOT NULL,
    stream_id TEXT NOT NULL,
    source_frame_uuid UUID,
    time_range TSTZRANGE NOT NULL,
    t_device TIMESTAMPTZ NOT NULL,
    t_ingest TIMESTAMPTZ,
    t_canonical TIMESTAMPTZ,
    t_end TIMESTAMPTZ,
    time_quality TEXT,
    text TEXT NOT NULL,
    search_document TSVECTOR GENERATED ALWAYS AS (to_tsvector('english', coalesce(text, ''))) STORED
);

CREATE INDEX IF NOT EXISTS idx_ocr_records_time_range_gist
    ON ocr_records USING GIST (time_range);
CREATE INDEX IF NOT EXISTS idx_ocr_records_search_gin
    ON ocr_records USING GIN (search_document);

CREATE TABLE IF NOT EXISTS audio_records (
    id UUID PRIMARY KEY,
    collector_id TEXT NOT NULL,
    stream_id TEXT NOT NULL,
    time_range TSTZRANGE NOT NULL,
    t_device TIMESTAMPTZ NOT NULL,
    t_ingest TIMESTAMPTZ,
    t_canonical TIMESTAMPTZ,
    t_end TIMESTAMPTZ,
    time_quality TEXT,
    blob_hash TEXT NOT NULL,
    blob_size BIGINT NOT NULL,
    codec TEXT NOT NULL,
    sample_rate BIGINT,
    channels BIGINT,
    duration_secs DOUBLE PRECISION NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_audio_records_time_range_gist
    ON audio_records USING GIST (time_range);

CREATE TABLE IF NOT EXISTS clipboard_records (
    id UUID PRIMARY KEY,
    collector_id TEXT NOT NULL,
    stream_id TEXT NOT NULL,
    time_range TSTZRANGE NOT NULL,
    t_device TIMESTAMPTZ NOT NULL,
    t_ingest TIMESTAMPTZ,
    t_canonical TIMESTAMPTZ,
    t_end TIMESTAMPTZ,
    time_quality TEXT,
    text TEXT,
    blob_hash TEXT,
    blob_size BIGINT,
    mime_type TEXT,
    search_document TSVECTOR GENERATED ALWAYS AS (to_tsvector('english', coalesce(text, ''))) STORED
);

CREATE INDEX IF NOT EXISTS idx_clipboard_records_time_range_gist
    ON clipboard_records USING GIST (time_range);
CREATE INDEX IF NOT EXISTS idx_clipboard_records_search_gin
    ON clipboard_records USING GIN (search_document);

CREATE TABLE IF NOT EXISTS shell_history_records (
    id UUID PRIMARY KEY,
    collector_id TEXT NOT NULL,
    stream_id TEXT NOT NULL,
    time_range TSTZRANGE NOT NULL,
    t_device TIMESTAMPTZ NOT NULL,
    t_ingest TIMESTAMPTZ,
    t_canonical TIMESTAMPTZ,
    t_end TIMESTAMPTZ,
    time_quality TEXT,
    command TEXT NOT NULL,
    working_dir TEXT,
    exit_code INTEGER,
    search_document TSVECTOR GENERATED ALWAYS AS (to_tsvector('english', coalesce(command, ''))) STORED
);

CREATE INDEX IF NOT EXISTS idx_shell_history_records_time_range_gist
    ON shell_history_records USING GIST (time_range);
CREATE INDEX IF NOT EXISTS idx_shell_history_records_search_gin
    ON shell_history_records USING GIN (search_document);

CREATE TABLE IF NOT EXISTS keystroke_records (
    id UUID PRIMARY KEY,
    collector_id TEXT NOT NULL,
    stream_id TEXT NOT NULL,
    time_range TSTZRANGE NOT NULL,
    t_device TIMESTAMPTZ NOT NULL,
    t_ingest TIMESTAMPTZ,
    t_canonical TIMESTAMPTZ,
    t_end TIMESTAMPTZ,
    time_quality TEXT,
    text TEXT NOT NULL,
    application TEXT,
    window_title TEXT,
    search_document TSVECTOR GENERATED ALWAYS AS (to_tsvector('english', coalesce(text, ''))) STORED
);

CREATE INDEX IF NOT EXISTS idx_keystroke_records_time_range_gist
    ON keystroke_records USING GIST (time_range);
CREATE INDEX IF NOT EXISTS idx_keystroke_records_search_gin
    ON keystroke_records USING GIN (search_document);
