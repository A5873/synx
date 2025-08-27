use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};
use std::io::Write;

/// Print a lightweight banner for CLI usage (only when appropriate)
pub fn print_banner() {
    // Only show banner in interactive/long-running operations
    // Most CLI operations should be silent by default
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    
    // Print a minimal, single-line banner
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Cyan))).unwrap();
    write!(&mut stdout, "✨ Synx").unwrap();
    stdout.reset().unwrap();
    
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::White))).unwrap();
    write!(&mut stdout, " v{}", env!("CARGO_PKG_VERSION")).unwrap();
    stdout.reset().unwrap();
    
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Green))).unwrap();
    writeln!(&mut stdout, " - Universal Syntax Validator").unwrap();
    stdout.reset().unwrap();
}

/// Print just the name and version (ultra-minimal)
pub fn print_minimal_banner() {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Cyan))).unwrap();
    write!(&mut stdout, "synx v{}", env!("CARGO_PKG_VERSION")).unwrap();
    stdout.reset().unwrap();
}
