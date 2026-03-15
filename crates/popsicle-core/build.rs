use std::fs;
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("builtin_files.rs");
    let mut out = fs::File::create(&dest_path).unwrap();

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let mut entries: Vec<(String, String)> = Vec::new();

    collect_files(
        &workspace_root.join("skills"),
        &workspace_root.join("skills"),
        ".popsicle/modules/official/skills",
        &mut entries,
    );

    collect_files(
        &workspace_root.join("pipelines"),
        &workspace_root.join("pipelines"),
        ".popsicle/modules/official/pipelines",
        &mut entries,
    );

    // Embed module.yaml for the official module
    let module_yaml = workspace_root.join("module.yaml");
    if module_yaml.exists() {
        let abs = module_yaml.canonicalize().unwrap();
        entries.push((
            ".popsicle/modules/official/module.yaml".into(),
            abs.display().to_string(),
        ));
        println!("cargo:rerun-if-changed={}", abs.display());
    }

    entries.sort_by(|a, b| a.0.cmp(&b.0));

    writeln!(out, "const BUILTIN_FILES: &[EmbeddedFile] = &[").unwrap();
    for (dest, abs_src) in &entries {
        writeln!(out, "    EmbeddedFile {{").unwrap();
        writeln!(out, "        path: {:?},", dest).unwrap();
        writeln!(out, "        content: include_str!({:?}),", abs_src).unwrap();
        writeln!(out, "    }},").unwrap();
        println!("cargo:rerun-if-changed={}", abs_src);
    }
    writeln!(out, "];").unwrap();

    // Also watch the directories themselves for new/deleted files
    rerun_if_dir_changed(&workspace_root.join("skills"));
    rerun_if_dir_changed(&workspace_root.join("pipelines"));
}

fn rerun_if_dir_changed(dir: &Path) {
    println!("cargo:rerun-if-changed={}", dir.display());
    let Ok(read) = fs::read_dir(dir) else {
        return;
    };
    for entry in read {
        let Ok(entry) = entry else { continue };
        if entry.path().is_dir() {
            rerun_if_dir_changed(&entry.path());
        }
    }
}

/// Recursively collect files under `scan_root`, mapping them to `dest_prefix/relative_path`.
fn collect_files(
    dir: &Path,
    scan_root: &Path,
    dest_prefix: &str,
    entries: &mut Vec<(String, String)>,
) {
    let Ok(read) = fs::read_dir(dir) else {
        return;
    };
    for entry in read {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, scan_root, dest_prefix, entries);
        } else {
            let relative = path.strip_prefix(scan_root).unwrap();
            let dest = format!("{}/{}", dest_prefix, relative.display());
            let abs = path.canonicalize().unwrap();
            entries.push((dest, abs.display().to_string()));
        }
    }
}
