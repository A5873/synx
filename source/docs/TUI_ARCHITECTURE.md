# TUI Architecture Documentation

This document describes the technical architecture and implementation details of Synx's Terminal User Interface (TUI).

## Overview

The TUI is built using the `tui-rs` crate (now maintained as `ratatui`) with `crossterm` for cross-platform terminal handling. It provides an interactive interface for reviewing and fixing code issues.

## Architecture Components

### Core Modules

```
src/tui/
├── mod.rs              # Main TUI application and event loop
├── syntax.rs           # Syntax highlighting and tree-sitter integration
├── issue_state.rs      # Issue state management and tracking
└── widgets.rs          # Custom TUI widgets
```

## Core Components

### 1. TuiApp (`src/tui/mod.rs`)

**Purpose:** Main application controller and event loop manager.

**Key Responsibilities:**
- Terminal initialization and cleanup
- Event handling and keyboard input processing
- Application state management
- UI rendering coordination

**Key Structures:**
```rust
pub struct TuiApp {
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    state: AppState,
    syntax_highlighter: SyntaxHighlighter,
    parsers: HashMap<String, Parser>,
}

pub struct AppState {
    issue_files: Vec<PathBuf>,
    current_file: usize,
    issues: Vec<ValidationIssue>,
    current_issue: usize,
    file_content: String,
    syntax_tree: Option<Tree>,
    active_tab: Tab,
    issue_states: HashMap<String, IssueState>,
    // ... additional state fields
}
```

### 2. SyntaxHighlighter (`src/tui/syntax.rs`)

**Purpose:** Syntax highlighting using `syntect` and AST visualization via `tree-sitter`.

**Key Features:**
- Multi-language syntax highlighting (Rust, Python, JavaScript, TypeScript, C/C++, Go, Java)
- Tree-sitter parser integration for AST generation
- Syntax tree formatting and visualization

**Parser Setup:**
```rust
// Parsers are initialized per language
parsers.insert("rust", create_rust_parser()?);
parsers.insert("python", create_python_parser()?);
// ... other languages
```

### 3. IssueState (`src/tui/issue_state.rs`)

**Purpose:** Issue state tracking and management throughout the interactive session.

**Key Structures:**
```rust
pub struct IssueState {
    pub issue: ValidationIssue,
    pub action: IssueAction,
    pub is_fixed: bool,
    pub selected_fix: Option<usize>,
    pub custom_fixes: Vec<String>,
    pub notes: String,
    pub timestamp: std::time::SystemTime,
}

pub enum IssueAction {
    Pending,
    Fix,
    Ignore,
    Defer,
}
```

### 4. Widgets (`src/tui/widgets.rs`)

**Purpose:** Custom UI components for different content types.

**Widget Types:**
- `CodeView`: Syntax-highlighted code display with issue highlighting
- `IssuePanel`: Issue details and metadata display
- `SyntaxTreeView`: AST visualization widget
- `ActionMenu`: Interactive action buttons and options
- `ExplanationPanel`: Rule explanations with examples

## UI Layout Architecture

### Tab-Based Interface

The interface uses a tabbed layout with four main views:

```
┌─────────────────────────────────────────────────────┐
│ [Issues] [Syntax Tree] [Actions] [Explanation]      │ <- Tab Bar
├─────────────────────────────────────────────────────┤
│                                                     │
│                Main Content Area                    │ <- Dynamic Content
│                                                     │
├─────────────────────────────────────────────────────┤
│ File 1/5: main.rs | Issue 2/7 (Warning) | q: Quit  │ <- Status Bar
└─────────────────────────────────────────────────────┘
```

### Layout Management

Uses `tui-rs`'s constraint-based layout system:

```rust
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .margin(1)
    .constraints([
        Constraint::Length(3),   // Tabs
        Constraint::Min(0),      // Main content
        Constraint::Length(3),   // Status bar
    ])
    .split(f.size());
```

## Event Handling System

### Event Loop

The main event loop follows this pattern:

```rust
while !self.state.should_exit {
    // 1. Render UI
    self.terminal.draw(|f| {
        Self::draw_ui_static(f, &state_clone, &highlighter_clone);
    })?;
    
    // 2. Poll for events
    if crossterm::event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            // 3. Handle keyboard input
            self.handle_key_event(key)?;
        }
    }
}
```

### Keyboard Mapping

Context-sensitive keyboard handling based on active tab:

| Context | Key | Action |
|---------|-----|--------|
| Global | `q` | Quit |
| Global | `Tab` | Switch tabs |
| Global | `n`/`p` | Navigate issues |
| Actions Tab | `1`/`2`/`3` | Apply specific fixes |
| Explanation Tab | `x` | Toggle examples |

## Rendering System

### Static Rendering Pattern

To avoid Rust borrowing issues with closures, the rendering uses static methods:

```rust
// Instead of:
self.terminal.draw(|f| self.draw_ui(f))?; // Borrowing error

// We use:
let state_clone = self.state.clone();
let highlighter_clone = self.syntax_highlighter.clone();
self.terminal.draw(|f| {
    Self::draw_ui_static(f, &state_clone, &highlighter_clone);
})?;
```

### Widget Rendering

Each widget implements the `Widget` trait from `tui-rs`:

```rust
impl<'a> Widget for CodeView<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Render syntax-highlighted code with issue markers
    }
}
```

## State Management

### Issue Tracking

Issues are tracked with unique IDs and state transitions:

```rust
// Generate unique IDs for issues
for issue in &issues {
    let id = Uuid::new_v4().to_string();
    issue_states.insert(id, IssueState::new(issue.clone()));
}
```

### Navigation History

Navigation maintains a history stack for backtracking:

```rust
// Save current position before navigation
self.state.navigation_history.push((self.state.current_file, self.state.current_issue));

// Restore previous position
if let Some((file_idx, issue_idx)) = self.state.navigation_history.pop() {
    // Navigate back
}
```

## Integration Points

### Tree-sitter Integration

Language parsers are initialized and cached:

```rust
pub fn create_rust_parser() -> Result<Parser> {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_rust::language())?;
    Ok(parser)
}
```

### Syntect Integration

Syntax highlighting uses `syntect` for token-level highlighting:

```rust
pub fn highlight<'a>(&self, code: &'a str, path: &Path) -> Vec<Spans<'a>> {
    let syntax = self.get_syntax_for_file(path)
        .unwrap_or_else(|| self.syntax_set.find_syntax_plain_text());
    
    // Generate highlighted spans
}
```

### Validation Integration

TUI accepts validation reports in standardized format:

```rust
pub struct ValidationReport {
    pub file_issues: HashMap<PathBuf, Vec<ValidationIssue>>,
}

pub struct ValidationIssue {
    pub file_path: PathBuf,
    pub issue_type: String,
    pub severity: IssueSeverity,
    pub message: String,
    pub line_start: usize,
    pub line_end: usize,
    pub suggested_fix: Option<String>,
    pub context: HashMap<String, String>,
}
```

## Performance Considerations

### Memory Management

- **Cloning Strategy**: State is cloned for rendering to avoid borrowing issues
- **Parser Caching**: Tree-sitter parsers are initialized once and reused
- **Selective Loading**: Only current file content is kept in memory

### Rendering Optimization

- **Static Methods**: Avoid complex borrowing in render closures  
- **Lazy Evaluation**: Syntax trees generated on-demand
- **Scrolling**: Virtual scrolling for large files

## Error Handling

### Graceful Degradation

```rust
// Fallback to plain text if syntax highlighting fails
let spans = match self.syntax_highlighter.highlight(code, path) {
    Ok(highlighted) => highlighted,
    Err(_) => self.syntax_highlighter.plain_text(code),
};
```

### Terminal Cleanup

Ensures terminal is properly restored on exit or panic:

```rust
// Always restore terminal state
disable_raw_mode()?;
execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
self.terminal.show_cursor()?;
```

## Extension Points

### Adding New Widgets

1. Implement the `Widget` trait
2. Add to the main rendering pipeline
3. Handle specific events if needed

### Adding New Languages

1. Add tree-sitter parser in `syntax.rs`
2. Add language detection logic
3. Add syntax highlighting rules

### Custom Actions

1. Extend `IssueAction` enum
2. Add keyboard handlers
3. Implement action logic in event handlers

## Testing Considerations

### Unit Testing

- Widget rendering logic
- State management functions  
- Event handling logic

### Integration Testing

- Full TUI workflows
- Keyboard interaction sequences
- Error scenarios

### Manual Testing

- Cross-platform terminal compatibility
- Different terminal sizes
- Various file types and sizes

This architecture provides a solid foundation for interactive code issue resolution while maintaining good separation of concerns and extensibility.
