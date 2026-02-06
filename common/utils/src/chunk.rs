use crate::cas::sha256_hex;

#[derive(Debug, Clone, Copy)]
pub enum OffsetPolicy {
    Strict,
    Resume { allow_offset: u64 },
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ChunkError {
    #[error("empty chunk is not allowed")]
    EmptyChunk,
    #[error("hash mismatch: expected {expected}, actual {actual}")]
    HashMismatch { expected: String, actual: String },
    #[error("offset overlap: expected {expected}, actual {actual}")]
    OffsetOverlap { expected: u64, actual: u64 },
    #[error("offset gap: expected {expected}, actual {actual}")]
    OffsetGap { expected: u64, actual: u64 },
}

/// Validates chunk offsets and hashes for resumable upload.
#[derive(Debug, Clone)]
pub struct ChunkOffsetValidator {
    next_offset: u64,
}

impl ChunkOffsetValidator {
    pub fn new(start_offset: u64) -> Self {
        Self {
            next_offset: start_offset,
        }
    }

    pub fn next_offset(&self) -> u64 {
        self.next_offset
    }

    pub fn validate_chunk(
        &mut self,
        offset: u64,
        bytes: &[u8],
        declared_hash: &str,
        policy: OffsetPolicy,
    ) -> Result<u64, ChunkError> {
        if bytes.is_empty() {
            return Err(ChunkError::EmptyChunk);
        }

        let actual_hash = sha256_hex(bytes);
        if actual_hash != declared_hash {
            return Err(ChunkError::HashMismatch {
                expected: declared_hash.to_string(),
                actual: actual_hash,
            });
        }

        let expected = self.next_offset;
        if offset == expected {
            self.next_offset = expected + bytes.len() as u64;
            return Ok(self.next_offset);
        }

        match policy {
            OffsetPolicy::Strict => {
                if offset > expected {
                    Err(ChunkError::OffsetGap {
                        expected,
                        actual: offset,
                    })
                } else {
                    Err(ChunkError::OffsetOverlap {
                        expected,
                        actual: offset,
                    })
                }
            }
            OffsetPolicy::Resume { allow_offset } => {
                if offset == allow_offset {
                    self.next_offset = offset + bytes.len() as u64;
                    Ok(self.next_offset)
                } else if offset > expected {
                    Err(ChunkError::OffsetGap {
                        expected,
                        actual: offset,
                    })
                } else {
                    Err(ChunkError::OffsetOverlap {
                        expected,
                        actual: offset,
                    })
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn ut_040_sequential_offsets_advance() {
        let mut v = ChunkOffsetValidator::new(0);
        let a = b"abc";
        let b = b"defg";

        let next = v
            .validate_chunk(0, a, &sha256_hex(a), OffsetPolicy::Strict)
            .unwrap();
        assert_eq!(next, 3);

        let next = v
            .validate_chunk(3, b, &sha256_hex(b), OffsetPolicy::Strict)
            .unwrap();
        assert_eq!(next, 7);
    }

    #[test]
    fn ut_040_rejects_gap_and_overlap() {
        let mut v = ChunkOffsetValidator::new(0);
        let a = b"abc";
        v.validate_chunk(0, a, &sha256_hex(a), OffsetPolicy::Strict)
            .unwrap();

        let b = b"def";
        let gap = v.validate_chunk(4, b, &sha256_hex(b), OffsetPolicy::Strict);
        assert_eq!(
            gap.unwrap_err(),
            ChunkError::OffsetGap {
                expected: 3,
                actual: 4
            }
        );

        let overlap = v.validate_chunk(2, b, &sha256_hex(b), OffsetPolicy::Strict);
        assert_eq!(
            overlap.unwrap_err(),
            ChunkError::OffsetOverlap {
                expected: 3,
                actual: 2
            }
        );
    }

    #[test]
    fn ut_040_allows_explicit_resume_offset() {
        let mut v = ChunkOffsetValidator::new(0);
        let a = b"abc";
        v.validate_chunk(0, a, &sha256_hex(a), OffsetPolicy::Strict)
            .unwrap();

        let resume = v
            .validate_chunk(
                0,
                a,
                &sha256_hex(a),
                OffsetPolicy::Resume { allow_offset: 0 },
            )
            .unwrap();
        assert_eq!(resume, 3);
    }

    #[test]
    fn ut_040_rejects_hash_mismatch() {
        let mut v = ChunkOffsetValidator::new(0);
        let a = b"abc";
        let err = v
            .validate_chunk(0, a, "bad-hash", OffsetPolicy::Strict)
            .unwrap_err();
        assert_eq!(
            err,
            ChunkError::HashMismatch {
                expected: "bad-hash".to_string(),
                actual: sha256_hex(a)
            }
        );
    }

    mod proptests {
        use super::*;
        use proptest::prelude::*;

        // Any sequence of non-empty chunks at correct offsets must advance the validator.
        proptest! {
            #![proptest_config(ProptestConfig::with_cases(256))]

            #[test]
            fn prop_sequential_offsets_always_advance(
                chunks in prop::collection::vec(prop::collection::vec(1u8..=255, 1..=1024), 1..=20),
            ) {
                let mut v = ChunkOffsetValidator::new(0);
                let mut expected_offset = 0u64;
                for chunk in &chunks {
                    let hash = sha256_hex(chunk);
                    let next = v.validate_chunk(expected_offset, chunk, &hash, OffsetPolicy::Strict).unwrap();
                    expected_offset += chunk.len() as u64;
                    prop_assert_eq!(next, expected_offset);
                }
                prop_assert_eq!(v.next_offset(), expected_offset);
            }

            #[test]
            fn prop_gap_always_detected(
                data in prop::collection::vec(1u8..=255, 1..=512),
                gap in 1u64..=1000,
            ) {
                let mut v = ChunkOffsetValidator::new(0);
                let hash = sha256_hex(&data);
                v.validate_chunk(0, &data, &hash, OffsetPolicy::Strict).unwrap();

                let bad_offset = v.next_offset() + gap;
                let bad_data = b"more";
                let bad_hash = sha256_hex(bad_data.as_slice());
                let err = v.validate_chunk(bad_offset, bad_data, &bad_hash, OffsetPolicy::Strict);
                let is_gap = matches!(err, Err(ChunkError::OffsetGap { .. }));
                prop_assert!(is_gap, "expected OffsetGap, got {:?}", err);
            }

            #[test]
            fn prop_overlap_always_detected(
                data in prop::collection::vec(1u8..=255, 2..=512),
            ) {
                let mut v = ChunkOffsetValidator::new(0);
                let hash = sha256_hex(&data);
                v.validate_chunk(0, &data, &hash, OffsetPolicy::Strict).unwrap();

                // Overlap: use offset 0 again (which is < next_offset since data.len() >= 2)
                let more = b"x";
                let more_hash = sha256_hex(more.as_slice());
                let err = v.validate_chunk(0, more, &more_hash, OffsetPolicy::Strict);
                let is_overlap = matches!(err, Err(ChunkError::OffsetOverlap { .. }));
                prop_assert!(is_overlap, "expected OffsetOverlap, got {:?}", err);
            }

            #[test]
            fn prop_resume_allows_retransmit(
                data in prop::collection::vec(1u8..=255, 1..=512),
            ) {
                let mut v = ChunkOffsetValidator::new(0);
                let hash = sha256_hex(&data);
                let next = v.validate_chunk(0, &data, &hash, OffsetPolicy::Strict).unwrap();

                // Resume at offset 0 should succeed
                let next2 = v.validate_chunk(0, &data, &hash, OffsetPolicy::Resume { allow_offset: 0 }).unwrap();
                prop_assert_eq!(next, next2);
            }

            #[test]
            fn prop_bad_hash_always_rejected(
                data in prop::collection::vec(1u8..=255, 1..=512),
                bad_suffix in "[a-f0-9]{4}",
            ) {
                let mut v = ChunkOffsetValidator::new(0);
                let real_hash = sha256_hex(&data);
                // Create a hash that differs from the real one
                let bad_hash = format!("0000{}{}", &bad_suffix, &real_hash[8..]);
                prop_assert_ne!(&bad_hash, &real_hash, "Extremely unlikely collision");
                let err = v.validate_chunk(0, &data, &bad_hash, OffsetPolicy::Strict);
                let is_mismatch = matches!(err, Err(ChunkError::HashMismatch { .. }));
                prop_assert!(is_mismatch, "expected HashMismatch, got {:?}", err);
            }
        }
    }
}
