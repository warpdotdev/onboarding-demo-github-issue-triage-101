use serde::Deserialize;
use std::process::{Command, Stdio};

#[derive(Debug, Clone, Deserialize)]
pub struct Issue {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    #[allow(dead_code)]
    pub state: String,
    pub author: Author,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    #[serde(default)]
    pub labels: Vec<Label>,
    #[serde(default)]
    pub comments: Vec<Comment>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Author {
    pub login: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Label {
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Comment {
    pub author: Author,
    pub body: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

/// Fetch issues from a GitHub repository using the gh CLI
pub fn fetch_issues(repo: &str, limit: u32) -> Result<Vec<Issue>, String> {
    let output = Command::new("gh")
        .args([
            "issue",
            "list",
            "-R",
            repo,
            "--state",
            "open",
            "--limit",
            &limit.to_string(),
            "--json",
            "number,title,body,state,author,createdAt,labels,comments",
        ])
        .output()
        .map_err(|e| format!("Failed to run gh CLI: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gh CLI error: {stderr}"));
    }

    let issues: Vec<Issue> =
        serde_json::from_slice(&output.stdout).map_err(|e| format!("Failed to parse JSON: {e}"))?;

    Ok(issues)
}

/// Open an issue in the browser
pub fn open_in_browser(repo: &str, issue_number: u64) -> Result<(), String> {
    let url = format!("https://github.com/{}/issues/{}", repo, issue_number);

    Command::new("open")
        .arg(&url)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to open browser: {e}"))?;

    Ok(())
}
