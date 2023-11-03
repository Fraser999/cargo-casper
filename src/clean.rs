use clap::Command;

pub const SUBCOMMAND_NAME: &str = "clean";

pub fn subcommand(display_order: usize) -> Command {
    Command::new(SUBCOMMAND_NAME)
        .about(
            "Clear all stored global state from the storage dir. Cached config options are \
            unaffected.",
        )
        .display_order(display_order)
}
