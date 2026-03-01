# Plan: Search Previews (Snippets & Thumbnails)

## Objective
Enhance search results with rich previews, including OCR text snippets and image thumbnails.

## Context
From `SPEC.md` Section 11.1:
- Result list with previews: screen thumbnails, OCR snippets, URL/title snippets, clipboard/command text.

## Phase 1: Research & Strategy
1. **Data Availability:** Confirm that `SearchResponse` from the backend includes necessary metadata or `blob_hash` for thumbnails.
2. **Snippet Logic:** Determine if the backend should generate text snippets (BM25 highlighted) or if the frontend should perform the highlighting. (Recommendation: Backend-generated snippets for performance).
3. **Lazy Loading:** Plan the React component for lazy-loading thumbnails as the user scrolls.

## Phase 2: Execution
1. **Backend Enhancement:** Update the query engine to generate context snippets around search terms for text results.
2. **Thumbnail API:** Ensure a gRPC or HTTP endpoint exists to fetch a downscaled "thumbnail" version of an image blob.
3. **UI Components:**
    - Create a `SearchResultCard` component.
    - Implement snippet rendering with term highlighting.
    - Add a `Thumbnail` component with a loading skeleton.
4. **Search Integration:** Update the main Search dashboard to use these new rich result cards.

## Phase 3: Verification
1. Run `just test-ui`.
2. Verify that searching for a term returns results with the term highlighted in a text snippet.
3. Verify that screen capture results show a recognizable thumbnail.

## AI Token-Efficient Guidelines
- Use `just diff-digest` to summarize changes.
- Use `tools/ai/run_and_digest.sh` for build verification.

## Model Recommendation
**Gemini 3-Flash-Preview** (Standard UI/Backend glue work).
