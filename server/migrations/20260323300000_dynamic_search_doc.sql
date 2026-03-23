CREATE OR REPLACE FUNCTION jsonb_to_tsvector_all(payload JSONB)
RETURNS TSVECTOR
LANGUAGE sql
IMMUTABLE PARALLEL SAFE
AS $$
    SELECT COALESCE(
        to_tsvector('english',
            string_agg(value, ' ')
        ),
        ''::tsvector
    )
    FROM jsonb_each_text(COALESCE(payload, '{}'::jsonb))
$$;

ALTER TABLE frames DROP COLUMN search_doc;

ALTER TABLE frames ADD COLUMN search_doc TSVECTOR
    GENERATED ALWAYS AS (jsonb_to_tsvector_all(payload)) STORED;

CREATE INDEX IF NOT EXISTS idx_frames_search ON frames USING GIN (search_doc);
