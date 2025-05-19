//! Issue state management for interactive TUI
//!
//! This module provides state tracking for issues during interactive mode.

use std::path::PathBuf;
use serde::{Serialize, Deserialize};
use crate::validators::ValidationIssue;

/// Available actions for an issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueAction {
    /// No action chosen yet
    Pending,
    /// Fix the issue
    Fix,
    /// Ignore the issue
    Ignore,
    /// Defer decision on this issue
    Defer,
}

impl Default for IssueAction {
    fn default() -> Self {
        Self::Pending
    }
}

/// A fix option for an issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FixOption {
    /// Description of the fix
    pub description: String,
    
    /// The fix implementation details (varies by issue type)
    pub implementation: FixImplementation,
    
    /// Estimated confidence in the fix (0-100)
    pub confidence: u8,
}

/// Implementation details for a fix option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FixImplementation {
    /// Replace text between line ranges
    ReplaceText {
        line_start: usize,
        line_end: usize,
        replacement: String,
    },
    
    /// Add new text at a specific line
    AddText {
        line: usize,
        text: String,
    },
    
    /// Delete text between line ranges
    DeleteText {
        line_start: usize,
        line_end: usize,
    },
    
    /// Move text from one location to another
    MoveText {
        source_start: usize,
        source_end: usize,
        target_line: usize,
    },
    
    /// Execute a command to fix the issue
    ExecuteCommand {
        command: String,
        args: Vec<String>,
    },
}

/// State of an issue during interactive fixing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueState {
    /// The original issue
    pub issue: ValidationIssue,
    
    /// Available fix options
    pub fix_options: Vec<FixOption>,
    
    /// Selected action
    pub action: IssueAction,
    
    /// Selected fix option index (if Fix action is selected)
    pub selected_fix: Option<usize>,
    
    /// Whether the issue has been fixed
    pub is_fixed: bool,
    
    /// Custom fix provided by the user
    pub custom_fix: Option<String>,
    
    /// Notes or comments about this issue
    pub notes: Option<String>,
    
    /// Timestamp when this state was last updated
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl IssueState {
    /// Create a new issue state
    pub fn new(issue: ValidationIssue) -> Self {
        let mut fix_options = Vec::new();
        
        // Generate fix options based on issue type
        match issue.issue_type.as_str() {
            "syntax_error" => {
                fix_options.push(FixOption {
                    description: "Auto-fix syntax error".to_string(),
                    implementation: FixImplementation::ReplaceText {
                        line_start: issue.line_start,
                        line_end: issue.line_end,
                        replacement: issue.suggested_fix.clone().unwrap_or_default(),
                    },
                    confidence: 80,
                });
            }
            "unused_variable" => {
                fix_options.push(FixOption {
                    description: "Remove unused variable".to_string(),
                    implementation: FixImplementation::DeleteText {
                        line_start: issue.line_start,
                        line_end: issue.line_end,
                    },
                    confidence: 90,
                });
                
                fix_options.push(FixOption {
                    description: "Add variable usage example".to_string(),
                    implementation: FixImplementation::AddText {
                        line: issue.line_end + 1,
                        text: format!("// Using the variable: {}", 
                            issue.context.get("variable_name").unwrap_or(&"var".to_string())),
                    },
                    confidence: 60,
                });
            }
            "formatting" => {
                fix_options.push(FixOption {
                    description: "Apply formatter".to_string(),
                    implementation: FixImplementation::ExecuteCommand {
                        command: "formatter".to_string(),
                        args: vec![issue.file_path.to_string_lossy().to_string()],
                    },
                    confidence: 95,
                });
            }
            // Add more issue types as needed
            _ => {
                // Generic fix based on suggested fix if available
                if let Some(suggested_fix) = &issue.suggested_fix {
                    fix_options.push(FixOption {
                        description: "Apply suggested fix".to_string(),
                        implementation: FixImplementation::ReplaceText {
                            line_start: issue.line_start,
                            line_end: issue.line_end,
                            replacement: suggested_fix.clone(),
                        },
                        confidence: 70,
                    });
                }
            }
        }
        
        Self {
            issue,
            fix_options,
            action: IssueAction::Pending,
            selected_fix: None,
            is_fixed: false,
            custom_fix: None,
            notes: None,
            last_updated: chrono::Utc::now(),
        }
    }
    
    /// Set the action for this issue
    pub fn set_action(&mut self, action: IssueAction) {
        self.action = action;
        self.last_updated = chrono::Utc::now();
        
        // Reset selected fix if not fixing
        if action != IssueAction::Fix {
            self.selected_fix = None;
        }
    }
    
    /// Select a fix option
    pub fn select_fix(&mut self, index: usize) -> bool {
        if index < self.fix_options.len() {
            self.selected_fix = Some(index);
            self.action = IssueAction::Fix;
            self.last_updated = chrono::Utc::now();
            true
        } else {
            false
        }
    }
    
    /// Add a custom fix
    pub fn add_custom_fix(&mut self, fix: String) {
        self.custom_fix = Some(fix);
        self.action = IssueAction::Fix;
        self.last_updated = chrono::Utc::now();
    }
    
    /// Mark the issue as fixed
    pub fn mark_fixed(&mut self) {
        self.is_fixed = true;
        self.last_updated = chrono::Utc::now();
    }
    
    /// Add notes about this issue
    pub fn add_notes(&mut self, notes: String) {
        self.notes = Some(notes);
        self.last_updated = chrono::Utc::now();
    }
    
    /// Get the currently selected fix
    pub fn get_selected_fix(&self) -> Option<&FixOption> {
        self.selected_fix.and_then(|idx| self.fix_options.get(idx))
    }
    
    /// Get the status of this issue as a string
    pub fn get_status(&self) -> &'static str {
        if self.is_fixed {
            "Fixed"
        } else {
            match self.action {
                IssueAction::Pending => "Pending",
                IssueAction::Fix => "To Fix",
                IssueAction::Ignore => "Ignored",
                IssueAction::Defer => "Deferred",
            }
        }
    }
}

/// A collection of issue states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueStateCollection {
    /// Map of issue IDs to states
    pub states: std::collections::HashMap<String, IssueState>,
    
    /// Timestamp when this collection was created
    pub created_at: chrono::DateTime<chrono::Utc>,
    
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl IssueStateCollection {
    /// Create a new, empty collection
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            states: std::collections::HashMap::new(),
            created_at: now,
            last_updated: now,
        }
    }
    
    /// Create a collection from validation issues
    pub fn from_issues(issues: Vec<ValidationIssue>) -> Self {
        let mut collection = Self::new();
        
        for issue in issues {
            let id = uuid::Uuid::new_v4().to_string();
            let state = IssueState::new(issue);
            collection.states.insert(id, state);
        }
        
        collection
    }
    
    /// Get statistics on issue states
    pub fn get_stats(&self) -> IssueStats {
        let mut stats = IssueStats::default();
        
        for state in self.states.values() {
            if state.is_fixed {
                stats.fixed += 1;
            } else {
                match state.action {
                    IssueAction::Pending => stats.pending += 1,
                    IssueAction::Fix => stats.to_fix += 1,
                    IssueAction::Ignore => stats.ignored += 1,
                    IssueAction::Defer => stats.deferred += 1,
                }
            }
        }
        
        stats.total = self.states.len();
        stats
    }
    
    /// Save the collection to a file
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)
    }
    
    /// Load a collection from a file
    pub fn load_from_file(path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let collection = serde_json::from_str(&json)?;
        Ok(collection)
    }
}

/// Statistics about issue states
#[derive(Debug, Clone, Copy, Default)]
pub struct IssueStats {
    /// Total number of issues
    pub total: usize,
    
    /// Number of fixed issues
    pub fixed: usize,
    
    /// Number of issues to fix
    pub to_fix: usize,
    
    /// Number of ignored issues
    pub ignored: usize,
    
    /// Number of deferred issues
    pub deferred: usize,
    
    /// Number of pending issues
    pub pending: usize,
}

impl IssueStats {
    /// Get the percentage of issues fixed
    pub fn percent_fixed(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            (self.fixed as f32 * 100.0) / (self.total as f32)
        }
    }
    
    /// Get the percentage of issues with decisions made
    pub fn percent_decided(&self) -> f32 {
        if self.total == 0 {
            0.0
        } else {
            let decided = self.fixed + self.to_fix + self.ignored + self.deferred;
            (decided as f32 * 100.0) / (self.total as f32)
        }
    }
}

