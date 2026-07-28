use std::fs;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Context;
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};

struct EmbeddedSkillFile {
    path: &'static str,
    contents: &'static [u8],
}

include!(concat!(env!("OUT_DIR"), "/embedded_skills_files.rs"));

const COMPATIBILITY_JSON: &[u8] = include_bytes!("../release-compatibility.json");
pub const INSTALL_MARKER: &str = ".scitex-release.json";

#[derive(Debug, Clone, serde::Deserialize, Serialize)]
pub struct BundledRelease {
    pub cli_version: String,
    pub skills_sha256: String,
    pub compatibility: Value,
}

impl BundledRelease {
    pub fn current() -> anyhow::Result<Self> {
        Ok(Self {
            cli_version: env!("CARGO_PKG_VERSION").to_string(),
            skills_sha256: skills_sha256(),
            compatibility: serde_json::from_slice(COMPATIBILITY_JSON)
                .context("embedded release compatibility metadata is invalid")?,
        })
    }
}

pub struct MaterializedSkills {
    root: PathBuf,
}

impl MaterializedSkills {
    pub fn create() -> anyhow::Result<Self> {
        let root = create_temp_dir("scitex-skills")?;
        let materialized = Self { root };
        let skills_root = materialized.skills_path();

        for file in EMBEDDED_SKILL_FILES {
            let relative = safe_relative_path(file.path)?;
            let destination = skills_root.join(relative);
            if let Some(parent) = destination.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!(
                        "failed to create embedded Skills directory {}",
                        parent.display()
                    )
                })?;
            }
            fs::write(&destination, file.contents).with_context(|| {
                format!(
                    "failed to materialize embedded Skill {}",
                    destination.display()
                )
            })?;
        }

        let marker = serde_json::to_vec_pretty(&BundledRelease::current()?)?;
        for skill in embedded_skill_names() {
            fs::write(skills_root.join(skill).join(INSTALL_MARKER), &marker)
                .with_context(|| format!("failed to write bundled release marker for {skill}"))?;
        }

        Ok(materialized)
    }

    pub fn skills_path(&self) -> PathBuf {
        self.root.join("skills")
    }
}

impl Drop for MaterializedSkills {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

pub fn embedded_paths() -> impl Iterator<Item = &'static str> {
    EMBEDDED_SKILL_FILES.iter().map(|file| file.path)
}

pub fn embedded_skill_names() -> impl Iterator<Item = &'static str> {
    EMBEDDED_SKILL_FILES
        .iter()
        .filter_map(|file| file.path.strip_suffix("/SKILL.md"))
        .filter(|name| !name.contains('/'))
}

pub fn skills_sha256() -> String {
    let mut hasher = Sha256::new();
    for file in EMBEDDED_SKILL_FILES {
        hasher.update((file.path.len() as u64).to_be_bytes());
        hasher.update(file.path.as_bytes());
        hasher.update((file.contents.len() as u64).to_be_bytes());
        hasher.update(file.contents);
    }
    hex::encode(hasher.finalize())
}

pub(crate) fn create_temp_dir(prefix: &str) -> anyhow::Result<PathBuf> {
    let base = std::env::temp_dir();
    let pid = std::process::id();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    for attempt in 0..100_u32 {
        let path = base.join(format!("{prefix}-{pid}-{timestamp}-{attempt}"));
        match fs::create_dir(&path) {
            Ok(()) => {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
                }
                return Ok(path);
            }
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("failed to create temporary directory {}", path.display())
                })
            }
        }
    }

    anyhow::bail!("failed to allocate a unique temporary directory for {prefix}")
}

fn safe_relative_path(path: &str) -> anyhow::Result<PathBuf> {
    let path = Path::new(path);
    if path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        anyhow::bail!("unsafe embedded Skill path: {}", path.display());
    }
    Ok(path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundles_all_expected_skill_entrypoints() {
        let paths = embedded_paths().collect::<Vec<_>>();
        assert!(!paths.is_empty());
        for name in [
            "scitex-shared",
            "scitex-orders",
            "scitex-templates",
            "scitex-inventory",
            "scitex-admin",
            "scitex-lab",
            "scitex-project",
            "scitex-users",
            "scitex-evo",
            "scitex-experiment",
            "scitex-task",
            "scitex-tashan",
            "scitex-tool",
            "scitex-error-report",
        ] {
            assert!(
                paths.contains(&format!("{name}/SKILL.md").as_str()),
                "missing bundled Skill entrypoint for {name}"
            );
        }
    }

    #[test]
    fn materializes_bundled_skills_and_cleans_up() {
        let materialized = MaterializedSkills::create().unwrap();
        let root = materialized.root.clone();
        assert!(materialized
            .skills_path()
            .join("scitex-shared/SKILL.md")
            .is_file());
        let marker = materialized
            .skills_path()
            .join("scitex-shared")
            .join(INSTALL_MARKER);
        let installed_release: BundledRelease =
            serde_json::from_slice(&fs::read(marker).unwrap()).unwrap();
        assert_eq!(installed_release.cli_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(installed_release.skills_sha256, skills_sha256());
        drop(materialized);
        assert!(!root.exists());
    }

    #[test]
    fn release_metadata_matches_openapi_fixture() {
        let release = BundledRelease::current().unwrap();
        let fixture = include_bytes!("../tests/fixtures/openapi.json");
        let actual = hex::encode(Sha256::digest(fixture));
        assert_eq!(release.compatibility["openapi"]["fixture_sha256"], actual);
        assert_eq!(release.cli_version, env!("CARGO_PKG_VERSION"));
        assert_eq!(release.skills_sha256.len(), 64);
    }

    #[test]
    fn rejects_unsafe_embedded_paths() {
        assert!(safe_relative_path("../SKILL.md").is_err());
        assert!(safe_relative_path("/tmp/SKILL.md").is_err());
    }
}
