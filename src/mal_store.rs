use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

const MAL_SEEN_DIR: &str = "mal_seen";

pub struct MalStore;

impl MalStore {
    fn path_for(username: &str) -> PathBuf {
        Path::new(MAL_SEEN_DIR).join(username)
    }

    pub fn load(username: &str) -> HashSet<String> {
        fs::read_to_string(Self::path_for(username))
            .unwrap_or_default()
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_string())
            .collect()
    }

    pub fn save(username: &str, hashes: &[String]) {
        fs::create_dir_all(MAL_SEEN_DIR).expect("cannot create mal_seen dir");
        fs::write(Self::path_for(username), hashes.join("\n")).expect("cannot write mal seen file");
    }
}
