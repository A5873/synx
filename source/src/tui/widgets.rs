//! Custom TUI widgets for the interactive mode
//!
//! This module provides custom TUI widgets for code display, syntax tree
//! visualization, issue details, and action menus.

use std::path::{Path, PathBuf};

use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, StatefulWidget, Widget, Wrap},
};
use tree_sitter::{Tree, Node};

use crate::validators::ValidationIssue;
use super::issue_state::{IssueState, IssueAction, FixOption};
use super::syntax::{SyntaxHighlighter, TreeFormatter};

/// Widget for displaying code with syntax highlighting
pub struct CodeView<'a> {
    /// Code content
    content: &'a str,
    
    /// Start line of the issue
    issue_line_start: usize,
    
    /// End line of the issue
    issue_line_end: usize,
    
    /// File path
    file_path: &'a Path,
    
    /// Syntax highlighter
    highlighter: &'a SyntaxHighlighter,
    
    /// Current scroll position
    scroll: u16,
}

impl<'a> CodeView<'a> {
    /// Create a new code view
    pub fn new(
        content: &'a str,
        issue_line_start: usize,
        issue_line_end: usize,
        file_path: &'a Path,
        highlighter: &'a SyntaxHighlighter,
        scroll: u16,
    ) -> Self {
        Self {
            content,
            issue_line_start,
            issue_line_end,
            file_path,
            highlighter,
            scroll,
        }
    }
}

impl<'a> Widget for CodeView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create a block for the code view
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Code: {}", self.file_path.display()));
        
        // Calculate inner area
        let inner_area = block.inner(area);
        
        // Render block
        block.render(area, buf);
        
        // Get highlighted code
        let highlighted = self.highlighter.highlight(self.content, self.file_path);
        
        // Calculate visible lines
        let height = inner_area.height as usize;
        let total_lines = highlighted.len();
        
        // Ensure scroll is in bounds
        let scroll = self.scroll as usize;
        let scroll = scroll.min(total_lines.saturating_sub(height));
        
        // Count visible lines and generate line numbers
        let visible_lines = height.min(total_lines.saturating_sub(scroll));
        
        // Get max line number width
        let line_num_width = (total_lines.to_string().len() + 1) as u16;
        
        // Calculate code area
        let code_area = Rect {
            x: inner_area.x + line_num_width,
            y: inner_area.y,
            width: inner_area.width.saturating_sub(line_num_width),
            height: inner_area.height,
        };
        
        // Render line numbers
        for i in 0..visible_lines {
            let line_idx = scroll + i;
            let line_num = line_idx + 1; // 1-indexed
            
            let is_issue_line = line_num >= self.issue_line_start && line_num <= self.issue_line_end;
            
            // Render line number with appropriate style
            let line_num_style = if is_issue_line {
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::DarkGray)
            };
            
            let line_num_text = format!("{:>width$}", line_num, width = (line_num_width - 1) as usize);
            buf.set_string(
                inner_area.x,
                inner_area.y + i as u16,
                line_num_text,
                line_num_style,
            );
        }
        
        // Render code
        for i in 0..visible_lines {
            let line_idx = scroll + i;
            
            if line_idx < highlighted.len() {
                let line = &highlighted[line_idx];
                let is_issue_line = (line_idx + 1) >= self.issue_line_start && (line_idx + 1) <= self.issue_line_end;
                
                // Render line spans
                if is_issue_line {
                    // Highlight the background of issue lines
                    buf.set_style(
                        Rect {
                            x: code_area.x,
                            y: code_area.y + i as u16,
                            width: code_area.width,
                            height: 1,
                        },
                        Style::default().bg(Color::DarkRed),
                    );
                }
                
                let mut x_offset = 0;
                for span in line.spans.iter() {
                    let text = span.content.as_ref();
                    let style = span.style;
                    
                    // Use the original style but override background for issue lines
                    let style = if is_issue_line {
                        style.patch(Style::default().bg(Color::DarkRed))
                    } else {
                        style
                    };
                    
                    buf.set_string(
                        code_area.x + x_offset,
                        code_area.y + i as u16,
                        text,
                        style,
                    );
                    
                    x_offset += text.width() as u16;
                }
            }
        }
    }
}

/// Widget for displaying a syntax tree
pub struct SyntaxTreeView<'a> {
    /// The syntax tree
    tree: &'a Tree,
    
    /// The source code
    code: &'a str,
}

impl<'a> SyntaxTreeView<'a> {
    /// Create a new syntax tree view
    pub fn new(tree: &'a Tree, code: &'a str) -> Self {
        Self { tree, code }
    }
}

impl<'a> Widget for SyntaxTreeView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create a block for the syntax tree view
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Syntax Tree");
        
        // Calculate inner area
        let inner_area = block.inner(area);
        
        // Render block
        block.render(area, buf);
        
        // Format the tree
        let formatter = TreeFormatter::new(self.tree, self.code);
        let spans = formatter.as_spans();
        
        // Render tree
        for (i, line) in spans.iter().enumerate() {
            if i < inner_area.height as usize {
                let mut x_offset = 0;
                for span in line.spans.iter() {
                    let text = span.content.as_ref();
                    let style = span.style;
                    
                    buf.set_string(
                        inner_area.x + x_offset,
                        inner_area.y + i as u16,
                        text,
                        style,
                    );
                    
                    x_offset += text.width() as u16;
                }
            } else {
                break;
            }
        }
    }
}

/// Widget for displaying issue details
pub struct IssuePanel<'a> {
    /// The issue to display
    issue: &'a ValidationIssue,
}

impl<'a> IssuePanel<'a> {
    /// Create a new issue panel
    pub fn new(issue: &'a ValidationIssue) -> Self {
        Self { issue }
    }
}

impl<'a> Widget for IssuePanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create a block for the issue panel
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Issue: {}", self.issue.issue_type));
        
        // Calculate inner area
        let inner_area = block.inner(area);
        
        // Render block
        block.render(area, buf);
        
        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(3), // Issue summary
                Constraint::Length(3), // Location
                Constraint::Min(0),    // Details
            ].as_ref())
            .split(inner_area);
        
        // Render issue summary
        let severity_style = match self.issue.severity.as_str() {
            "error" => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            "warning" => Style::default().fg(Color::Yellow),
            "info" => Style::default().fg(Color::Blue),
            _ => Style::default().fg(Color::White),
        };
        
        let summary = Paragraph::new(vec![
            Spans::from(vec![
                Span::styled("Type: ", Style::default().fg(Color::Gray)),
                Span::styled(&self.issue.issue_type, Style::default().fg(Color::White)),
            ]),
            Spans::from(vec![
                Span::styled("Severity: ", Style::default().fg(Color::Gray)),
                Span::styled(&self.issue.severity, severity_style),
            ]),
        ]);
        
        // Render location
        let location = Paragraph::new(vec![
            Spans::from(vec![
                Span::styled("File: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    self.issue.file_path.display().to_string(),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
            Spans::from(vec![
                Span::styled("Line: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    format!("{}-{}", self.issue.line_start, self.issue.line_end),
                    Style::default().fg(Color::Cyan),
                ),
            ]),
        ]);
        
        // Render details
        let description = self.issue.description.clone().unwrap_or_else(|| "No description available".to_string());
        let details = Paragraph::new(description)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::TOP).title("Details"));
        
        // Render components
        summary.render(chunks[0], buf);
        location.render(chunks[1], buf);
        details.render(chunks[2], buf);
    }
}

/// Widget for displaying an action menu
pub struct ActionMenu<'a> {
    /// The issue to display actions for
    issue: &'a ValidationIssue,
    
    /// Current state of the issue
    issue_state: &'a IssueState,
}

impl<'a> ActionMenu<'a> {
    /// Create a new action menu
    pub fn new(issue: &'a ValidationIssue, issue_state: &'a IssueState) -> Self {
        Self { issue, issue_state }
    }
}

impl<'a> Widget for ActionMenu<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Create a block for the action menu
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Actions: {}", self.issue_state.get_status()));
        
        // Calculate inner area
        let inner_area = block.inner(area);
        
        // Render block
        block.render(area, buf);
        
        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(2),  // Current status
                Constraint::Min(10),    // Fix options
                Constraint::Length(4),  // Actions
            ].as_ref())
            .split(inner_area);
        
        // Render current status
        let status_style = match self.issue_state.action {
            IssueAction::Pending => Style::default().fg(Color::Gray),
            IssueAction::Fix => Style::default().fg(Color::Green),
            IssueAction::Ignore => Style::default().fg(Color::Yellow),
            IssueAction::Defer => Style::default().fg(Color::Blue),
        };
        
        let status = Paragraph::new(Spans::from(vec![
            Span::styled("Status: ", Style::default().fg(Color::Gray)),
            Span::styled(self.issue_state.get_status(), status_style),
        ]));
        
        // Render fix options
        let fix_options: Vec<ListItem> = self.issue_state.fix_options
            .iter()
            .enumerate()
            .map(|(i, option)| {
                let is_selected = self.issue_state.selected_fix.map_or(false, |idx| idx == i);
                let number_style = if is_selected {
                    Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Yellow)
                };
                
                let confidence_style = if option.confidence > 80 {
                    Style::default().fg(Color::Green)
                } else if option.confidence > 50 {
                    Style::default().fg(Color::Yellow)
                } else {
                    Style::default().fg(Color::Red)
                };
                
                ListItem::new(Spans::from(vec![
                    Span::styled(format!("{}. ", i + 1), number_style),
                    Span::styled(&option.description, Style::default()),
                    Span::styled(format!(" ({}% confidence)", option.confidence), confidence_style),
                ]))
            })
            .collect();
        
        let fix_options_list = List::new(fix_options)
            .block(Block::default().borders(Borders::TOP).title("Available Fixes"));
        
        // Render actions
        let actions = vec![
            Spans::from(vec![
                Span::styled("f", Style::default().fg(Color::Yellow)),
                Span::raw(": Fix  "),
                Span::styled("i", Style::default().fg(Color::Yellow)),
                Span::raw(": Ignore  "),
                Span::styled("d", Style::default().fg(Color::Yellow)),
                Span::raw(": Defer"),
            ]),
            Spans::from(vec![
                Span::styled("1-9", Style::default().fg(Color::Yellow)),
                Span::raw(": Select fix  "),
                Span::styled("a", Style::default().fg(Color::Yellow)),
                Span::raw(": Auto-fix  "),
                Span::styled("c", Style::default().fg(Color::Yellow)),

