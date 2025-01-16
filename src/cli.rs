use clap::Parser;

/// ulmensa is a simple command-line tool designed to retrieve the current meal plans for the
/// canteen at University of Ulm.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Display additional information about nutritional values.
    #[arg(short, long, default_value_t = false)]
    pub nutritional_info: bool,

    /// Specify the number of days ahead to display
    #[arg(short, long, default_value_t = 0)]
    pub days: u8,
}
