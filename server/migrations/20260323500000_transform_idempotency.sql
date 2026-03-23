CREATE UNIQUE INDEX IF NOT EXISTS idx_frames_transform_dedup
    ON frames (source_frame_id, stream_id, modality)
    WHERE source_frame_id IS NOT NULL;
