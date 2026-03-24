DROP INDEX IF EXISTS idx_frames_search;
ALTER TABLE frames DROP COLUMN IF EXISTS search_doc;
DROP FUNCTION IF EXISTS jsonb_to_tsvector_all(JSONB);

CREATE OR REPLACE FUNCTION smart_search_doc(modality TEXT, payload JSONB)
RETURNS TSVECTOR
LANGUAGE sql
IMMUTABLE PARALLEL SAFE
AS $$
    SELECT CASE
        WHEN modality IN ('Processes', 'Hyprland', 'Mouse', 'VectorEmbedding') THEN
            ''::tsvector
        ELSE
            COALESCE(
                setweight(to_tsvector('english', COALESCE(payload->>'text', '') || ' ' || COALESCE(payload->>'content', '') || ' ' || COALESCE(payload->>'transcript', '')), 'A') ||
                setweight(to_tsvector('english', COALESCE(payload->>'title', '') || ' ' || COALESCE(payload->>'url', '') || ' ' || COALESCE(payload->>'command', '') || ' ' || COALESCE(payload->>'application', '')), 'B') ||
                setweight(to_tsvector('english', COALESCE(payload->>'window_title', '')), 'C'),
                ''::tsvector
            )
    END
$$;

ALTER TABLE frames ADD COLUMN search_doc TSVECTOR
    GENERATED ALWAYS AS (smart_search_doc(modality, payload)) STORED;

CREATE INDEX idx_frames_search ON frames USING GIN (search_doc);
