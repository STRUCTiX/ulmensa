mod cli;

use time::{Duration, OffsetDateTime};
use ulmensa_lib::{Format, Mealplan};

#[tokio::main]
async fn main() {
    let args = cli::get_args();

    let menu = Mealplan::from(args.days).await;
    let format = if args.json {
        Format::Json
    } else if args.nutritional_info {
        Format::TableNutrition
    } else {
        Format::Table
    };
    println!("{}", menu.display(format));
}
