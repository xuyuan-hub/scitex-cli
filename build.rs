use std::env;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=skills");
    println!("cargo:rerun-if-changed=release-compatibility.json");

    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let skills_root = manifest_dir.join("skills");
    let mut files = Vec::new();
    collect_files(&skills_root, &skills_root, &mut files);
    files.sort_by(|left, right| left.0.cmp(&right.0));

    if files.is_empty() {
        panic!("no files found under {}", skills_root.display());
    }

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let generated_path = out_dir.join("embedded_skills_files.rs");
    let mut generated = fs::File::create(&generated_path)
        .unwrap_or_else(|error| panic!("failed to create {}: {error}", generated_path.display()));

    writeln!(
        generated,
        "const EMBEDDED_SKILL_FILES: &[EmbeddedSkillFile] = &["
    )
    .unwrap();
    for (relative, absolute) in files {
        writeln!(
            generated,
            "    EmbeddedSkillFile {{ path: {relative:?}, contents: include_bytes!({absolute:?}) }},",
            absolute = absolute.to_string_lossy(),
        )
        .unwrap();
    }
    writeln!(generated, "];").unwrap();
}

fn collect_files(root: &Path, current: &Path, files: &mut Vec<(String, PathBuf)>) {
    let entries = fs::read_dir(current)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", current.display()));

    for entry in entries {
        let entry = entry.unwrap_or_else(|error| {
            panic!(
                "failed to read an entry under {}: {error}",
                current.display()
            )
        });
        let path = entry.path();
        let file_type = entry
            .file_type()
            .unwrap_or_else(|error| panic!("failed to inspect {}: {error}", path.display()));

        if file_type.is_dir() {
            collect_files(root, &path, files);
        } else if file_type.is_file() {
            let relative = path
                .strip_prefix(root)
                .unwrap()
                .components()
                .map(|component| component.as_os_str().to_string_lossy())
                .collect::<Vec<_>>()
                .join("/");
            files.push((relative, path));
        }
    }
}
