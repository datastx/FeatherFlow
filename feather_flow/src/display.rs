use colored::*;

/// Returns the FeatherFlow ASCII art logo
pub fn get_logo() -> String {
    let logo = r#"
  ███████╗███████╗ █████╗ ████████╗██╗  ██╗███████╗██████╗ 
  ██╔════╝██╔════╝██╔══██╗╚══██╔══╝██║  ██║██╔════╝██╔══██╗
  █████╗  █████╗  ███████║   ██║   ███████║█████╗  ██████╔╝
  ██╔══╝  ██╔══╝  ██╔══██║   ██║   ██╔══██║██╔══╝  ██╔══██╗
  ██║     ███████╗██║  ██║   ██║   ██║  ██║███████╗██║  ██║
  ╚═╝     ╚══════╝╚═╝  ╚═╝   ╚═╝   ╚═╝  ╚═╝╚══════╝╚═╝  ╚═╝
  ███████╗██╗      ██████╗ ██╗    ██╗
  ██╔════╝██║     ██╔═══██╗██║    ██║
  █████╗  ██║     ██║   ██║██║ █╗ ██║
  ██╔══╝  ██║     ██║   ██║██║███╗██║
  ██║     ███████╗╚██████╔╝╚███╔███╔╝
  ╚═╝     ╚══════╝ ╚═════╝  ╚══╝╚══╝ 
    "#;

    logo.to_string()
}

/// Returns a colored version of the logo
pub fn get_colored_logo() -> ColoredString {
    get_logo().bright_cyan()
}

/// Returns the compact version of the logo
pub fn get_compact_logo() -> String {
    let logo = r#"
  ____ ____ ____ _____ _  _ ____ ____    ____ _    ____ _ _ _ 
  |___ |___ |__|   |   |__| |___ |__/    |___ |    |  | | | | 
  |    |___ |  |   |   |  | |___ |  \    |    |___ |__| |_|_| 
    "#;

    logo.to_string()
}

/// Returns a colored version of the compact logo
pub fn get_compact_colored_logo() -> ColoredString {
    get_compact_logo().bright_cyan()
}

/// Display version information with the ASCII art logo
pub fn display_version() {
    println!("{}", get_colored_logo());
    println!("FeatherFlow CLI version {}", env!("CARGO_PKG_VERSION"));
    println!("A Rust-based SQL transformation tool");
    println!("Repository: {}", env!("CARGO_PKG_REPOSITORY"));
}

/// Display a welcome message for the parse command
pub fn display_parse_welcome() {
    println!("{}", get_compact_colored_logo());
}
