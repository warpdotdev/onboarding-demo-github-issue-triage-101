use crate::github::{self, Issue};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Filter,
}

pub struct App {
    pub repo: String,
    pub issues: Vec<Issue>,
    pub selected: usize,
    pub filter: String,
    pub input_mode: InputMode,
    pub loading: bool,
    pub error: Option<String>,
    runtime: tokio::runtime::Runtime,
}

impl App {
    pub fn new(repo: String) -> Self {
        let runtime = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
        Self {
            repo,
            issues: Vec::new(),
            selected: 0,
            filter: String::new(),
            input_mode: InputMode::Normal,
            loading: true,
            error: None,
            runtime,
        }
    }

    /// Fetch issues from GitHub
    pub fn refresh(&mut self) {
        self.loading = true;
        self.error = None;

        let repo = self.repo.clone();
        match self.runtime.block_on(github::fetch_issues(&repo, 100)) {
            Ok(issues) => {
                self.issues = issues;
                self.selected = 0;
            }
            Err(e) => {
                self.error = Some(e);
            }
        }

        self.loading = false;
    }

    /// Get filtered issues based on current filter text
    pub fn filtered_issues(&self) -> Vec<&Issue> {
        if self.filter.is_empty() {
            self.issues.iter().collect()
        } else {
            let filter_lower = self.filter.to_lowercase();
            self.issues
                .iter()
                .filter(|issue| {
                    issue.title.to_lowercase().contains(&filter_lower)
                        || issue
                            .labels
                            .iter()
                            .any(|l| l.name.to_lowercase().contains(&filter_lower))
                })
                .collect()
        }
    }

    /// Move selection down
    pub fn next(&mut self) {
        let filtered_len = self.filtered_issues().len();
        if filtered_len > 0 {
            self.selected = (self.selected + 1).min(filtered_len - 1);
        }
    }

    /// Move selection up
    pub fn previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    /// Get currently selected issue
    pub fn selected_issue(&self) -> Option<&Issue> {
        self.filtered_issues().get(self.selected).copied()
    }

    /// Open selected issue in browser
    pub fn open_selected(&self) {
        if let Some(issue) = self.selected_issue() {
            let _ = github::open_in_browser(&self.repo, issue.number);
        }
    }

    /// Start filter input mode
    pub fn start_filter(&mut self) {
        self.input_mode = InputMode::Filter;
    }

    /// Exit filter input mode
    pub fn exit_filter(&mut self) {
        self.input_mode = InputMode::Normal;
    }

    /// Clear filter
    pub fn clear_filter(&mut self) {
        self.filter.clear();
        self.selected = 0;
    }

    /// Add character to filter
    pub fn filter_push(&mut self, c: char) {
        self.filter.push(c);
        self.selected = 0;
    }

    /// Remove last character from filter
    pub fn filter_pop(&mut self) {
        self.filter.pop();
        self.selected = 0;
    }
}
