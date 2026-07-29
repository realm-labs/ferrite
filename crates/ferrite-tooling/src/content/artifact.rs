use crate::content::model::ArtifactLock;
use anyhow::{Context as _, Result, ensure};
use ferrite_registry::bundle::{FamilyName, Sha1Digest, SourceArtifact};
use ferrite_registry::digest::ContentDigest;
use sha1::{Digest as _, Sha1};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::Path;

pub(crate) fn verify(name: &str, path: &Path, expected: &ArtifactLock) -> Result<SourceArtifact> {
    let file =
        File::open(path).with_context(|| format!("open locked artifact {}", path.display()))?;
    let actual_size = file
        .metadata()
        .with_context(|| format!("read metadata for {}", path.display()))?
        .len();
    ensure!(
        actual_size == expected.size,
        "{name} size drift: expected {}, got {actual_size}",
        expected.size
    );

    let mut input = BufReader::new(file);
    let mut sha1 = Sha1::new();
    let mut blake3 = blake3::Hasher::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let read = input
            .read(&mut buffer)
            .with_context(|| format!("hash {}", path.display()))?;
        if read == 0 {
            break;
        }
        sha1.update(&buffer[..read]);
        blake3.update(&buffer[..read]);
    }
    let actual_sha1 = encode_hex(sha1.finalize().as_slice());
    ensure!(
        actual_sha1 == expected.sha1,
        "{name} SHA-1 drift: expected {}, got {actual_sha1}",
        expected.sha1
    );
    Ok(SourceArtifact::new(
        FamilyName::new(name)?,
        Sha1Digest::new(actual_sha1)?,
        actual_size,
        ContentDigest::from_bytes(*blake3.finalize().as_bytes()),
    ))
}

pub(crate) fn sha1_lines<'a>(values: impl IntoIterator<Item = &'a str>) -> String {
    let mut hasher = Sha1::new();
    for value in values {
        hasher.update(value.as_bytes());
        hasher.update(b"\n");
    }
    encode_hex(hasher.finalize().as_slice())
}

fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn line_digest_is_sorted_by_the_caller_and_newline_terminated() {
        assert_eq!(
            sha1_lines(["minecraft:a", "minecraft:b"]),
            "a748a46b7a22afdb3e5c8ae050853b79c519b7c7"
        );
    }

    #[test]
    fn artifact_verification_fails_closed_on_size_drift() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("artifact.jar");
        fs::write(&path, b"fixture").unwrap();
        let lock = ArtifactLock {
            sha1: encode_hex(Sha1::digest(b"fixture").as_slice()),
            size: 7,
        };
        assert!(verify("fixture", &path, &lock).is_ok());

        let drifted = ArtifactLock {
            sha1: lock.sha1,
            size: 8,
        };
        assert!(verify("fixture", &path, &drifted).is_err());
    }
}
