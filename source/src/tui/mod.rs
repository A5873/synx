//! Interactive TUI for Synx
//!
//! This module provides an interactive terminal UI for stepping through code issues
//! and interactively fixing them, similar to `git add -p`.
//!
//! Features:
//! - Issue navigation
//! - Syntax tree visualization
//! - Interactive issue fixing
//! - Keyboard shortcuts
//! - Real-time monitoring (htop-style)

// pub mod interactive; // Temporarily disabled due to compilation issues

use std::collections::HashMap;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

use anyhow::{Result, Context, anyhow};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::{debug, info};
use serde::{Serialize, Deserialize};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Tabs, Wrap},
    Frame, Terminal,
};
use tree_sitter::{Parser, Tree};
use uuid::Uuid;

// Import lint rule explanations
// Remove lints module import as it doesn't exist

use crate::analysis::IssueSeverity;

// Temporary ValidationIssue type for TUI compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub file_path: PathBuf,
    pub issue_type: String,
    pub severity: IssueSeverity,
    pub message: String,
    pub line_start: usize,
    pub line_end: usize,
    pub suggested_fix: Option<String>,
    pub context: std::collections::HashMap<String, String>,
}

// Temporary ValidationReport type for TUI compatibility  
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub file_issues: std::collections::HashMap<PathBuf, Vec<ValidationIssue>>,
}

// Temporary LintRules type for TUI compatibility
#[derive(Debug, Clone)]
pub struct LintRules {
    pub rules_by_language: std::collections::HashMap<String, Vec<LintRule>>,
}

impl LintRules {
    pub fn new() -> Self {
        Self {
            rules_by_language: std::collections::HashMap::new(),
        }
    }
    
    pub fn find_rule_by_code(&self, code: &str) -> Option<&LintRule> {
        for rules in self.rules_by_language.values() {
            for rule in rules {
                if rule.code == code {
                    return Some(rule);
                }
            }
        }
        None
    }
}

// Temporary LintRule type for TUI compatibility
#[derive(Debug, Clone)]
pub struct LintRule {
    pub code: String,
    pub title: String,
    pub description: String,
    pub examples: Vec<String>,
}

mod syntax;
mod issue_state;
mod widgets;

use issue_state::{IssueState, IssueAction};
use syntax::SyntaxHighlighter;
use widgets::{CodeView, SyntaxTreeView, IssuePanel, ActionMenu};

// TUI application state
pub struct TuiApp {
    // Terminal and UI state
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    
    // Application state
    state: AppState,
    
    // Syntax highlighter
    syntax_highlighter: SyntaxHighlighter,
    
    // Tree-sitter parsers
    parsers: HashMap<String, Parser>,
}

#[derive(Debug, Clone)]
pub struct AppState {
    // List of files with issues
    issue_files: Vec<PathBuf>,
    
    // Current file being examined
    current_file: usize,
    
    // Current file's issues
    issues: Vec<ValidationIssue>,
    
    // Current issue index
    current_issue: usize,
    
    // File content
    file_content: String,
    
    // Syntax tree for current file
    syntax_tree: Option<Tree>,
    
    // Active tab
    active_tab: Tab,
    
    // Issue state tracking
    issue_states: HashMap<String, IssueState>,
    
    // Whether the application should exit
    should_exit: bool,
    
    // UI navigation history
    navigation_history: Vec<(usize, usize)>,
    
    // Current view scroll position
    scroll_position: u16,
    
    // Lint rules collection
    lint_rules: LintRules,
    
    // Currently selected lint rule
    current_rule: Option<String>,
    
    // Whether to show examples in explanation
    show_examples: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Issues,
    SyntaxTree,
    Actions,
    Explanation,
}

impl TuiApp {
    /// Initialize a new TUI application
    pub fn new(validation_report: ValidationReport) -> Result<Self> {
        // Setup terminal
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        
        // Initialize syntax highlighter
        let syntax_highlighter = SyntaxHighlighter::new()?;
        
        // Initialize parsers
        let mut parsers = HashMap::new();
        parsers.insert("rust".to_string(), syntax::create_rust_parser()?);
        parsers.insert("python".to_string(), syntax::create_python_parser()?);
        parsers.insert("javascript".to_string(), syntax::create_javascript_parser()?);
        parsers.insert("typescript".to_string(), syntax::create_typescript_parser()?);
        parsers.insert("c".to_string(), syntax::create_c_parser()?);
        parsers.insert("cpp".to_string(), syntax::create_cpp_parser()?);
        parsers.insert("go".to_string(), syntax::create_go_parser()?);
        parsers.insert("java".to_string(), syntax::create_java_parser()?);
        
        // Collect files with issues
        let mut issue_files = Vec::new();
        for (path, issues) in &validation_report.file_issues {
            if !issues.is_empty() {
                issue_files.push(path.clone());
            }
        }
        
        if issue_files.is_empty() {
            return Err(anyhow!("No issues found to fix"));
        }
        
        // Initialize app state
        let first_file = issue_files[0].clone();
        let file_content = std::fs::read_to_string(&first_file)
            .context(format!("Failed to read file: {}", first_file.display()))?;
        
        let issues = validation_report.file_issues
            .get(&first_file)
            .cloned()
            .unwrap_or_default();
        
        // Generate issue states
        let mut issue_states = HashMap::new();
        for issue in &issues {
            let id = Uuid::new_v4().to_string();
            issue_states.insert(id, IssueState::new(issue.clone()));
        }
        
        // Parse syntax tree for the file
        let syntax_tree = syntax::parse_file(&first_file, &parsers)?;
        
        // Initialize lint rules
        let lint_rules = LintRules::new();

        let state = AppState {
            issue_files,
            current_file: 0,
            issues,
            current_issue: 0,
            file_content,
            syntax_tree,
            active_tab: Tab::Issues,
            issue_states,
            should_exit: false,
            navigation_history: Vec::new(),
            scroll_position: 0,
            lint_rules,
            current_rule: None,
            show_examples: true,
        };
        
        Ok(Self {
            terminal,
            state,
            syntax_highlighter,
            parsers,
        })
    }
    
    /// Run the TUI application
    pub fn run(&mut self) -> Result<()> {
        // Enter alternate screen and enable raw mode
        enable_raw_mode()?;
        execute!(
            io::stdout(),
            EnterAlternateScreen,
            EnableMouseCapture
        )?;
        
        // Main event loop
        while !self.state.should_exit {
            // Draw UI using the proper draw_ui method
            // Clone state for the closure to avoid borrowing issues
            let state_clone = self.state.clone();
            let highlighter_clone = self.syntax_highlighter.clone();
            
            self.terminal.draw(|f| {
                Self::draw_ui_static(f, &state_clone, &highlighter_clone);
            })?;
            
            // Handle events
            if crossterm::event::poll(Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    self.handle_key_event(key)?;
                }
            }
        }
        
        // Restore terminal
        disable_raw_mode()?;
        execute!(
            io::stdout(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        
        Ok(())
    }
    
    /// Draw the user interface (static version for borrowing safety)
    fn draw_ui_static<B: Backend>(f: &mut Frame<B>, state: &AppState, syntax_highlighter: &SyntaxHighlighter) {
        // Create the layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),   // Tabs
                Constraint::Min(0),      // Main content
                Constraint::Length(3),   // Status bar
            ].as_ref())
            .split(f.size());
        
        // Draw tabs
        Self::draw_tabs_static(f, chunks[0], state);
        
        // Draw main content
        match state.active_tab {
            Tab::Issues => Self::draw_issues_view_static(f, chunks[1], state, syntax_highlighter),
            Tab::SyntaxTree => Self::draw_syntax_tree_view_static(f, chunks[1], state),
            Tab::Actions => Self::draw_actions_view_static(f, chunks[1], state),
            Tab::Explanation => Self::draw_explanation_view_static(f, chunks[1], state),
        }
        
        // Draw status bar
        Self::draw_status_bar_static(f, chunks[2], state);
    }
    
    /// Draw the user interface
    fn draw_ui<B: Backend>(&mut self, f: &mut Frame<B>) {
        // Create the layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3),   // Tabs
                Constraint::Min(0),      // Main content
                Constraint::Length(3),   // Status bar
            ].as_ref())
            .split(f.size());
        
        // Draw tabs
        self.draw_tabs(f, chunks[0]);
        
        // Draw main content
        match self.state.active_tab {
            Tab::Issues => self.draw_issues_view(f, chunks[1]),
            Tab::SyntaxTree => self.draw_syntax_tree_view(f, chunks[1]),
            Tab::Actions => self.draw_actions_view(f, chunks[1]),
            Tab::Explanation => self.draw_explanation_view(f, chunks[1]),
        }
        
        // Draw status bar
        self.draw_status_bar(f, chunks[2]);
    }
    
    /// Draw the tab bar
    fn draw_tabs<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let tab_titles = vec!["Issues", "Syntax Tree", "Actions", "Explanation"];
        let active_tab_idx = match self.state.active_tab {
            Tab::Issues => 0,
            Tab::SyntaxTree => 1,
            Tab::Actions => 2,
            Tab::Explanation => 3,
        };
        
        let tabs = Tabs::new(
            tab_titles.iter().map(|t| {
                Spans::from(Span::styled(*t, Style::default().fg(Color::White)))
            }).collect()
        )
        .block(Block::default().borders(Borders::ALL).title("Synx Interactive Mode"))
        .select(active_tab_idx)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        );
        
        f.render_widget(tabs, area);
    }
    
    /// Draw the issues view
    fn draw_issues_view<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60),  // Code view
                Constraint::Percentage(40),  // Issue details
            ].as_ref())
            .split(area);
        
        // Draw code with issue highlighted
        self.draw_code_view(f, chunks[0]);
        
        // Draw issue details
        self.draw_issue_details(f, chunks[1]);
    }
    
    /// Draw the explanation view
    fn draw_explanation_view<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        if let Some(rule_code) = &self.state.current_rule {
            if let Some(rule) = self.state.lint_rules.find_rule_by_code(rule_code) {
                let explanation_panel = widgets::ExplanationPanel::new(rule, self.state.show_examples);
                f.render_widget(explanation_panel, area);
            } else {
                // No rule found, show error message
                let message = Paragraph::new(format!("No explanation found for rule code: {}", rule_code))
                    .block(Block::default().borders(Borders::ALL).title("Explanation Error"))
                    .wrap(Wrap { trim: true });
                f.render_widget(message, area);
            }
        } else {
            // No rule selected, show message
            let message = Paragraph::new("No rule selected. Press 'e' while viewing an issue to see its explanation.")
                .block(Block::default().borders(Borders::ALL).title("Explanation"))
                .wrap(Wrap { trim: true });
            f.render_widget(message, area);
        }
    }
    
    /// Draw the code view
    fn draw_code_view<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        if self.state.issues.is_empty() {
            let paragraph = Paragraph::new("No issues to display")
                .block(Block::default().borders(Borders::ALL).title("Code"));
            f.render_widget(paragraph, area);
            return;
        }
        
        let current_issue = &self.state.issues[self.state.current_issue];
        let file_path = &self.state.issue_files[self.state.current_file];
        
        // Create CodeView widget
        let code_view = CodeView::new(
            &self.state.file_content,
            current_issue.line_start,
            current_issue.line_end,
            file_path,
            &self.syntax_highlighter,
            self.state.scroll_position,
        );
        
        f.render_widget(code_view, area);
    }
    
    /// Draw issue details
    fn draw_issue_details<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        if self.state.issues.is_empty() {
            let paragraph = Paragraph::new("No issues to display")
                .block(Block::default().borders(Borders::ALL).title("Issue Details"));
            f.render_widget(paragraph, area);
            return;
        }
        
        let current_issue = &self.state.issues[self.state.current_issue];
        let issue_panel = IssuePanel::new(current_issue);
        
        f.render_widget(issue_panel, area);
    }
    
    /// Draw the syntax tree view
    fn draw_syntax_tree_view<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        if let Some(tree) = &self.state.syntax_tree {
            let tree_view = SyntaxTreeView::new(tree, &self.state.file_content);
            f.render_widget(tree_view, area);
        } else {
            let paragraph = Paragraph::new("Syntax tree not available")
                .block(Block::default().borders(Borders::ALL).title("Syntax Tree"));
            f.render_widget(paragraph, area);
        }
    }
    
    /// Draw the actions view
    fn draw_actions_view<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        if self.state.issues.is_empty() {
            let paragraph = Paragraph::new("No issues to take action on")
                .block(Block::default().borders(Borders::ALL).title("Actions"));
            f.render_widget(paragraph, area);
            return;
        }
        
        let current_issue = &self.state.issues[self.state.current_issue];
        let issue_id = self.get_current_issue_id();
        let issue_state = self.state.issue_states.get(&issue_id).unwrap();
        
        let action_menu = ActionMenu::new(current_issue, issue_state);
        f.render_widget(action_menu, area);
    }
    
    /// Draw the status bar
    fn draw_status_bar<B: Backend>(&self, f: &mut Frame<B>, area: Rect) {
        let file_info = if !self.state.issue_files.is_empty() {
            let current_file = &self.state.issue_files[self.state.current_file];
            format!(
                "File {}/{}: {}",
                self.state.current_file + 1,
                self.state.issue_files.len(),
                current_file.display()
            )
        } else {
            "No files with issues".to_string()
        };
        
        let issue_info = if !self.state.issues.is_empty() {
            format!(
                "Issue {}/{} ({})",
                self.state.current_issue + 1,
                self.state.issues.len(),
                self.state.issues[self.state.current_issue].severity
            )
        } else {
            "No issues".to_string()
        };
        
        // Customize help text based on active tab
        let help_text = match self.state.active_tab {
            Tab::Explanation => "q: Quit | Tab: Switch view | x: Toggle examples | t: Next related rule | e: Back to issue",
            _ => "q: Quit | Tab: Switch view | n: Next issue | p: Previous issue | f: Fix | i: Ignore | e: Explanation",
        };
        
        let status_text = vec![
            Spans::from(Span::styled(file_info, Style::default().fg(Color::Cyan))),
            Spans::from(Span::styled(issue_info, Style::default().fg(Color::Yellow))),
            Spans::from(Span::styled(help_text, Style::default().fg(Color::Gray))),
        ];
        
        let status_paragraph = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: false });
            
        f.render_widget(status_paragraph, area);
    }
    
    /// Handle keyboard events
    fn handle_key_event(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            // Quit
            KeyCode::Char('q') => {
                self.state.should_exit = true;
            }
            
            // Switch tabs
            KeyCode::Tab => {
                self.state.active_tab = match self.state.active_tab {
                    Tab::Issues => Tab::SyntaxTree,
                    Tab::SyntaxTree => Tab::Actions,
                    Tab::Actions => Tab::Explanation,
                    Tab::Explanation => Tab::Issues,
                };
            }
            
            // Show explanation for current issue
            KeyCode::Char('e') => {
                if !self.state.issues.is_empty() {
                    let current_issue = &self.state.issues[self.state.current_issue];
                    
                    // Extract rule code from issue type (format typically: "type:rule_code")
                    let rule_code = if let Some(code_pos) = current_issue.issue_type.find(':') {
                        let code = &current_issue.issue_type[code_pos + 1..];
                        Some(code.trim().to_string())
                    } else {
                        // As a fallback, try to guess the rule code from the issue type
                        match current_issue.issue_type.as_str() {
                            "unused_variable" | "unused_var" => Some("R0001".to_string()), // Rust unused variable
                            "unused_import" => Some("R0002".to_string()),                  // Rust unused import
                            "unused_must_use" => Some("R0003".to_string()),               // Rust unused must_use
                            "dead_code" => Some("R0004".to_string()),                     // Rust dead code
                            "no_var" => Some("J0002".to_string()),                       // JS no var
                            _ => None,
                        }
                    };
                    
                    // Set current rule if found
                    if let Some(code) = rule_code {
                        debug!("Showing explanation for rule: {}", code);
                        self.state.current_rule = Some(code);
                        
                        // Switch to explanation tab
                        self.state.active_tab = Tab::Explanation;
                    } else {
                        debug!("No rule code found for issue: {}", current_issue.issue_type);
                    }
                }
            }
            
            // Toggle code examples in explanation
            KeyCode::Char('x') => {
                if self.state.active_tab == Tab::Explanation {
                    self.state.show_examples = !self.state.show_examples;
                    debug!("Examples display toggled: {}", self.state.show_examples);
                }
            }
            
            // Navigation - next issue
            KeyCode::Char('n') => {
                if !self.state.issues.is_empty() {
                    // Save current position in navigation history
                    self.state.navigation_history.push((self.state.current_file, self.state.current_issue));
                    
                    // Move to next issue
                    self.next_issue()?;
                }
            }
            
            // Navigation - previous issue
            KeyCode::Char('p') => {
                if !self.state.issues.is_empty() {
                    // Save current position in navigation history
                    self.state.navigation_history.push((self.state.current_file, self.state.current_issue));
                    
                    // Move to previous issue
                    self.prev_issue()?;
                }
            }
            
            // Navigation - next file
            KeyCode::Char('N') => {
                if self.state.current_file < self.state.issue_files.len() - 1 {
                    // Save current position in navigation history
                    self.state.navigation_history.push((self.state.current_file, self.state.current_issue));
                    
                    // Move to next file
                    self.next_file()?;
                }
            }
            
            // Navigation - previous file
            KeyCode::Char('P') => {
                if self.state.current_file > 0 {
                    // Save current position in navigation history
                    self.state.navigation_history.push((self.state.current_file, self.state.current_issue));
                    
                    // Move to previous file
                    self.prev_file()?;
                }
            }
            
            // Navigation - back in history
            KeyCode::Char('b') => {
                if let Some((file_idx, issue_idx)) = self.state.navigation_history.pop() {
                    if file_idx != self.state.current_file {
                        self.load_file(file_idx)?;
                    }
                    self.state.current_issue = issue_idx;
                }
            }
            
            // Fix current issue
            KeyCode::Char('f') => {
                if !self.state.issues.is_empty() {
                    let issue_id = self.get_current_issue_id();
                    let issue_state = self.state.issue_states.get_mut(&issue_id).unwrap();
                    issue_state.action = IssueAction::Fix;
                    
                    // Apply the fix if possible
                    if let Err(e) = self.apply_fix_to_current_issue() {
                        // Handle error (in production code, this would show an error message)
                        debug!("Failed to apply fix: {}", e);
                    } else {
                        // Move to next issue
                        self.next_issue()?;
                    }
                }
            }
            
            // Ignore current issue
            KeyCode::Char('i') => {
                if !self.state.issues.is_empty() {
                    let issue_id = self.get_current_issue_id();
                    let issue_state = self.state.issue_states.get_mut(&issue_id).unwrap();
                    issue_state.action = IssueAction::Ignore;
                    
                    // Move to next issue
                    self.next_issue()?;
                }
            }
            
            // Scroll controls
            KeyCode::Up => {
                if self.state.scroll_position > 0 {
                    self.state.scroll_position -= 1;
                }
            }
            
            KeyCode::Down => {
                self.state.scroll_position += 1;
            }
            
            KeyCode::PageUp => {
                if self.state.scroll_position >= 10 {
                    self.state.scroll_position -= 10;
                } else {
                    self.state.scroll_position = 0;
                }
            }
            
            KeyCode::PageDown => {
                self.state.scroll_position += 10;
            }
            
            KeyCode::Home => {
                self.state.scroll_position = 0;
            }
            
            KeyCode::End => {
                // Approximate total lines (this would be calculated based on actual content)
                let total_lines = self.state.file_content.lines().count() as u16;
                if total_lines > 20 {
                    self.state.scroll_position = total_lines - 20;
                }
            }
            
            // Handle action-specific shortcuts when in Actions tab
            // Handle explanation-specific shortcuts when in Explanation tab
            _ if self.state.active_tab == Tab::Explanation => {
                match key.code {
                    KeyCode::Char('t') => {
                        // Toggle between explanation views
                        if let Some(current_code) = &self.state.current_rule {
                            // Try to find alternative rules to display
                            let rule_family = match current_code.chars().next() {
                                Some('R') => "R", // Rust rules
                                Some('J') => "J", // JavaScript rules
                                Some('P') => "P", // Python rules
                                _ => "",
                            };
                            
                            if !rule_family.is_empty() {
                                // Find next rule in the family
                                let rules: Vec<&LintRule> = self.state.lint_rules.rules_by_language
                                    .values()
                                    .flatten()
                                    .filter(|r| r.code.starts_with(rule_family))
                                    .collect();
                                
                                if !rules.is_empty() {
                                    // Find current rule index
                                    let current_idx = rules.iter().position(|r| r.code == *current_code);
                                    
                                    if let Some(idx) = current_idx {
                                        // Get next rule in sequence (or wrap around)
                                        let next_idx = (idx + 1) % rules.len();
                                        self.state.current_rule = Some(rules[next_idx].code.clone());
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            
            // Handle action-specific shortcuts when in Actions tab
            _ if self.state.active_tab == Tab::Actions => {
                match key.code {
                    KeyCode::Char('1') => {
                        // Apply first fix option
                        self.apply_fix_option(0)?;
                    }
                    KeyCode::Char('2') => {
                        // Apply second fix option
                        self.apply_fix_option(1)?;
                    }
                    KeyCode::Char('3') => {
                        // Apply third fix option
                        self.apply_fix_option(2)?;
                    }
                    KeyCode::Char('a') => {
                        // Apply automatic fix
                        self.apply_automatic_fix()?;
                    }
                    _ => {}
                }
            }
            
            _ => {} // Ignore other keys
        }
        
        Ok(())
    }
    
    //
    // Helper methods for issue navigation
    //
    
    /// Get the ID for the current issue
    fn get_current_issue_id(&self) -> String {
        if self.state.issues.is_empty() {
            return String::new();
        }
        
        // In a real implementation, we would have a proper lookup mechanism
        // For now, we'll use the issue index as part of the ID string
        let issue_ids: Vec<String> = self.state.issue_states.keys().cloned().collect();
        issue_ids[self.state.current_issue].clone()
    }
    
    /// Move to the next issue
    fn next_issue(&mut self) -> Result<()> {
        if self.state.issues.is_empty() {
            return Ok(());
        }
        
        if self.state.current_issue < self.state.issues.len() - 1 {
            // More issues in this file
            self.state.current_issue += 1;
        } else {
            // Move to next file if available
            self.next_file()?;
        }
        
        Ok(())
    }
    
    /// Move to the previous issue
    fn prev_issue(&mut self) -> Result<()> {
        if self.state.issues.is_empty() {
            return Ok(());
        }
        
        if self.state.current_issue > 0 {
            // Previous issue in this file
            self.state.current_issue -= 1;
        } else {
            // Move to previous file if available
            self.prev_file()?;
        }
        
        Ok(())
    }
    
    /// Move to the next file
    fn next_file(&mut self) -> Result<()> {
        if self.state.current_file < self.state.issue_files.len() - 1 {
            self.state.current_file += 1;
            self.load_file(self.state.current_file)?;
            self.state.current_issue = 0;
        }
        
        Ok(())
    }
    
    /// Move to the previous file
    fn prev_file(&mut self) -> Result<()> {
        if self.state.current_file > 0 {
            self.state.current_file -= 1;
            self.load_file(self.state.current_file)?;
            // Go to the last issue in the previous file
            if !self.state.issues.is_empty() {
                self.state.current_issue = self.state.issues.len() - 1;
            }
        }
        
        Ok(())
    }
    
    /// Load a file at the specified index
    fn load_file(&mut self, file_index: usize) -> Result<()> {
        if file_index >= self.state.issue_files.len() {
            return Err(anyhow!("Invalid file index"));
        }
        
        let file_path = &self.state.issue_files[file_index];
        
        // Load file content
        self.state.file_content = std::fs::read_to_string(file_path)
            .context(format!("Failed to read file: {}", file_path.display()))?;
        
        // Update issues for this file
        // In a real implementation, this would come from a validation report
        // For now, we'll assume empty issues
        self.state.issues = Vec::new();
        
        // Parse syntax tree for the file
        self.state.syntax_tree = syntax::parse_file(file_path, &self.parsers)?;
        
        // Reset scroll position
        self.state.scroll_position = 0;
        
        Ok(())
    }
    
    //
    // Helper methods for issue fixing
    //
    
    /// Apply fix option at the specified index
    fn apply_fix_option(&mut self, option_index: usize) -> Result<()> {
        if self.state.issues.is_empty() {
            return Ok(());
        }
        
        let issue_id = self.get_current_issue_id();
        let issue_state = self.state.issue_states.get_mut(&issue_id).unwrap();
        
        // In a real implementation, this would apply the specific fix
        // For now, we'll just mark it as fixed
        issue_state.action = IssueAction::Fix;
        issue_state.selected_fix = Some(option_index);
        
        // Apply the fix
        self.apply_fix_to_current_issue()?;
        
        // Move to next issue
        self.next_issue()?;
        
        Ok(())
    }
    
    /// Apply automatic fix
    fn apply_automatic_fix(&mut self) -> Result<()> {
        if self.state.issues.is_empty() {
            return Ok(());
        }
        
        let issue_id = self.get_current_issue_id();
        let issue_state = self.state.issue_states.get_mut(&issue_id).unwrap();
        
        issue_state.action = IssueAction::Fix;
        issue_state.selected_fix = Some(0); // Default to first fix option
        
        // Apply the fix
        self.apply_fix_to_current_issue()?;
        
        // Move to next issue
        self.next_issue()?;
        
        Ok(())
    }
    
    /// Apply fix to current issue
    fn apply_fix_to_current_issue(&mut self) -> Result<()> {
        if self.state.issues.is_empty() {
            return Ok(());
        }
        
        let current_issue = &self.state.issues[self.state.current_issue];
        let file_path = &self.state.issue_files[self.state.current_file];
        
        // In a real implementation, this would:
        // 1. Get the fix from the issue
        // 2. Apply it to the file content
        // 3. Write the changes back to disk
        
        // For demonstration purposes, let's simulate a fix
        // Here we'd update the file_content and write it back to disk
        
        info!("Applied fix to {} for issue at line {}", 
            file_path.display(), 
            current_issue.line_start
        );
        
        // Mark the issue as fixed in the state
        let issue_id = self.get_current_issue_id();
        let issue_state = self.state.issue_states.get_mut(&issue_id).unwrap();
        issue_state.is_fixed = true;
        
        Ok(())
    }
    
    //
    // Helper methods for state persistence
    //
    
    /// Save the current state
    fn save_state(&self) -> Result<()> {
        // In a real implementation, this would save:
        // 1. Which issues have been fixed
        // 2. Which issues have been ignored
        // 3. Any custom fixes that were applied
        
        // For demonstration purposes, we'll just log the state
        let fixed_count = self.state.issue_states.values()
            .filter(|state| state.is_fixed)
            .count();
            
        let ignored_count = self.state.issue_states.values()
            .filter(|state| state.action == IssueAction::Ignore)
            .count();
            
        info!("State summary: {} fixed, {} ignored", fixed_count, ignored_count);
        
        Ok(())
    }
    
    /// Get results of the interactive session
    pub fn get_results(&self) -> InteractiveResults {
        // Count fixed and ignored issues
        let fixed_issues = self.state.issue_states.values()
            .filter(|state| state.is_fixed)
            .count();
            
        let ignored_issues = self.state.issue_states.values()
            .filter(|state| state.action == IssueAction::Ignore)
            .count();
            
        let remaining_issues = self.state.issue_states.len() - fixed_issues - ignored_issues;
        
        InteractiveResults {
            fixed_issues,
            ignored_issues,
            remaining_issues,
        }
    }
    
    //
    // Static drawing methods for borrowing safety
    //
    
    /// Draw the tab bar (static version)
    fn draw_tabs_static<B: Backend>(f: &mut Frame<B>, area: Rect, state: &AppState) {
        let tab_titles = vec!["Issues", "Syntax Tree", "Actions", "Explanation"];
        let active_tab_idx = match state.active_tab {
            Tab::Issues => 0,
            Tab::SyntaxTree => 1,
            Tab::Actions => 2,
            Tab::Explanation => 3,
        };
        
        let tabs = Tabs::new(
            tab_titles.iter().map(|t| {
                Spans::from(Span::styled(*t, Style::default().fg(Color::White)))
            }).collect()
        )
        .block(Block::default().borders(Borders::ALL).title("Synx Interactive Mode"))
        .select(active_tab_idx)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD)
        );
        
        f.render_widget(tabs, area);
    }
    
    /// Draw the issues view (static version)
    fn draw_issues_view_static<B: Backend>(f: &mut Frame<B>, area: Rect, state: &AppState, syntax_highlighter: &SyntaxHighlighter) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(60),  // Code view
                Constraint::Percentage(40),  // Issue details
            ].as_ref())
            .split(area);
        
        // Draw code with issue highlighted
        Self::draw_code_view_static(f, chunks[0], state, syntax_highlighter);
        
        // Draw issue details
        Self::draw_issue_details_static(f, chunks[1], state);
    }
    
    /// Draw the code view (static version)
    fn draw_code_view_static<B: Backend>(f: &mut Frame<B>, area: Rect, state: &AppState, syntax_highlighter: &SyntaxHighlighter) {
        if state.issues.is_empty() {
            let paragraph = Paragraph::new("No issues to display")
                .block(Block::default().borders(Borders::ALL).title("Code"));
            f.render_widget(paragraph, area);
            return;
        }
        
        let current_issue = &state.issues[state.current_issue];
        let file_path = &state.issue_files[state.current_file];
        
        // Create CodeView widget
        let code_view = CodeView::new(
            &state.file_content,
            current_issue.line_start,
            current_issue.line_end,
            file_path,
            syntax_highlighter,
            state.scroll_position,
        );
        
        f.render_widget(code_view, area);
    }
    
    /// Draw issue details (static version)
    fn draw_issue_details_static<B: Backend>(f: &mut Frame<B>, area: Rect, state: &AppState) {
        if state.issues.is_empty() {
            let paragraph = Paragraph::new("No issues to display")
                .block(Block::default().borders(Borders::ALL).title("Issue Details"));
            f.render_widget(paragraph, area);
            return;
        }
        
        let current_issue = &state.issues[state.current_issue];
        let issue_panel = IssuePanel::new(current_issue);
        
        f.render_widget(issue_panel, area);
    }
    
    /// Draw the syntax tree view (static version)
    fn draw_syntax_tree_view_static<B: Backend>(f: &mut Frame<B>, area: Rect, state: &AppState) {
        if let Some(tree) = &state.syntax_tree {
            let tree_view = SyntaxTreeView::new(tree, &state.file_content);
            f.render_widget(tree_view, area);
        } else {
            let paragraph = Paragraph::new("Syntax tree not available")
                .block(Block::default().borders(Borders::ALL).title("Syntax Tree"));
            f.render_widget(paragraph, area);
        }
    }
    
    /// Draw the actions view (static version)
    fn draw_actions_view_static<B: Backend>(f: &mut Frame<B>, area: Rect, state: &AppState) {
        if state.issues.is_empty() {
            let paragraph = Paragraph::new("No issues to take action on")
                .block(Block::default().borders(Borders::ALL).title("Actions"));
            f.render_widget(paragraph, area);
            return;
        }
        
        let current_issue = &state.issues[state.current_issue];
        
        // For the static version, we need to handle the issue ID lookup differently
        // since we can't call self.get_current_issue_id()
        let issue_ids: Vec<String> = state.issue_states.keys().cloned().collect();
        let fallback_id = String::new();
        let issue_id = if state.current_issue < issue_ids.len() {
            &issue_ids[state.current_issue]
        } else {
            // Fallback to first available ID
            issue_ids.first().unwrap_or(&fallback_id)
        };
        
        if let Some(issue_state) = state.issue_states.get(issue_id) {
            let action_menu = ActionMenu::new(current_issue, issue_state);
            f.render_widget(action_menu, area);
        } else {
            let paragraph = Paragraph::new("Issue state not found")
                .block(Block::default().borders(Borders::ALL).title("Actions Error"));
            f.render_widget(paragraph, area);
        }
    }
    
    /// Draw the explanation view (static version)
    fn draw_explanation_view_static<B: Backend>(f: &mut Frame<B>, area: Rect, state: &AppState) {
        if let Some(rule_code) = &state.current_rule {
            if let Some(rule) = state.lint_rules.find_rule_by_code(rule_code) {
                let explanation_panel = widgets::ExplanationPanel::new(rule, state.show_examples);
                f.render_widget(explanation_panel, area);
            } else {
                // No rule found, show error message
                let message = Paragraph::new(format!("No explanation found for rule code: {}", rule_code))
                    .block(Block::default().borders(Borders::ALL).title("Explanation Error"))
                    .wrap(Wrap { trim: true });
                f.render_widget(message, area);
            }
        } else {
            // No rule selected, show message
            let message = Paragraph::new("No rule selected. Press 'e' while viewing an issue to see its explanation.")
                .block(Block::default().borders(Borders::ALL).title("Explanation"))
                .wrap(Wrap { trim: true });
            f.render_widget(message, area);
        }
    }
    
    /// Draw the status bar (static version)
    fn draw_status_bar_static<B: Backend>(f: &mut Frame<B>, area: Rect, state: &AppState) {
        let file_info = if !state.issue_files.is_empty() {
            let current_file = &state.issue_files[state.current_file];
            format!(
                "File {}/{}: {}",
                state.current_file + 1,
                state.issue_files.len(),
                current_file.display()
            )
        } else {
            "No files with issues".to_string()
        };
        
        let issue_info = if !state.issues.is_empty() {
            format!(
                "Issue {}/{} ({})",
                state.current_issue + 1,
                state.issues.len(),
                state.issues[state.current_issue].severity
            )
        } else {
            "No issues".to_string()
        };
        
        // Customize help text based on active tab
        let help_text = match state.active_tab {
            Tab::Explanation => "q: Quit | Tab: Switch view | x: Toggle examples | t: Next related rule | e: Back to issue",
            _ => "q: Quit | Tab: Switch view | n: Next issue | p: Previous issue | f: Fix | i: Ignore | e: Explanation",
        };
        
        let status_text = vec![
            Spans::from(Span::styled(file_info, Style::default().fg(Color::Cyan))),
            Spans::from(Span::styled(issue_info, Style::default().fg(Color::Yellow))),
            Spans::from(Span::styled(help_text, Style::default().fg(Color::Gray))),
        ];
        
        let status_paragraph = Paragraph::new(status_text)
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: false });
            
        f.render_widget(status_paragraph, area);
    }
}

/// Results of the interactive fixing session
#[derive(Debug, Clone)]
pub struct InteractiveResults {
    pub fixed_issues: usize,
    pub ignored_issues: usize,
    pub remaining_issues: usize,
}

/// Run the TUI application with the provided validation report
pub fn run_interactive_mode(validation_report: ValidationReport) -> Result<InteractiveResults> {
    // Initialize the TUI app
    let mut app = TuiApp::new(validation_report)?;
    
    // Run the app
    app.run()?;
    
    // Return the results
    Ok(app.get_results())
}

