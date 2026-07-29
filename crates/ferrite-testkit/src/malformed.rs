//! Reusable bounded malformed-input corpora and rejection harnesses.

use thiserror::Error;

pub const MAX_CORPUS_CASES: usize = 4_096;
pub const MAX_CASE_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MalformedCase {
    name: String,
    bytes: Vec<u8>,
}

impl MalformedCase {
    pub fn new(name: impl Into<String>, bytes: Vec<u8>) -> Result<Self, MalformedError> {
        let name = name.into();
        if name.is_empty() {
            return Err(MalformedError::EmptyName);
        }
        if bytes.len() > MAX_CASE_BYTES {
            return Err(MalformedError::CaseTooLarge {
                actual: bytes.len(),
                maximum: MAX_CASE_BYTES,
            });
        }
        Ok(Self { name, bytes })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MalformedCorpus {
    cases: Vec<MalformedCase>,
}

impl MalformedCorpus {
    pub fn truncations(valid: &[u8]) -> Result<Self, MalformedError> {
        if valid.len() > MAX_CORPUS_CASES {
            return Err(MalformedError::TooManyCases {
                actual: valid.len(),
                maximum: MAX_CORPUS_CASES,
            });
        }
        let cases = (0..valid.len())
            .map(|length| {
                MalformedCase::new(format!("truncate-at-{length}"), valid[..length].to_vec())
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self { cases })
    }

    pub fn from_cases(cases: Vec<MalformedCase>) -> Result<Self, MalformedError> {
        if cases.len() > MAX_CORPUS_CASES {
            return Err(MalformedError::TooManyCases {
                actual: cases.len(),
                maximum: MAX_CORPUS_CASES,
            });
        }
        Ok(Self { cases })
    }

    pub fn cases(&self) -> &[MalformedCase] {
        &self.cases
    }

    pub fn require_rejected<E>(
        &self,
        mut decode: impl FnMut(&[u8]) -> Result<(), E>,
    ) -> Result<(), AcceptedMalformedInput> {
        for case in &self.cases {
            if decode(case.bytes()).is_ok() {
                return Err(AcceptedMalformedInput {
                    case: case.name.clone(),
                });
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MalformedError {
    #[error("malformed input case name cannot be empty")]
    EmptyName,
    #[error("malformed input has {actual} bytes, exceeding the {maximum}-byte limit")]
    CaseTooLarge { actual: usize, maximum: usize },
    #[error("malformed corpus has {actual} cases, exceeding the {maximum}-case limit")]
    TooManyCases { actual: usize, maximum: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("decoder accepted malformed input case {case:?}")]
pub struct AcceptedMalformedInput {
    pub case: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_strict_prefix_must_be_rejected() {
        let corpus = MalformedCorpus::truncations(b"MAGIC").unwrap();
        corpus
            .require_rejected(|bytes| if bytes == b"MAGIC" { Ok(()) } else { Err(()) })
            .unwrap();
        assert_eq!(corpus.cases().len(), 5);
    }
}
