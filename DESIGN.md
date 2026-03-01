# Design Notes

## Search Previews (2026-03-01)

### Scope

- Enhance search results with:
  - text snippets around query terms,
  - highlighted term matches,
  - lightweight thumbnails for image modalities.

### Architecture Decisions

- Keep `query_timeline` as the primary key retrieval path.
- Add an interface backend enrichment command:
  - `get_frame_data_thumbnails(keys)` returns frame metadata plus downscaled image previews.
- Perform snippet construction in the frontend from enriched frame fields.
  - Rationale: avoids proto churn and allows UI-level tuning of snippet length and highlight behavior.

### Data Flow

1. UI calls `query_timeline` with text query.
2. UI calls `get_frame_data_thumbnails` for returned keys.
3. UI builds `SearchResult` models with:
   - `snippet`,
   - `highlightTerms`,
   - `preview` (thumbnail data URL for image frames).
4. `ResultCard` renders lazy thumbnail + highlighted snippet.

### Validation

- Added `SearchDashboard` UI tests for:
  - snippet highlighting,
  - thumbnail rendering.
- Verified with `just test-ui` and `just validate`.
