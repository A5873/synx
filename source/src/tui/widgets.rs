//! TUI widgets for code visualization and issue management

use std::path::Path;
use tui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Widget, Wrap},
};
use super::ValidationIssue;
use crate::analysis::IssueSeverity;
use super::{
    syntax::SyntaxHighlighter,
    issue_state::IssueState,
    LintRule,
};

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
        
        // Split content into lines for display
        let lines: Vec<&str> = self.content.lines().collect();
        let total_lines = lines.len();
        
        // Calculate visible lines
        let height = inner_area.height as usize;
        let scroll = self.scroll as usize;
        let scroll = scroll.min(total_lines.saturating_sub(height));
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
        
        // Render line numbers and code
        for i in 0..visible_lines {
            let line_idx = scroll + i;
            let line_num = line_idx + 1; // 1-indexed
            
            let is_issue_line = line_num >= self.issue_line_start && line_num <= self.issue_line_end;
            
            // Render line number
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
            
            // Render code line
            if line_idx < lines.len() {
                let line_content = lines[line_idx];
                let line_style = if is_issue_line {
                    Style::default().bg(Color::DarkGray)
                } else {
                    Style::default()
                };
                
                buf.set_string(
                    code_area.x,
                    code_area.y + i as u16,
                    line_content,
                    line_style,
                );
            }
        }
    }
}

/// Widget for displaying issue details
pub struct IssuePanel<'a> {
    issue: &'a ValidationIssue,
}

impl<'a> IssuePanel<'a> {
    pub fn new(issue: &'a ValidationIssue) -> Self {
        Self { issue }
    }
}

impl<'a> Widget for IssuePanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Issue Details");
        
        let inner = block.inner(area);
        block.render(area, buf);
        
        // Create issue info text
        let info_text = vec![
            Spans::from(vec![
                Span::styled("Type: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&self.issue.issue_type),
            ]),
            Spans::from(vec![
                Span::styled("Severity: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::styled(
                    format!("{:?}", self.issue.severity),
                    match self.issue.severity {
                        IssueSeverity::Critical => Style::default().fg(Color::Red),
                        IssueSeverity::High => Style::default().fg(Color::Red),
                        IssueSeverity::Medium => Style::default().fg(Color::Yellow),
                        IssueSeverity::Low => Style::default().fg(Color::Blue),
                    }
                ),
            ]),
            Spans::from(vec![
                Span::styled("Line: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(format!("{}-{}", self.issue.line_start, self.issue.line_end)),
            ]),
            Spans::from(vec![Span::raw("")]), // Empty line
            Spans::from(vec![
                Span::styled("Description:", Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Spans::from(vec![Span::raw(&self.issue.message)]),
        ];
        
        let paragraph = Paragraph::new(info_text)
            .wrap(Wrap { trim: true });
        
        paragraph.render(inner, buf);
    }
}

/// Widget for displaying syntax tree
pub struct SyntaxTreeView<'a> {
    tree: &'a tree_sitter::Tree,
    content: &'a str,
}

impl<'a> SyntaxTreeView<'a> {
    pub fn new(tree: &'a tree_sitter::Tree, content: &'a str) -> Self {
        Self { tree, content }
    }
}

impl<'a> Widget for SyntaxTreeView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Syntax Tree");
        
        let inner = block.inner(area);
        block.render(area, buf);
        
        // Simplified tree rendering
        let tree_text = vec![
            Spans::from(vec![Span::raw("Root Node:")]),
            Spans::from(vec![Span::raw(format!("  Kind: {}", self.tree.root_node().kind()))]),
            Spans::from(vec![Span::raw(format!("  Children: {}", self.tree.root_node().child_count()))]),
            Spans::from(vec![Span::raw("")]),
            Spans::from(vec![Span::raw("Tree structure visualization")]),
            Spans::from(vec![Span::raw("would be implemented here...")]),
        ];
        
        let paragraph = Paragraph::new(tree_text)
            .wrap(Wrap { trim: true });
        
        paragraph.render(inner, buf);
    }
}

/// Widget for displaying action menu
pub struct ActionMenu<'a> {
    issue: &'a ValidationIssue,
    issue_state: &'a IssueState,
}

impl<'a> ActionMenu<'a> {
    pub fn new(issue: &'a ValidationIssue, issue_state: &'a IssueState) -> Self {
        Self { issue, issue_state }
    }
}

impl<'a> Widget for ActionMenu<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title("Actions");
        
        let inner = block.inner(area);
        block.render(area, buf);
        
        // Create action items
        let action_items = vec![
            ListItem::new(Spans::from(vec![
                Span::styled("[f] ", Style::default().fg(Color::Green)),
                Span::raw("Fix this issue"),
            ])),
            ListItem::new(Spans::from(vec![
                Span::styled("[i] ", Style::default().fg(Color::Yellow)),
                Span::raw("Ignore this issue"),
            ])),
            ListItem::new(Spans::from(vec![
                Span::styled("[a] ", Style::default().fg(Color::Blue)),
                Span::raw("Apply automatic fix"),
            ])),
        ];
        
        let action_list = List::new(action_items);
        action_list.render(inner, buf);
    }
}

/// Widget for displaying rule explanation
pub struct ExplanationPanel<'a> {
    rule: &'a LintRule,
    show_examples: bool,
}

impl<'a> ExplanationPanel<'a> {
    pub fn new(rule: &'a LintRule, show_examples: bool) -> Self {
        Self { rule, show_examples }
    }
}

impl<'a> Widget for ExplanationPanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .title(format!("Rule Explanation - {}", self.rule.code));
        
        let inner = block.inner(area);
        block.render(area, buf);
        
        let mut explanation_text = vec![
            Spans::from(vec![
                Span::styled("Title: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(&self.rule.title),
            ]),
            Spans::from(vec![Span::raw("")]), // Empty line
            Spans::from(vec![
                Span::styled("Description:", Style::default().add_modifier(Modifier::BOLD)),
            ]),
            Spans::from(vec![Span::raw(&self.rule.description)]),
        ];
        
        if self.show_examples && !self.rule.examples.is_empty() {
            explanation_text.push(Spans::from(vec![Span::raw("")])); // Empty line
            explanation_text.push(Spans::from(vec![
                Span::styled("Examples:", Style::default().add_modifier(Modifier::BOLD)),
            ]));
            
            for example in &self.rule.examples {
                explanation_text.push(Spans::from(vec![Span::raw(format!("  {}", example))]));
            }
        }
        
        let paragraph = Paragraph::new(explanation_text)
            .wrap(Wrap { trim: true });
        
        paragraph.render(inner, buf);
    }
}
