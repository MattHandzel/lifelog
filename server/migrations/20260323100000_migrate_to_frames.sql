-- Phase 4: Migrate existing per-modality Postgres data into the unified frames table.
-- Each INSERT uses ON CONFLICT (id) DO NOTHING so this migration is idempotent.
-- Guards each migration with a table-existence check so this is safe on fresh deployments.

DO $$
BEGIN

-- 1. screen_records
IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'screen_records') THEN
    INSERT INTO frames (id, collector_id, stream_id, modality, time_range,
        t_device, t_ingest, t_canonical, t_end, time_quality,
        blob_hash, blob_size, indexed, payload)
    SELECT id, collector_id, stream_id, 'Screen', time_range,
        t_device, COALESCE(t_ingest, NOW()), COALESCE(t_canonical, t_device), t_end,
        COALESCE(time_quality, 'unknown'),
        blob_hash, blob_size::integer, true,
        jsonb_build_object('width', width, 'height', height, 'mime_type', mime_type)
    FROM screen_records ON CONFLICT (id) DO NOTHING;
    RAISE NOTICE 'Migrated screen_records to frames';
END IF;

-- 2. browser_records
IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'browser_records') THEN
    INSERT INTO frames (id, collector_id, stream_id, modality, time_range,
        t_device, t_ingest, t_canonical, t_end, time_quality,
        indexed, payload)
    SELECT id, collector_id, stream_id, 'Browser', time_range,
        t_device, COALESCE(t_ingest, NOW()), COALESCE(t_canonical, t_device), t_end,
        COALESCE(time_quality, 'unknown'),
        true,
        jsonb_build_object('url', url, 'title', title, 'visit_count', visit_count)
    FROM browser_records ON CONFLICT (id) DO NOTHING;
    RAISE NOTICE 'Migrated browser_records to frames';
END IF;

-- 3. ocr_records
IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'ocr_records') THEN
    INSERT INTO frames (id, collector_id, stream_id, modality, time_range,
        t_device, t_ingest, t_canonical, t_end, time_quality,
        indexed, source_frame_id, payload)
    SELECT id, collector_id, stream_id, 'Ocr', time_range,
        t_device, COALESCE(t_ingest, NOW()), COALESCE(t_canonical, t_device), t_end,
        COALESCE(time_quality, 'unknown'),
        true, source_frame_uuid,
        jsonb_build_object('text', text)
    FROM ocr_records ON CONFLICT (id) DO NOTHING;
    RAISE NOTICE 'Migrated ocr_records to frames';
END IF;

-- 4. audio_records
IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'audio_records') THEN
    INSERT INTO frames (id, collector_id, stream_id, modality, time_range,
        t_device, t_ingest, t_canonical, t_end, time_quality,
        blob_hash, blob_size, indexed, payload)
    SELECT id, collector_id, stream_id, 'Audio', time_range,
        t_device, COALESCE(t_ingest, NOW()), COALESCE(t_canonical, t_device), t_end,
        COALESCE(time_quality, 'unknown'),
        blob_hash, blob_size::integer, true,
        jsonb_build_object('codec', codec, 'sample_rate', sample_rate,
                           'channels', channels, 'duration_secs', duration_secs)
    FROM audio_records ON CONFLICT (id) DO NOTHING;
    RAISE NOTICE 'Migrated audio_records to frames';
END IF;

-- 5. clipboard_records
IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'clipboard_records') THEN
    INSERT INTO frames (id, collector_id, stream_id, modality, time_range,
        t_device, t_ingest, t_canonical, t_end, time_quality,
        blob_hash, blob_size, indexed, payload)
    SELECT id, collector_id, stream_id, 'Clipboard', time_range,
        t_device, COALESCE(t_ingest, NOW()), COALESCE(t_canonical, t_device), t_end,
        COALESCE(time_quality, 'unknown'),
        blob_hash, blob_size::integer, true,
        jsonb_build_object('text', text, 'mime_type', mime_type)
    FROM clipboard_records ON CONFLICT (id) DO NOTHING;
    RAISE NOTICE 'Migrated clipboard_records to frames';
END IF;

-- 6. shell_history_records
IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'shell_history_records') THEN
    INSERT INTO frames (id, collector_id, stream_id, modality, time_range,
        t_device, t_ingest, t_canonical, t_end, time_quality,
        indexed, payload)
    SELECT id, collector_id, stream_id, 'ShellHistory', time_range,
        t_device, COALESCE(t_ingest, NOW()), COALESCE(t_canonical, t_device), t_end,
        COALESCE(time_quality, 'unknown'),
        true,
        jsonb_build_object('command', command, 'working_dir', working_dir,
                           'exit_code', exit_code)
    FROM shell_history_records ON CONFLICT (id) DO NOTHING;
    RAISE NOTICE 'Migrated shell_history_records to frames';
END IF;

-- 7. keystroke_records
IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'keystroke_records') THEN
    INSERT INTO frames (id, collector_id, stream_id, modality, time_range,
        t_device, t_ingest, t_canonical, t_end, time_quality,
        indexed, payload)
    SELECT id, collector_id, stream_id, 'Keystroke', time_range,
        t_device, COALESCE(t_ingest, NOW()), COALESCE(t_canonical, t_device), t_end,
        COALESCE(time_quality, 'unknown'),
        true,
        jsonb_build_object('text', text, 'application', application,
                           'window_title', window_title)
    FROM keystroke_records ON CONFLICT (id) DO NOTHING;
    RAISE NOTICE 'Migrated keystroke_records to frames';
END IF;

-- 8. transcription_records
IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'transcription_records') THEN
    INSERT INTO frames (id, collector_id, stream_id, modality, time_range,
        t_device, t_ingest, t_canonical, t_end, time_quality,
        indexed, source_frame_id, payload)
    SELECT id, collector_id, stream_id, 'Transcription', time_range,
        t_device, COALESCE(t_ingest, NOW()), COALESCE(t_canonical, t_device), t_end,
        COALESCE(time_quality, 'unknown'),
        true, source_frame_uuid,
        jsonb_build_object('text', text, 'model', model, 'confidence', confidence)
    FROM transcription_records ON CONFLICT (id) DO NOTHING;
    RAISE NOTICE 'Migrated transcription_records to frames';
END IF;

END $$;
