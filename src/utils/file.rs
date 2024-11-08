use std::fs;
use std::path::Path;
use std::io;
use tempfile::TempDir;
use std::process::Command;

pub fn extract_repo_name(repository: &str) -> String {
    repository
        .trim_end_matches(".git")
        .split('/')
        .last()
        .unwrap_or("unknown")
        .to_string()
}

pub fn repo_to_filename(repository: &str, branch: &str) -> String {
    let repo_name = extract_repo_name(repository);
    format!("{}_{}.yml", repo_name, branch.replace('/', "_"))
}

pub async fn clone_repository(repository: &str, branch: &str) -> io::Result<(TempDir, String)> {
    let temp_dir = TempDir::new()?;
    let temp_path = temp_dir.path();

    let clone_result = Command::new("git")
        .args(&[
            "clone",
            "--depth", "1",
            "-b", branch,
            repository,
            temp_path.to_str().unwrap()
        ])
        .output()?;

    if !clone_result.status.success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Failed to clone repository: {}",
                String::from_utf8_lossy(&clone_result.stderr)
            ),
        ));
    }

    let config_path = temp_path.join(".pipeline.yml");
    let config_content = fs::read_to_string(&config_path)?;

    Ok((temp_dir, config_content))
}

pub fn ensure_directory(path: &str) -> io::Result<()> {
    if !Path::new(path).exists() {
        fs::create_dir_all(path)?;
    }
    Ok(())
}

pub fn save_file(path: &str, content: &str) -> io::Result<()> {
    if let Some(parent) = Path::new(path).parent() {
        ensure_directory(parent.to_str().unwrap())?;
    }
    fs::write(path, content)
}

pub fn read_file(path: &str) -> io::Result<String> {
    fs::read_to_string(path)
}