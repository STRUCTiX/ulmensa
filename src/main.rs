mod cli;

use time::{Duration, OffsetDateTime};
use ulmensa_lib::{Format, Mealplan};

#[tokio::main]
async fn main() {
    let args = cli::get_args();
    let client = reqwest::Client::new();

    let days_offset = Duration::days(args.days as i64);
    let date = OffsetDateTime::now_utc()
        .saturating_add(days_offset)
        .date()
        .to_string();
    let form = [
        ("func", "make_spl"),
        ("locId", "1"),
        ("date", &date),
        ("lang", "de"),
        ("startThisWeek", &date),
        ("startNextWeek", &date),
    ];
    let resp = client
        .post("https://sw-ulm-spl51.maxmanager.xyz/inc/ajax-php_konnektor.inc.php")
        .form(&form)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let menu = Mealplan::parse_menu(&resp);
    let format = if args.json {
        Format::Json
    } else if args.nutritional_info {
        Format::TableNutrition
    } else {
        Format::Table
    };
    println!("{}", menu.display(format));
}
