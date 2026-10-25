use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-env-changed=FOLISIFT_BUILD");
    println!("cargo:rerun-if-changed=build.rs");

    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo = manifest.join("../..");
    let git_head = repo.join(".git/HEAD");
    if git_head.exists() {
        println!("cargo:rerun-if-changed={}", git_head.display());
        if let Ok(head) = std::fs::read_to_string(&git_head)
            && let Some(reference) = head.trim().strip_prefix("ref: ")
        {
            let reference = repo.join(".git").join(reference);
            if reference.exists() {
                println!("cargo:rerun-if-changed={}", reference.display());
            }
        }
    }
    let git_log = repo.join(".git/logs/HEAD");
    if git_log.exists() {
        println!("cargo:rerun-if-changed={}", git_log.display());
    }

    let build = std::env::var("FOLISIFT_BUILD")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| git_count(&repo))
        .unwrap_or_else(|| "0".to_string());

    println!("cargo:rustc-env=FOLISIFT_BUILD={build}");
}

fn git_count(repo: &Path) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-list", "--count", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let count = String::from_utf8(output.stdout).ok()?;
    let count = count.trim();
    if count.is_empty() {
        None
    } else {
        Some(count.to_string())
    }
}
