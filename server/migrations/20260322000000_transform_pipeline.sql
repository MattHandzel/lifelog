CREATE TABLE IF NOT EXISTS transcription_records (
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
    model TEXT,
    confidence REAL,
    search_document TSVECTOR GENERATED ALWAYS AS (to_tsvector('english', coalesce(text, ''))) STORED
);

CREATE INDEX IF NOT EXISTS idx_transcription_records_time_range_gist
    ON transcription_records USING GIST (time_range);
CREATE INDEX IF NOT EXISTS idx_transcription_records_search_gin
    ON transcription_records USING GIN (search_document);
CREATE INDEX IF NOT EXISTS idx_transcription_records_collector
    ON transcription_records (collector_id);

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'transform_watermarks' AND column_name = 'origin'
    ) THEN
        ALTER TABLE transform_watermarks
            DROP CONSTRAINT IF EXISTS transform_watermarks_pkey;
        ALTER TABLE transform_watermarks
            ADD COLUMN origin TEXT NOT NULL DEFAULT '*';
        ALTER TABLE transform_watermarks
            ADD PRIMARY KEY (transform_id, origin);
    END IF;
END $$;
