use std::path::{Path, PathBuf};

pub fn replace_home_dir_in_path(path: String) -> String {
    let home_dir = dirs::home_dir().expect("Failed to get home directory");
    path.replace("~", home_dir.to_str().unwrap())
}
