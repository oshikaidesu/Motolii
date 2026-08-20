fn main() -> iced::Result {
    iced::application(
        motolii_shell::Shell::new,
        motolii_shell::Shell::update,
        motolii_shell::Shell::view,
    )
    .title(motolii_shell::Shell::title)
    .subscription(motolii_shell::Shell::subscription)
    .run()
}
