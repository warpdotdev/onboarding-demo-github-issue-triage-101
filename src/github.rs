use std::process::{Command, Stdio};

use octocrab::Octocrab;
use serde_json::json;

#[derive(Debug, Clone)]
pub struct Issue {
    pub number: u64,
    pub title: String,
    pub body: Option<String>,
    pub author: Author,
    pub created_at: String,
    pub labels: Vec<Label>,
    pub comments: Vec<Comment>,
}

#[derive(Debug, Clone)]
pub struct Author {
    pub login: String,
}

#[derive(Debug, Clone)]
pub struct Label {
    pub name: String,
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct Comment {
    pub author: String,
    pub body: String,
}
/// Build octocrab client, using GITHUB_TOKEN if available
fn build_client() -> Result<Octocrab, String> {
    let builder = Octocrab::builder();
    let builder = if let Ok(token) = std::env::var("GITHUB_TOKEN") {
        builder.personal_token(token)
    } else {
        builder
    };
    builder
        .build()
        .map_err(|e| format!("Failed to build client: {e}"))
}

/// Fetch issues from a GitHub repository using the GitHub REST API
pub async fn fetch_issues(repo: &str, limit: u32) -> Result<Vec<Issue>, String> {
    let (owner, repo_name) = repo
        .split_once('/')
        .ok_or_else(|| "Invalid repo format, expected owner/repo".to_string())?;

    let client = build_client()?;
    let issue_handler = client.issues(owner, repo_name);

    // Fetch issues
    let page = issue_handler
        .list()
        .state(octocrab::params::State::Open)
        .per_page(limit.min(100) as u8)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch issues: {e}"))?;

    let mut issues = Vec::new();
    for gh_issue in page.items {
        // Skip pull requests (GitHub API returns PRs as issues too)
        if gh_issue.pull_request.is_some() {
            continue;
        }

        // Fetch comments for this issue
        let comments = fetch_comments(&client, owner, repo_name, gh_issue.number).await;

        let issue = Issue {
            number: gh_issue.number,
            title: gh_issue.title,
            body: gh_issue.body,
            author: Author {
                login: gh_issue.user.login,
            },
            created_at: gh_issue.created_at.to_rfc3339(),
            labels: gh_issue
                .labels
                .into_iter()
                .map(|l| Label {
                    name: l.name,
                    color: l.color,
                })
                .collect(),
            comments,
        };
        issues.push(issue);
    }

    Ok(issues)
}

const MAX_COMMENTS: u8 = 10;

async fn fetch_comments(
    client: &Octocrab,
    owner: &str,
    repo: &str,
    issue_number: u64,
) -> Vec<Comment> {
    let Ok(page) = client
        .issues(owner, repo)
        .list_comments(issue_number)
        .per_page(MAX_COMMENTS)
        .send()
        .await
    else {
        return Vec::new();
    };

    page.items
        .into_iter()
        .map(|c| Comment {
            author: c.user.login,
            body: c.body.unwrap_or_default(),
        })
        .collect()
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

/// Trigger an Oz cloud agent run for the issue number.
pub async fn assign_issue_to_oz(issue_number: u64) -> Result<String, String> {
    let api_key = std::env::var("WARP_API_KEY")
        .or_else(|_| std::env::var("OZ_API_KEY"))
        .map_err(|_| "Missing API key: set WARP_API_KEY (or OZ_API_KEY)".to_string())?;
    let base_url =
        std::env::var("WARP_SERVER_URL").unwrap_or_else(|_| "https://app.warp.dev".to_string());
    let prompt = format!("address issue number {}", issue_number);

    let mut body = json!({
        "prompt": prompt
    });
    if let Ok(environment_id) = std::env::var("OZ_ENVIRONMENT_ID") {
        body["config"] = json!({ "environment_id": environment_id });
    }

    let client = reqwest::Client::new();
    let response = client
        .post(format!(
            "{}/api/v1/agent/run",
            base_url.trim_end_matches('/')
        ))
        .bearer_auth(api_key)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Failed to call Oz API: {e}"))?;

    let status = response.status();
    let payload = response
        .json::<serde_json::Value>()
        .await
        .unwrap_or_else(|_| json!({}));
    if !status.is_success() {
        return Err(format!(
            "Oz API error {}: {}",
            status.as_u16(),
            payload
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("request failed")
        ));
    }

    let run_id = payload
        .get("run_id")
        .or_else(|| payload.get("id"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");
    Ok(format!("Assigned to Oz run {}", run_id))
}
