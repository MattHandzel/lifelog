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
            .validate_chunk(0, a, &sha256_hex(a), OffsetPolicy::Resume { allow_offset: 0 })
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
}
