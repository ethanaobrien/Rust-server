use std::fs;

pub struct GetByPath {
    pub is_file: bool,
    pub is_directory: bool,
    pub error: bool,
    pub path: String,
    pub length: u64
}

impl GetByPath {
    pub fn new(path: &str) -> GetByPath {
        let mut file = false;
        let mut dir = false;
        let mut error = false;
        let mut length : u64 = 0;
        match fs::metadata(path) {
            Ok(metadata) => {
                if metadata.is_file() {
                    file = true;
                } else if metadata.is_dir() {
                    dir = true;
                } else {
                    error = true;
                }
                length = metadata.len();
            }
            Err(_) => {
                error = true;
            }
        }
        GetByPath {
            is_file: file,
            is_directory: dir,
            error: error,
            path: path.to_string(),
            length: length
        }
    }
    //will write more later
}
