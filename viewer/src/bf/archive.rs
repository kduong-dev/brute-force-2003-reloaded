//! Level archives (data/<level>.tgz: gzip'd tar). Members are keyed by base name.

use std::collections::HashMap;
use std::io::Read;
use std::path::Path;

pub struct Archive {
    files: HashMap<String, Vec<u8>>,
}

impl Archive {
    /// Read every member whose base name passes `keep`.
    pub fn open(path: &Path, keep: impl Fn(&str) -> bool) -> Result<Self, String> {
        Self::open_until(path, keep, usize::MAX)
    }

    /// Read the members whose base name passes `keep`, stopping once `count` are read (the rest
    /// of the gzip stream isn't unpacked: a level's one levels-*.xmb, without its models).
    pub fn open_until(path: &Path, keep: impl Fn(&str) -> bool, count: usize) -> Result<Self, String> {
        let f = std::fs::File::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let mut tar = tar::Archive::new(flate2::read::GzDecoder::new(std::io::BufReader::new(f)));
        let mut files = HashMap::new();
        for entry in tar.entries().map_err(|e| e.to_string())? {
            let mut entry = entry.map_err(|e| e.to_string())?;
            if !entry.header().entry_type().is_file() {
                continue;
            }
            let p = entry.path().map_err(|e| e.to_string())?.into_owned();
            let base = p.file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
            if base.is_empty() || !keep(&base) {
                continue;
            }
            let mut buf = Vec::with_capacity(entry.size() as usize);
            entry.read_to_end(&mut buf).map_err(|e| e.to_string())?;
            files.insert(base, buf);
            if files.len() >= count {
                break;
            }
        }
        Ok(Self { files })
    }

    pub fn get(&self, name: &str) -> Option<&[u8]> {
        self.files.get(name).map(|v| v.as_slice()).filter(|v| !v.is_empty())
    }

    /// Base names starting with `prefix` and ending with `ext`, sorted.
    pub fn find(&self, prefix: &str, ext: &str) -> Vec<String> {
        let mut v: Vec<String> =
            self.files.keys().filter(|n| n.starts_with(prefix) && n.ends_with(ext)).cloned().collect();
        v.sort();
        v
    }
}
