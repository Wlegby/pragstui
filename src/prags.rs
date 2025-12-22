#![allow(dead_code)]
use std::{fs, io, path::PathBuf};

#[derive(Clone, Debug, Default)]
pub struct Project {
    pub path: String,
    pub name: String,
    pub tags: Vec<String>,
}

impl Project {
    pub fn get_all(self, path: Option<String>) -> Vec<Project> {
        let mut projects = Vec::new();

        let files = self.get_toml(path);

        for file in files {
            let tags = self.get_tag_from_file(&file);
            let name = self.name_from_file(&file);
            projects.push(Project {
                path: file.to_str().unwrap().to_string(),
                name,
                tags,
            })
        }

        projects
    }

    fn get_toml(&self, path: Option<String>) -> Vec<PathBuf> {
        let path = if let Some(p) = path {
            PathBuf::from(p)
        } else {
            PathBuf::from("./")
        };

        let start = if path.exists() {
            path
        } else {
            PathBuf::from("./")
        };

        let mut folders: Vec<PathBuf> = vec![start];
        let mut toml: Vec<PathBuf> = Vec::new();

        let mut i = 0;
        loop {
            if i < folders.len() {
                let current_folder = folders[i].clone();
                let _ = self.go_through_folder(current_folder, &mut folders, &mut toml);

                i += 1;
            } else {
                break;
            }
        }

        toml
    }

    fn go_through_folder(
        &self,
        path: PathBuf,
        folders: &mut Vec<PathBuf>,
        tomls: &mut Vec<PathBuf>,
    ) -> io::Result<()> {
        let content = fs::read_dir(path)?;

        let mut temp_folders: Vec<PathBuf> = Vec::new();
        let mut is_project = false;

        for entry in content {
            let entry = entry?;
            if entry.file_type()?.is_file() && entry.file_name().to_str().unwrap() == "Cargo.toml" {
                tomls.push(entry.path());
                is_project = true;
                break;
            } else if entry.file_type()?.is_dir() {
                temp_folders.push(entry.path());
            }
        }

        if !is_project {
            temp_folders
                .into_iter()
                .for_each(|folder| folders.push(folder));
        }

        Ok(())
    }
    fn get_tag_from_file(&self, file: &PathBuf) -> Vec<String> {
        let mut tags = Vec::new();

        let content = fs::read_to_string(file).expect("Failed to read file");

        for line in content.lines() {
            if line.len() > 2 && line.starts_with("#-") {
                tags.push(line[2..].to_lowercase());
            }
        }

        tags.sort();
        tags.dedup();
        tags
    }
    fn name_from_file(&self, file: &PathBuf) -> String {
        let content = fs::read_to_string(file).expect("Failed to read file");
        for line in content.lines() {
            let words: Vec<&str> = line.split_whitespace().collect();

            if !line.is_empty() && words[0] == "name" {
                return words[2].replace('"', "").to_lowercase();
            }
        }

        "none".to_string()
    }
}
