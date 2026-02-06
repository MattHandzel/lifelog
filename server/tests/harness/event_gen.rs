//! Deterministic data generators for integration tests.
//!
//! Uses `rand_chacha::ChaCha8Rng` seeded from a u64 for full reproducibility.

#![allow(dead_code)]

use lifelog_proto::Chunk;
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use utils::cas::sha256_hex;

/// Generate a sequence of chunks with deterministic content.
///
/// Each chunk gets sequential offsets and correct SHA256 hashes.
/// The content is pseudo-random but reproducible from the seed.
pub fn generate_chunk_sequence(
    collector_id: &str,
    stream_id: &str,
    session_id: u64,
    n: usize,
    chunk_size: usize,
    seed: u64,
) -> Vec<Chunk> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut offset = 0u64;
    let mut chunks = Vec::with_capacity(n);

    for _ in 0..n {
        let data: Vec<u8> = (0..chunk_size).map(|_| rng.random::<u8>()).collect();
        let hash = sha256_hex(&data);

        chunks.push(Chunk {
            stream: Some(lifelog_proto::StreamIdentity {
                collector_id: collector_id.to_string(),
                stream_id: stream_id.to_string(),
                session_id,
            }),
            offset,
            data,
            hash,
        });

        offset += chunk_size as u64;
    }

    chunks
}

/// Compute the expected final offset for a chunk sequence.
pub fn expected_final_offset(n: usize, chunk_size: usize) -> u64 {
    (n * chunk_size) as u64
}

/// Collect all hashes from a chunk sequence (for CAS verification).
pub fn collect_hashes(chunks: &[Chunk]) -> Vec<String> {
    chunks.iter().map(|c| c.hash.clone()).collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_generation() {
        let a = generate_chunk_sequence("c1", "s1", 1, 5, 64, 42);
        let b = generate_chunk_sequence("c1", "s1", 1, 5, 64, 42);
        assert_eq!(a.len(), b.len());
        for (ca, cb) in a.iter().zip(b.iter()) {
            assert_eq!(ca.data, cb.data);
            assert_eq!(ca.hash, cb.hash);
            assert_eq!(ca.offset, cb.offset);
        }
    }

    #[test]
    fn different_seeds_different_content() {
        let a = generate_chunk_sequence("c1", "s1", 1, 3, 64, 1);
        let b = generate_chunk_sequence("c1", "s1", 1, 3, 64, 2);
        // Very unlikely to be equal with different seeds
        assert_ne!(a[0].data, b[0].data);
    }

    #[test]
    fn offsets_are_sequential() {
        let chunks = generate_chunk_sequence("c1", "s1", 1, 4, 100, 99);
        assert_eq!(chunks[0].offset, 0);
        assert_eq!(chunks[1].offset, 100);
        assert_eq!(chunks[2].offset, 200);
        assert_eq!(chunks[3].offset, 300);
        assert_eq!(expected_final_offset(4, 100), 400);
    }
}
