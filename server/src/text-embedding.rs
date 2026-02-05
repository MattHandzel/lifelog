use anyhow::Result;
use rust_bert::pipelines::sentence_embeddings::{
    SentenceEmbeddingsBuilder, SentenceEmbeddingsModel, SentenceEmbeddingsModelType,
};

/// A struct to hold the sentence embedding model.
pub struct TextEmbedder {
    model: SentenceEmbeddingsModel,
}

impl TextEmbedder {
    /// Creates a new `TextEmbedder` by downloading and initializing the model.
    pub fn new() -> Result<Self> {
        let model = SentenceEmbeddingsBuilder::remote(SentenceEmbeddingsModelType::AllMiniLmL6V2)
            .create_model()?;
        Ok(Self { model })
    }

    /// Embeds the provided text into a vector of f32 values.
    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // The encode method returns a Vec<Vec<f32>> even for a single sentence.
        let embeddings = self.model.encode(&[text])?;
        // Return the embedding vector for the first (and only) sentence.
        Ok(embeddings.into_iter().next().unwrap())
    }
}

/// Computes the cosine similarity between two vectors.
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot / (norm_a * norm_b)
    }
}

/// Given a query and a list of documents, retrieves the document most similar to the query.
/// Returns the document and its similarity score.
pub fn retrieve_most_similar(
    embedder: &TextEmbedder,
    query: &str,
    documents: &[&str],
) -> Result<Option<(String, f32)>> {
    // Embed the query.
    let query_embedding = embedder.embed(query)?;
    let mut best_similarity = -1.0;
    let mut best_doc: Option<String> = None;

    // Loop through each document, compute its embedding and the similarity.
    for &doc in documents {
        let doc_embedding = embedder.embed(doc)?;
        let similarity = cosine_similarity(&query_embedding, &doc_embedding);
        if similarity > best_similarity {
            best_similarity = similarity;
            best_doc = Some(doc.to_string());
        }
    }
    Ok(best_doc.map(|doc| (doc, best_similarity)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    /// Test that the cosine similarity of a vector with itself is 1.0.
    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        let similarity = cosine_similarity(&a, &b);
        assert_relative_eq!(similarity, 1.0, epsilon = 1e-5);
    }

    /// Test that the cosine similarity of orthogonal vectors is 0.0.
    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0];
        let b = vec![0.0, 1.0];
        let similarity = cosine_similarity(&a, &b);
        assert_relative_eq!(similarity, 0.0, epsilon = 1e-5);
    }

    /// Test that the embedding produced has the expected dimension.
    /// For the AllMiniLmL6V2 model, the expected dimension is 384.
    #[test]
    fn test_embedding_dimension() -> Result<()> {
        let embedder = TextEmbedder::new()?;
        let embedding = embedder.embed("Test sentence")?;
        assert_eq!(embedding.len(), 384, "Embedding dimension should be 384.");
        Ok(())
    }

    /// Test the retrieval functionality by ensuring that the document most similar to a query
    /// is correctly identified.
    #[test]
    fn test_retrieve_most_similar() -> Result<()> {
        let embedder = TextEmbedder::new()?;
        let documents = vec![
            "The cat sat on the mat.",
            "A quick brown fox jumps over the lazy dog.",
            "Rust programming language is fast and reliable.",
        ];
        let query = "I love systems programming in Rust.";
        let result = retrieve_most_similar(&embedder, query, &documents)?;
        assert!(result.is_some(), "Should find a similar document.");
        let (doc, similarity) = result.unwrap();
        // We expect that the Rust-related document is the most similar.
        assert!(
            doc.contains("Rust"),
            "The retrieved document should be about Rust."
        );
        // Expect the similarity to be reasonably high.
        assert!(
            similarity > 0.5,
            "The similarity score should be reasonably high."
        );
        Ok(())
    }

    /// Test that when an empty list of documents is provided, no document is returned.
    #[test]
    fn test_retrieve_no_documents() -> Result<()> {
        let embedder = TextEmbedder::new()?;
        let documents: Vec<&str> = vec![];
        let query = "Any query.";
        let result = retrieve_most_similar(&embedder, query, &documents)?;
        assert!(
            result.is_none(),
            "Should return None when no documents are provided."
        );
        Ok(())
    }
}
