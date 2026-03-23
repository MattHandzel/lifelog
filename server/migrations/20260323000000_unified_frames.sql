CREATE TABLE IF NOT EXISTS frames (
    id              UUID PRIMARY KEY,
    collector_id    TEXT NOT NULL,
    stream_id       TEXT NOT NULL,
    modality        TEXT NOT NULL,
    time_range      TSTZRANGE NOT NULL,
    t_device        TIMESTAMPTZ,
    t_ingest        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    t_canonical     TIMESTAMPTZ NOT NULL,
    t_end           TIMESTAMPTZ,
    time_quality    TEXT NOT NULL DEFAULT 'unknown',
    blob_hash       TEXT,
    blob_size       INTEGER,
    indexed         BOOLEAN NOT NULL DEFAULT true,
    source_frame_id UUID,
    payload         JSONB NOT NULL DEFAULT '{}',
    search_doc      TSVECTOR GENERATED ALWAYS AS (
        to_tsvector('english',
            COALESCE(payload->>'text', '')       || ' ' ||
            COALESCE(payload->>'url', '')        || ' ' ||
            COALESCE(payload->>'title', '')      || ' ' ||
            COALESCE(payload->>'command', '')    || ' ' ||
            COALESCE(payload->>'window_title', ''))
    ) STORED
);

CREATE INDEX IF NOT EXISTS idx_frames_time_gist    ON frames USING GIST (time_range);
CREATE INDEX IF NOT EXISTS idx_frames_collector    ON frames (collector_id, modality);
CREATE INDEX IF NOT EXISTS idx_frames_modality_t   ON frames (modality, t_canonical);
CREATE INDEX IF NOT EXISTS idx_frames_search       ON frames USING GIN (search_doc);
CREATE INDEX IF NOT EXISTS idx_frames_payload      ON frames USING GIN (payload jsonb_path_ops);
CREATE INDEX IF NOT EXISTS idx_frames_blob         ON frames (blob_hash) WHERE blob_hash IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_frames_source       ON frames (source_frame_id) WHERE source_frame_id IS NOT NULL;
