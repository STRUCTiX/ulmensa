mod cli;
mod lib;

use time::OffsetDateTime;

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();

    let date = OffsetDateTime::now_utc().date().to_string();
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

    let menu = lib::parse_menu(&resp);
    println!("{}", lib::display_menu_table(&menu));
}
