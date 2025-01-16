use clap::Parser;

/// ulmensa is a command line utility that retrieves and displays the current meal plans offered at the
/// University of Ulm canteen. Supports multiple output formats and can show nutritional information.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Display additional information about nutritional values.
    #[arg(short, long, default_value_t = false)]
    pub nutritional_info: bool,

    /// Specify the number of days ahead to display.
    #[arg(short, long, default_value_t = 0)]
    pub days: u8,

    /// Output the meal plan in JSON format.
    #[arg(short, long, default_value_t = false)]
    pub json: bool,
}

pub fn get_args() -> Args {
    Args::parse()
}
