use std::path::Path;
use std::fs;
use colored::*;
use console::Emoji;
use anyhow::Result;

static ERROR_MARK: Emoji<'_, '_> = Emoji("❌", "x");
static WARNING_MARK: Emoji<'_, '_> = Emoji("⚠️", "!");
static INFO_MARK: Emoji<'_, '_> = Emoji("ℹ️", "i");
static LINE_MARK: Emoji<'_, '_> = Emoji("│", "|");

/// Represents a validation error with context
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub file_path: String,
    pub error_type: ErrorType,
    pub message: String,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub code: Option<String>,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ErrorType {
    SyntaxError,
    TypeError,
    Warning,
    Lint,
    CompileError,
    RuntimeError,
}

impl ErrorType {
    pub fn color(&self) -> Color {
        match self {
            ErrorType::SyntaxError | ErrorType::CompileError => Color::Red,
            ErrorType::TypeError => Color::Magenta,
            ErrorType::Warning => Color::Yellow,
            ErrorType::Lint => Color::Cyan,
            ErrorType::RuntimeError => Color::BrightRed,
        }
    }

    pub fn emoji(&self) -> &'static Emoji<'static, 'static> {
        match self {
            ErrorType::SyntaxError | ErrorType::CompileError | ErrorType::RuntimeError => &ERROR_MARK,
            ErrorType::TypeError => &ERROR_MARK,
            ErrorType::Warning => &WARNING_MARK,
            ErrorType::Lint => &INFO_MARK,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            ErrorType::SyntaxError => "Syntax Error",
            ErrorType::TypeError => "Type Error",
            ErrorType::Warning => "Warning",
            ErrorType::Lint => "Lint",
            ErrorType::CompileError => "Compile Error",
            ErrorType::RuntimeError => "Runtime Error",
        }
    }
}

/// Enhanced error display with colorization and context
pub struct ErrorDisplay<'a> {
    pub error: &'a ValidationError,
    pub show_code_context: bool,
    pub context_lines: usize,
}

impl<'a> ErrorDisplay<'a> {
    pub fn new(error: &'a ValidationError) -> Self {
        Self {
            error,
            show_code_context: true,
            context_lines: 2,
        }
    }

    pub fn with_context(mut self, show: bool) -> Self {
        self.show_code_context = show;
        self
    }

    pub fn with_context_lines(mut self, lines: usize) -> Self {
        self.context_lines = lines;
        self
    }

    pub fn display(&self) -> Result<()> {
        let error = self.error;
        let error_type = &error.error_type;
        
        // Header with error type and emoji
        println!("\n{} {} {}", 
            error_type.emoji(),
            error_type.name().color(error_type.color()).bold(),
            error.file_path.bright_white().underline()
        );

        // Location information
        if let (Some(line), Some(column)) = (error.line, error.column) {
            println!("   {} {}:{}",
                "at".bright_black(),
                line.to_string().bright_blue(),
                column.to_string().bright_blue()
            );
        }

        // Error message
        println!("   {} {}", 
            LINE_MARK.to_string().bright_black(),
            error.message.color(error_type.color())
        );

        // Show code context if available and requested
        if self.show_code_context && error.line.is_some() {
            self.display_code_context()?;
        }

        // Show suggestion if available
        if let Some(suggestion) = &error.suggestion {
            println!("   {} {}: {}", 
                INFO_MARK,
                "Suggestion".bright_green().bold(),
                suggestion.bright_white()
            );
        }

        // Show error code if available
        if let Some(code) = &error.code {
            println!("   {} {}: {}", 
                "Code".bright_black(),
                code.bright_black(),
                ""
            );
        }

        Ok(())
    }

    fn display_code_context(&self) -> Result<()> {
        let error = self.error;
        let line_num = error.line.unwrap_or(1);
        let file_content = fs::read_to_string(&error.file_path)?;
        let lines: Vec<&str> = file_content.lines().collect();

        if lines.is_empty() {
            return Ok(());
        }

        let start_line = line_num.saturating_sub(self.context_lines + 1);
        let end_line = (line_num + self.context_lines).min(lines.len());
        let error_line_idx = line_num.saturating_sub(1);

        println!();
        
        for (idx, line) in lines.iter().enumerate().take(end_line).skip(start_line) {
            let line_number = idx + 1;
            let is_error_line = idx == error_line_idx;
            
            if is_error_line {
                // Highlight the error line
                println!(" {} {} {}", 
                    format!("{:>4}", line_number).red().bold(),
                    LINE_MARK.to_string().red(),
                    line.bright_white().on_red()
                );
                
                // Show column indicator if available
                if let Some(column) = error.column {
                    let spaces = " ".repeat(7 + column.saturating_sub(1));
                    println!("{}{}",
                        spaces,
                        "^".repeat(1).red().bold()
                    );
                }
            } else {
                // Regular context line
                println!(" {} {} {}", 
                    format!("{:>4}", line_number).bright_black(),
                    LINE_MARK.to_string().bright_black(),
                    highlight_syntax(line, &get_language_from_path(&error.file_path))
                );
            }
        }
        
        println!();
        Ok(())
    }
}

/// Parse compiler/linter output and extract structured errors
pub fn parse_validation_output(
    file_path: &Path,
    output: &str,
    language: &str,
) -> Vec<ValidationError> {
    match language {
        "rust" => parse_rust_errors(file_path, output),
        "python" => parse_python_errors(file_path, output),
        "javascript" | "typescript" => parse_js_ts_errors(file_path, output),
        "c" | "cpp" => parse_c_cpp_errors(file_path, output),
        "java" => parse_java_errors(file_path, output),
        "go" => parse_go_errors(file_path, output),
        _ => parse_generic_errors(file_path, output),
    }
}

fn parse_rust_errors(file_path: &Path, output: &str) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let path_str = file_path.to_string_lossy().to_string();
    
    for line in output.lines() {
        if line.contains("error:") || line.contains("warning:") {
            let error_type = if line.contains("error:") {
                ErrorType::CompileError
            } else {
                ErrorType::Warning
            };
            
            // Parse Rust error format: file:line:column: error: message
            if let Some(location_end) = line.find(": ") {
                let location_part = &line[..location_end];
                let message_part = &line[location_end + 2..];
                
                let (line_num, column_num) = parse_location(location_part);
                
                errors.push(ValidationError {
                    file_path: path_str.clone(),
                    error_type,
                    message: message_part.to_string(),
                    line: line_num,
                    column: column_num,
                    code: None,
                    suggestion: None,
                });
            }
        }
    }
    
    if errors.is_empty() && !output.is_empty() {
        errors.push(ValidationError {
            file_path: path_str,
            error_type: ErrorType::CompileError,
            message: output.to_string(),
            line: None,
            column: None,
            code: None,
            suggestion: None,
        });
    }
    
    errors
}

fn parse_python_errors(file_path: &Path, output: &str) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let path_str = file_path.to_string_lossy().to_string();
    
    for line in output.lines() {
        if line.trim().starts_with("File") && line.contains("line") {
            // Python error format: File "filename", line N
            if let Some(line_start) = line.find("line ") {
                let line_part = &line[line_start + 5..];
                if let Some(line_end) = line_part.find(',') {
                    if let Ok(line_num) = line_part[..line_end].parse::<usize>() {
                        errors.push(ValidationError {
                            file_path: path_str.clone(),
                            error_type: ErrorType::SyntaxError,
                            message: "Python syntax error".to_string(),
                            line: Some(line_num),
                            column: None,
                            code: None,
                            suggestion: None,
                        });
                    }
                }
            }
        }
    }
    
    if errors.is_empty() && !output.is_empty() {
        errors.push(ValidationError {
            file_path: path_str,
            error_type: ErrorType::SyntaxError,
            message: output.to_string(),
            line: None,
            column: None,
            code: None,
            suggestion: None,
        });
    }
    
    errors
}

fn parse_js_ts_errors(file_path: &Path, output: &str) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let path_str = file_path.to_string_lossy().to_string();
    
    for line in output.lines() {
        if line.contains("SyntaxError") || line.contains("TypeError") {
            let error_type = if line.contains("SyntaxError") {
                ErrorType::SyntaxError
            } else {
                ErrorType::TypeError
            };
            
            errors.push(ValidationError {
                file_path: path_str.clone(),
                error_type,
                message: line.to_string(),
                line: None,
                column: None,
                code: None,
                suggestion: None,
            });
        }
    }
    
    if errors.is_empty() && !output.is_empty() {
        errors.push(ValidationError {
            file_path: path_str,
            error_type: ErrorType::SyntaxError,
            message: output.to_string(),
            line: None,
            column: None,
            code: None,
            suggestion: None,
        });
    }
    
    errors
}

fn parse_c_cpp_errors(file_path: &Path, output: &str) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let path_str = file_path.to_string_lossy().to_string();
    
    for line in output.lines() {
        if line.contains(": error:") || line.contains(": warning:") {
            let error_type = if line.contains(": error:") {
                ErrorType::CompileError
            } else {
                ErrorType::Warning
            };
            
            // Parse GCC/Clang format: file:line:column: error: message
            let (line_num, column_num) = parse_location(line);
            
            let message = if let Some(msg_start) = line.find(": error:") {
                &line[msg_start + 8..]
            } else if let Some(msg_start) = line.find(": warning:") {
                &line[msg_start + 10..]
            } else {
                line
            };
            
            errors.push(ValidationError {
                file_path: path_str.clone(),
                error_type,
                message: message.to_string(),
                line: line_num,
                column: column_num,
                code: None,
                suggestion: None,
            });
        }
    }
    
    if errors.is_empty() && !output.is_empty() {
        errors.push(ValidationError {
            file_path: path_str,
            error_type: ErrorType::CompileError,
            message: output.to_string(),
            line: None,
            column: None,
            code: None,
            suggestion: None,
        });
    }
    
    errors
}

fn parse_java_errors(file_path: &Path, output: &str) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let path_str = file_path.to_string_lossy().to_string();
    
    for line in output.lines() {
        if line.contains(".java:") && (line.contains("error:") || line.contains("warning:")) {
            let error_type = if line.contains("error:") {
                ErrorType::CompileError
            } else {
                ErrorType::Warning
            };
            
            let (line_num, column_num) = parse_location(line);
            
            errors.push(ValidationError {
                file_path: path_str.clone(),
                error_type,
                message: line.to_string(),
                line: line_num,
                column: column_num,
                code: None,
                suggestion: None,
            });
        }
    }
    
    if errors.is_empty() && !output.is_empty() {
        errors.push(ValidationError {
            file_path: path_str,
            error_type: ErrorType::CompileError,
            message: output.to_string(),
            line: None,
            column: None,
            code: None,
            suggestion: None,
        });
    }
    
    errors
}

fn parse_go_errors(file_path: &Path, output: &str) -> Vec<ValidationError> {
    let mut errors = Vec::new();
    let path_str = file_path.to_string_lossy().to_string();
    
    for line in output.lines() {
        if line.contains(".go:") {
            let (line_num, column_num) = parse_location(line);
            
            errors.push(ValidationError {
                file_path: path_str.clone(),
                error_type: ErrorType::CompileError,
                message: line.to_string(),
                line: line_num,
                column: column_num,
                code: None,
                suggestion: None,
            });
        }
    }
    
    if errors.is_empty() && !output.is_empty() {
        errors.push(ValidationError {
            file_path: path_str,
            error_type: ErrorType::CompileError,
            message: output.to_string(),
            line: None,
            column: None,
            code: None,
            suggestion: None,
        });
    }
    
    errors
}

fn parse_generic_errors(file_path: &Path, output: &str) -> Vec<ValidationError> {
    if output.is_empty() {
        return Vec::new();
    }
    
    vec![ValidationError {
        file_path: file_path.to_string_lossy().to_string(),
        error_type: ErrorType::CompileError,
        message: output.to_string(),
        line: None,
        column: None,
        code: None,
        suggestion: None,
    }]
}

fn parse_location(line: &str) -> (Option<usize>, Option<usize>) {
    // Try to parse line:column format from compiler output
    let parts: Vec<&str> = line.split(':').collect();
    
    if parts.len() >= 3 {
        let line_num = parts[1].parse::<usize>().ok();
        let column_num = parts[2].parse::<usize>().ok();
        (line_num, column_num)
    } else {
        (None, None)
    }
}

fn get_language_from_path(file_path: &str) -> String {
    if let Some(ext) = file_path.split('.').last() {
        ext.to_lowercase()
    } else {
        "text".to_string()
    }
}

fn highlight_syntax(line: &str, language: &str) -> String {
    // Basic syntax highlighting for common languages
    match language {
        "rust" | "rs" => highlight_rust(line),
        "python" | "py" => highlight_python(line),
        "javascript" | "js" => highlight_javascript(line),
        "c" | "cpp" | "cc" | "cxx" => highlight_c_cpp(line),
        _ => line.to_string(),
    }
}

fn highlight_rust(line: &str) -> String {
    let mut result = line.to_string();
    
    // Keywords
    let keywords = ["fn", "let", "mut", "if", "else", "match", "for", "while", "loop", "impl", "struct", "enum", "pub", "use", "mod"];
    for keyword in &keywords {
        result = result.replace(keyword, &keyword.blue().to_string());
    }
    
    // Strings
    if let Some(start) = result.find('"') {
        if let Some(end) = result[start + 1..].find('"') {
            let string_part = &result[start..=start + end + 1];
            result = result.replace(string_part, &string_part.green().to_string());
        }
    }
    
    result
}

fn highlight_python(line: &str) -> String {
    let mut result = line.to_string();
    
    // Keywords
    let keywords = ["def", "class", "if", "else", "elif", "for", "while", "try", "except", "import", "from", "return"];
    for keyword in &keywords {
        result = result.replace(keyword, &keyword.blue().to_string());
    }
    
    // Strings
    if let Some(start) = result.find('"') {
        if let Some(end) = result[start + 1..].find('"') {
            let string_part = &result[start..=start + end + 1];
            result = result.replace(string_part, &string_part.green().to_string());
        }
    }
    
    result
}

fn highlight_javascript(line: &str) -> String {
    let mut result = line.to_string();
    
    // Keywords
    let keywords = ["function", "var", "let", "const", "if", "else", "for", "while", "return", "class"];
    for keyword in &keywords {
        result = result.replace(keyword, &keyword.blue().to_string());
    }
    
    // Strings
    if let Some(start) = result.find('"') {
        if let Some(end) = result[start + 1..].find('"') {
            let string_part = &result[start..=start + end + 1];
            result = result.replace(string_part, &string_part.green().to_string());
        }
    }
    
    result
}

fn highlight_c_cpp(line: &str) -> String {
    let mut result = line.to_string();
    
    // Keywords
    let keywords = ["int", "char", "float", "double", "void", "if", "else", "for", "while", "return", "struct", "class"];
    for keyword in &keywords {
        result = result.replace(keyword, &keyword.blue().to_string());
    }
    
    // Strings
    if let Some(start) = result.find('"') {
        if let Some(end) = result[start + 1..].find('"') {
            let string_part = &result[start..=start + end + 1];
            result = result.replace(string_part, &string_part.green().to_string());
        }
    }
    
    result
}

/// Display multiple validation errors in a formatted way
pub fn display_validation_errors(errors: &[ValidationError]) -> Result<()> {
    if errors.is_empty() {
        return Ok(());
    }

    println!("\n{} {} found:", 
        ERROR_MARK,
        if errors.len() == 1 { "error" } else { "errors" }.red().bold()
    );

    for (i, error) in errors.iter().enumerate() {
        ErrorDisplay::new(error).display()?;
        
        // Add separator between errors (except for the last one)
        if i < errors.len() - 1 {
            println!("{}", "─".repeat(60).bright_black());
        }
    }

    Ok(())
}
