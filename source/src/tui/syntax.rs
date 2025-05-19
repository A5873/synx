//! Syntax highlighting and tree-sitter integration
//!
//! This module provides functionality for syntax highlighting and
//! tree-sitter integration for the TUI.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Result, Context, anyhow};
use log::{debug, info};
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::{SyntaxSet, SyntaxReference, SyntaxDefinition};
use syntect::util::LinesWithEndings;
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tree_sitter::{Parser, Tree, Language, Node, TreeCursor};

/// Syntax highlighter using syntect
pub struct SyntaxHighlighter {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    current_theme: Theme,
}

impl SyntaxHighlighter {
    /// Create a new syntax highlighter
    pub fn new() -> Result<Self> {
        let syntax_set = SyntaxSet::load_defaults_newlines();
        let theme_set = ThemeSet::load_defaults();
        
        // Use a dark theme (Solarized Dark)
        let current_theme = theme_set.themes["Solarized (dark)"].clone();
        
        Ok(Self {
            syntax_set,
            theme_set,
            current_theme,
        })
    }
    
    /// Get syntax definition for a file
    pub fn get_syntax_for_file(&self, path: &Path) -> Option<&SyntaxReference> {
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        self.syntax_set.find_syntax_by_extension(extension)
    }
    
    /// Highlight code for display
    pub fn highlight(&self, code: &str, path: &Path) -> Vec<Spans> {
        let syntax = match self.get_syntax_for_file(path) {
            Some(syntax) => syntax,
            None => return self.plain_text(code),
        };
        
        let mut h = self.syntax_set.build_highlighter(&self.current_theme);
        
        let mut spans_vec = Vec::new();
        
        for line in LinesWithEndings::from(code) {
            let mut line_spans = Vec::new();
            
            // Simple implementation just uses default colors for now
            // In a real implementation, we would map syntect styles to TUI styles
            line_spans.push(Span::styled(line.to_string(), Style::default().fg(Color::White)));
            
            spans_vec.push(Spans::from(line_spans));
        }
        
        spans_vec
    }
    
    /// Generate plain text spans for code without highlighting
    fn plain_text(&self, code: &str) -> Vec<Spans> {
        code.lines()
            .map(|line| {
                Spans::from(vec![
                    Span::styled(line, Style::default().fg(Color::White)),
                ])
            })
            .collect()
    }
}

/// Tree formatter for displaying the AST
pub struct TreeFormatter {
    tree: Tree,
    code: String,
}

impl TreeFormatter {
    /// Create a new tree formatter
    pub fn new(tree: &Tree, code: &str) -> Self {
        Self {
            tree: tree.clone(),
            code: code.to_string(),
        }
    }
    
    /// Format the tree as a string
    pub fn format(&self) -> String {
        let mut result = String::new();
        let root_node = self.tree.root_node();
        
        self.format_node(&mut result, root_node, 0);
        
        result
    }
    
    /// Format a node and its children
    fn format_node(&self, result: &mut String, node: Node, depth: usize) {
        let indent = "  ".repeat(depth);
        let node_text = if node.child_count() == 0 {
            let start = node.start_byte();
            let end = node.end_byte();
            if start < self.code.len() && end <= self.code.len() {
                format!(" \"{}\"", &self.code[start..end])
            } else {
                String::new()
            }
        } else {
            String::new()
        };
        
        result.push_str(&format!(
            "{}{} ({}-{}){}",
            indent,
            node.kind(),
            node.start_position().row + 1,
            node.end_position().row + 1,
            node_text
        ));
        result.push('\n');
        
        let child_count = node.child_count();
        for i in 0..child_count {
            if let Some(child) = node.child(i) {
                self.format_node(result, child, depth + 1);
            }
        }
    }
    
    /// Get a Vec of Spans for TUI display
    pub fn as_spans(&self) -> Vec<Spans> {
        let formatted = self.format();
        
        formatted
            .lines()
            .map(|line| {
                // Highlight different parts of the tree with different colors
                if line.contains("ERROR") {
                    Spans::from(Span::styled(line, Style::default().fg(Color::Red)))
                } else if line.contains("identifier") || line.contains("string") {
                    Spans::from(Span::styled(line, Style::default().fg(Color::Yellow)))
                } else if line.contains("function") || line.contains("method") {
                    Spans::from(Span::styled(line, Style::default().fg(Color::Green)))
                } else {
                    Spans::from(Span::styled(line, Style::default().fg(Color::White)))
                }
            })
            .collect()
    }
}

/// Parse a file using the appropriate tree-sitter parser
pub fn parse_file(path: &Path, parsers: &HashMap<String, Parser>) -> Result<Option<Tree>> {
    let extension = path.extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    
    let parser = match parsers.get(&extension) {
        Some(parser) => parser,
        None => return Ok(None),
    };
    
    let code = std::fs::read_to_string(path)
        .context(format!("Failed to read file: {}", path.display()))?;
    
    match parser.parse(&code, None) {
        Some(tree) => Ok(Some(tree)),
        None => Ok(None),
    }
}

/// Create a Rust parser
pub fn create_rust_parser() -> Result<Parser> {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_rust::language())
        .map_err(|e| anyhow!("Failed to create Rust parser: {}", e))?;
    Ok(parser)
}

/// Create a Python parser
pub fn create_python_parser() -> Result<Parser> {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_python::language())
        .map_err(|e| anyhow!("Failed to create Python parser: {}", e))?;
    Ok(parser)
}

/// Create a JavaScript parser
pub fn create_javascript_parser() -> Result<Parser> {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_javascript::language())
        .map_err(|e| anyhow!("Failed to create JavaScript parser: {}", e))?;
    Ok(parser)
}

/// Create a TypeScript parser
pub fn create_typescript_parser() -> Result<Parser> {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_typescript::language_typescript())
        .map_err(|e| anyhow!("Failed to create TypeScript parser: {}", e))?;
    Ok(parser)
}

/// Create a C parser
pub fn create_c_parser() -> Result<Parser> {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_c::language())
        .map_err(|e| anyhow!("Failed to create C parser: {}", e))?;
    Ok(parser)
}

/// Create a C++ parser
pub fn create_cpp_parser() -> Result<Parser> {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_cpp::language())
        .map_err(|e| anyhow!("Failed to create C++ parser: {}", e))?;
    Ok(parser)
}

/// Create a Go parser
pub fn create_go_parser() -> Result<Parser> {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_go::language())
        .map_err(|e| anyhow!("Failed to create Go parser: {}", e))?;
    Ok(parser)
}

/// Create a Java parser
pub fn create_java_parser() -> Result<Parser> {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_java::language())
        .map_err(|e| anyhow!("Failed to create Java parser: {}", e))?;
    Ok(parser)
}

