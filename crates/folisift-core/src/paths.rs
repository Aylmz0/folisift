use std::path::{Component, Path, PathBuf};

use crate::error::{Error, Result, io};

/// The workspace root. Every path handed to the engine is resolved through it.
#[derive(Debug, Clone)]
pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    /// Open an existing directory as a workspace.
    pub fn open(root: impl AsRef<Path>) -> Result<Self> {
        let root = root.as_ref();
        let root = root.canonicalize().map_err(|error| io(root, error))?;
        if !root.is_dir() {
            return Err(Error::Config {
                path: root,
                message: "workspace root is not a directory".to_string(),
            });
        }
        Ok(Self { root })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Resolve a path that must already exist and stay inside the workspace.
    pub fn resolve_read(&self, path: impl AsRef<Path>) -> Result<PathBuf> {
        let path = self.resolve_lexically(path.as_ref())?;
        let path = path.canonicalize().map_err(|error| io(&path, error))?;
        self.ensure_inside(&path)?;
        Ok(path)
    }

    /// Resolve a path that may be created. Only `guides/`, `output/` and
    /// `data/` are writable; `sources/` is deliberately read-only.
    pub fn resolve_write(&self, path: impl AsRef<Path>) -> Result<PathBuf> {
        let path = self.resolve_lexically(path.as_ref())?;
        let path = canonicalize_allow_missing(&path)?;
        self.ensure_inside(&path)?;

        let relative = path
            .strip_prefix(&self.root)
            .map_err(|_| Error::PathEscape { path: path.clone() })?;
        let first = relative.components().next();
        let allowed = matches!(
            first,
            Some(Component::Normal(name))
                if name == "guides" || name == "output" || name == "data"
        );
        if !allowed {
            return Err(Error::ReadOnly { path });
        }
        Ok(path)
    }

    /// Resolve a guide directory by slug.
    pub fn guide_dir(&self, slug: &str) -> Result<PathBuf> {
        validate_slug(slug)?;
        let path = self.root.join("guides").join(slug);
        self.resolve_read(path)
    }

    fn resolve_lexically(&self, path: &Path) -> Result<PathBuf> {
        if path.as_os_str().is_empty() {
            return Err(Error::PathEscape {
                path: path.to_path_buf(),
            });
        }
        let joined = if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.root.join(path)
        };
        let normalized = normalize_lexically(&joined)?;
        if !is_inside(&self.root, &normalized) {
            return Err(Error::PathEscape { path: normalized });
        }
        Ok(normalized)
    }

    fn ensure_inside(&self, path: &Path) -> Result<()> {
        if is_inside(&self.root, path) {
            Ok(())
        } else {
            Err(Error::PathEscape {
                path: path.to_path_buf(),
            })
        }
    }
}

/// Validate a guide slug: lowercase ASCII letters, digits and single hyphens.
pub fn validate_slug(slug: &str) -> Result<()> {
    let invalid = slug.is_empty()
        || slug.len() > 64
        || slug.starts_with('-')
        || slug.ends_with('-')
        || slug.contains("--")
        || !slug
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    if invalid {
        Err(Error::InvalidSlug {
            slug: slug.to_string(),
        })
    } else {
        Ok(())
    }
}

fn is_inside(root: &Path, path: &Path) -> bool {
    path == root || path.starts_with(root)
}

/// Collapse `.` and `..` without touching the filesystem. Going above the
/// filesystem root is rejected.
fn normalize_lexically(path: &Path) -> Result<PathBuf> {
    let mut output = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => output.push(prefix.as_os_str()),
            Component::RootDir => output.push(Component::RootDir.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !output.pop() {
                    return Err(Error::PathEscape {
                        path: path.to_path_buf(),
                    });
                }
            }
            Component::Normal(part) => output.push(part),
        }
    }
    Ok(output)
}

/// Canonicalize the deepest existing ancestor, then append the missing tail.
fn canonicalize_allow_missing(path: &Path) -> Result<PathBuf> {
    let mut missing: Vec<std::ffi::OsString> = Vec::new();
    let mut current = path.to_path_buf();

    loop {
        match current.canonicalize() {
            Ok(mut base) => {
                for part in missing.iter().rev() {
                    base.push(part);
                }
                return Ok(base);
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                let Some(name) = current.file_name().map(ToOwned::to_owned) else {
                    return Err(Error::PathEscape {
                        path: path.to_path_buf(),
                    });
                };
                missing.push(name);
                if !current.pop() {
                    return Err(Error::PathEscape {
                        path: path.to_path_buf(),
                    });
                }
            }
            Err(error) => return Err(io(&current, error)),
        }
    }
}
