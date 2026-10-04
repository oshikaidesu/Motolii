#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "macos")]
pub fn run() -> Result<(), String> {
    macos::run()
}

#[cfg(target_os = "macos")]
pub use macos::motolii_a1_start;
