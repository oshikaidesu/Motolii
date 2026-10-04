fn main() {
    #[cfg(target_os = "macos")]
    {
        if let Err(error) = motolii_glue::run() {
            eprintln!("a0: {error}");
            std::process::exit(1);
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        eprintln!("a0 is macOS only");
        std::process::exit(1);
    }
}
