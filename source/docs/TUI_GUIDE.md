# Synx Interactive TUI Guide

The Synx Terminal User Interface (TUI) provides an interactive way to review and fix code issues, similar to `git add -p` for interactive staging.

## Getting Started

### Launch Interactive Mode

To start the interactive TUI, use the `interactive` command with a validation report:

```bash
synx interactive --report path/to/validation_report.json
```

Or run validation and enter interactive mode directly:

```bash
synx validate --interactive path/to/your/project
```

### Interface Overview

The TUI consists of four main sections:

1. **Tab Bar** - Switch between different views
2. **Main Content Area** - Shows the active tab's content
3. **Status Bar** - Current file/issue info and keyboard shortcuts

## Tabs and Views

### 1. Issues Tab (Default)

**Layout:**
- **Left Panel (60%)**: Syntax-highlighted code with issue location highlighted
- **Right Panel (40%)**: Issue details including:
  - Issue type and severity
  - Description and message
  - Suggested fix (if available)
  - File location and line numbers

**Purpose:** Review code issues in context with their surrounding code.

### 2. Syntax Tree Tab

**Content:** 
- Abstract Syntax Tree (AST) visualization of the current file
- Shows the hierarchical structure of the code
- Useful for understanding complex parsing issues

**Purpose:** Analyze code structure and understand how the parser interprets the code.

### 3. Actions Tab

**Content:**
- Available fix options for the current issue
- Interactive buttons for different fix strategies:
  - **Automatic Fix** - Apply the suggested fix
  - **Fix Option 1/2/3** - Choose from multiple fix alternatives
  - **Custom Fix** - Manual intervention options

**Purpose:** Take action on issues with multiple fix strategies.

### 4. Explanation Tab

**Content:**
- Detailed explanation of the current issue's rule
- **Rule Information:**
  - Rule code and title
  - Detailed description
  - Why this rule matters
- **Code Examples** (toggleable):
  - Bad code examples (what triggers the rule)
  - Good code examples (how to fix it)
  - Real-world scenarios

**Purpose:** Understand why an issue exists and learn best practices.

## Navigation

### Tab Navigation
- **Tab** - Cycle through tabs (Issues → Syntax Tree → Actions → Explanation → Issues...)

### Issue Navigation
- **n** - Next issue
- **p** - Previous issue
- **N** - Next file (skip to first issue in next file)
- **P** - Previous file (skip to last issue in previous file)
- **b** - Back in navigation history

### Scroll Navigation (in code views)
- **↑/↓** - Scroll up/down by one line
- **Page Up/Page Down** - Scroll by 10 lines
- **Home** - Jump to top
- **End** - Jump to bottom

## Issue Management

### Taking Action
- **f** - Fix current issue (apply suggested fix)
- **i** - Ignore current issue (skip without fixing)
- **e** - Show explanation for current issue (switches to Explanation tab)

### In Actions Tab
- **1/2/3** - Apply specific fix option
- **a** - Apply automatic fix (same as 'f' key)

### In Explanation Tab
- **x** - Toggle code examples on/off
- **t** - Browse related rules in the same family
- **e** - Return to issue view

## Status Information

The status bar shows:
- **File Info**: `File 2/15: src/main.rs` - Current file number and path
- **Issue Info**: `Issue 3/7 (Warning)` - Current issue number and severity
- **Keyboard Shortcuts** - Context-sensitive help based on active tab

## Session Management

### Session Results
At the end of each session, you'll see:
- **Fixed Issues**: Issues that were automatically resolved
- **Ignored Issues**: Issues you chose to skip
- **Remaining Issues**: Issues that still need attention

### Persistence
- Issue states are tracked during the session
- Navigation history is maintained
- You can always go back to review previous decisions

## Tips for Effective Use

### 1. Start with High-Severity Issues
Navigate through issues and focus on `Error` and `Warning` level items first.

### 2. Use Explanations for Learning
Press `e` on unfamiliar rule violations to understand the reasoning and see examples.

### 3. Review Before Fixing
Always review the suggested fix in the Issues tab before applying it.

### 4. Use Syntax Tree for Complex Issues
For parsing or structural issues, check the Syntax Tree tab to understand the code structure.

### 5. Batch Similar Issues
When you encounter similar issues across files, develop a fixing strategy and apply it consistently.

## Keyboard Reference

### Universal Commands
| Key | Action |
|-----|--------|
| `q` | Quit the TUI |
| `Tab` | Switch tabs |
| `↑/↓` | Scroll up/down |
| `Page Up/Down` | Scroll by page |
| `Home/End` | Jump to top/bottom |

### Navigation
| Key | Action |
|-----|--------|
| `n` | Next issue |
| `p` | Previous issue |
| `N` | Next file |
| `P` | Previous file |
| `b` | Back in history |

### Issue Management
| Key | Action |
|-----|--------|
| `f` | Fix current issue |
| `i` | Ignore current issue |
| `e` | Show explanation |

### Actions Tab
| Key | Action |
|-----|--------|
| `1/2/3` | Apply specific fix |
| `a` | Apply automatic fix |

### Explanation Tab
| Key | Action |
|-----|--------|
| `x` | Toggle examples |
| `t` | Next related rule |
| `e` | Back to issue |

## Troubleshooting

### Performance
- For large files, use scroll controls rather than trying to view everything at once
- The syntax tree can be memory-intensive for very large files

### Display Issues
- Ensure your terminal supports colors and Unicode characters
- Minimum terminal size: 80x24 characters recommended

### Keyboard Issues
- If shortcuts don't work, ensure your terminal is capturing key events properly
- Some terminal multiplexers may interfere with certain key combinations

## Examples

### Typical Workflow

1. **Launch**: `synx validate --interactive src/`
2. **Review**: Look at the issue in context (Issues tab)
3. **Understand**: Press `e` to see explanation if needed
4. **Decide**: Press `f` to fix or `i` to ignore
5. **Continue**: Press `n` for next issue
6. **Complete**: Press `q` when done, review session summary

### Handling Complex Issues

1. **Examine Code**: Issues tab - see the problematic code
2. **Understand Structure**: Syntax Tree tab - see how code is parsed
3. **Choose Fix**: Actions tab - select appropriate fix strategy
4. **Learn**: Explanation tab - understand the rule and see examples

This interactive approach helps you not just fix issues, but understand and learn from them for better code quality in the future.
