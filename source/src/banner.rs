use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};
use std::io::Write;

/// Print a beautiful ASCII banner for CLI usage (only when appropriate)
pub fn print_banner() {
    // Only show banner in interactive/long-running operations
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    
    // Enhanced ASCII banner with better typography
    println!();
    
    // Top border (49 chars total)
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Cyan)).set_bold(true)).unwrap();
    writeln!(&mut stdout, "╭─────────────────────────────────────────────────╮").unwrap();
    stdout.reset().unwrap();
    
    // Title line: │  { ✓ } SYNX v0.3.0                         │
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Cyan)).set_bold(true)).unwrap();
    write!(&mut stdout, "│  ").unwrap();
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Blue)).set_bold(true)).unwrap();
    write!(&mut stdout, "{{ ").unwrap();
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Magenta)).set_bold(true)).unwrap();
    write!(&mut stdout, "✓ ").unwrap();
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Blue)).set_bold(true)).unwrap();
    write!(&mut stdout, "}} ").unwrap();
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::White)).set_bold(true)).unwrap();
    write!(&mut stdout, "SYNX ").unwrap();
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Yellow)).set_bold(false)).unwrap();
    write!(&mut stdout, "v{}", env!("CARGO_PKG_VERSION")).unwrap();
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Cyan)).set_bold(true)).unwrap();
    writeln!(&mut stdout, "                         │").unwrap();
    stdout.reset().unwrap();
    
    // Subtitle line: │  Universal Syntax Validator & Linter         │
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Cyan)).set_bold(true)).unwrap();
    write!(&mut stdout, "│  ").unwrap();
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Green)).set_bold(false)).unwrap();
    write!(&mut stdout, "Universal Syntax Validator & Linter").unwrap();
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Cyan)).set_bold(true)).unwrap();
    writeln!(&mut stdout, "         │").unwrap();
    stdout.reset().unwrap();
    
    // Features line: │  Parse • Validate • Lint • Format           │
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Cyan)).set_bold(true)).unwrap();
    write!(&mut stdout, "│  ").unwrap();
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Blue)).set_bold(false)).unwrap();
    write!(&mut stdout, "Parse").unwrap();
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Cyan))).unwrap();
    write!(&mut stdout, " • ").unwrap();
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Green)).set_bold(false)).unwrap();
    write!(&mut stdout, "Validate").unwrap();
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Cyan))).unwrap();
    write!(&mut stdout, " • ").unwrap();
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Yellow)).set_bold(false)).unwrap();
    write!(&mut stdout, "Lint").unwrap();
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Cyan))).unwrap();
    write!(&mut stdout, " • ").unwrap();
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Magenta)).set_bold(false)).unwrap();
    write!(&mut stdout, "Format").unwrap();
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Cyan)).set_bold(true)).unwrap();
    writeln!(&mut stdout, "           │").unwrap();
    stdout.reset().unwrap();
    
    // Bottom border (49 chars total)
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Cyan)).set_bold(true)).unwrap();
    writeln!(&mut stdout, "╰─────────────────────────────────────────────────╯").unwrap();
    stdout.reset().unwrap();
    
    println!();
}

/// Print just the name and version (ultra-minimal)
pub fn print_minimal_banner() {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Cyan))).unwrap();
    write!(&mut stdout, "synx v{}", env!("CARGO_PKG_VERSION")).unwrap();
    stdout.reset().unwrap();
}
