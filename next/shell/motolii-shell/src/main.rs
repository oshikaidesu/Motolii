//! wraps: iced — 窓を開けるだけ。判断は `motolii_shell` にあり、ここには置かない。
//!
//! ここが薄いのは運転席のため — 窓を開けずに `Shell` を直接動かせる形にしてある。

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
