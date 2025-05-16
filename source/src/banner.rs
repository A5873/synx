use termcolor::{ColorChoice, ColorSpec, StandardStream, WriteColor};
use std::io::Write;

const LOGO: &str = r#"
   ███████╗██╗   ██╗███╗   ██╗██╗  ██╗         
   ██╔════╝╚██╗ ██╔╝████╗  ██║╚██╗██╔╝         
   ███████╗ ╚████╔╝ ██╔██╗ ██║ ╚███╔╝          
   ╚════██║  ╚██╔╝  ██║╚██╗██║ ██╔██╗          
   ███████║   ██║   ██║ ╚████║██╔╝ ██╗         
   ╚══════╝   ╚═╝   ╚═╝  ╚═══╝╚═╝  ╚═╝         
                                                 
   [ Universal Syntax Validator ]                
   ===============================               
      "Validate with confidence"                 
"#;

pub fn print_banner() {
    let mut stdout = StandardStream::stdout(ColorChoice::Auto);
    
    // Print the logo in teal color
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::Cyan))).unwrap();
    write!(&mut stdout, "{}", LOGO).unwrap();
    stdout.reset().unwrap();
    
    // Print version
    stdout.set_color(ColorSpec::new().set_fg(Some(termcolor::Color::White))).unwrap();
    writeln!(&mut stdout, "Version: {}", env!("CARGO_PKG_VERSION")).unwrap();
    stdout.reset().unwrap();
}
