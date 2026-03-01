use fd_core::{emitter::emit_document, parser::parse_document};
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let args: Vec<String> = env::args().collect();
    let paths: Vec<PathBuf> = if args.len() > 1 {
        args[1..].iter().map(PathBuf::from).collect()
    } else {
        // Glob for .fd files in examples/ and docs/
        let mut paths = Vec::new();
        for dir in ["examples", "docs/design", "crates/fd-core/tests/fixtures"] {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map_or(false, |e| e == "fd") {
                        paths.push(path);
                    }
                }
            }
            // Recurse one level into subdirectories
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let sub = entry.path();
                    if sub.is_dir() {
                        if let Ok(sub_entries) = fs::read_dir(&sub) {
                            for sub_entry in sub_entries.flatten() {
                                let path = sub_entry.path();
                                if path.extension().map_or(false, |e| e == "fd") {
                                    paths.push(path);
                                }
                            }
                        }
                    }
                }
            }
        }
        paths
    };

    let mut migrated = 0;
    let mut skipped = 0;

    for path in &paths {
        let input = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("SKIP {}: {}", path.display(), e);
                skipped += 1;
                continue;
            }
        };

        // Only migrate files that still use label:
        if !input.contains("label:") {
            skipped += 1;
            continue;
        }

        match parse_document(&input) {
            Ok(graph) => {
                let output = emit_document(&graph);
                if let Err(e) = fs::write(path, &output) {
                    eprintln!("ERROR writing {}: {}", path.display(), e);
                } else {
                    migrated += 1;
                    println!("✓ {}", path.display());
                }
            }
            Err(e) => {
                eprintln!("PARSE ERROR {}: {}", path.display(), e);
                skipped += 1;
            }
        }
    }

    println!("\nMigrated: {}, Skipped: {}", migrated, skipped);
}
