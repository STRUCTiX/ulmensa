mod lib;

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();
    let form = [
        ("func", "make_spl"),
        ("locId", "1"),
        ("date", "2025-01-14"),
        ("lang", "de"),
        ("startThisWeek", "2025-01-14"),
        ("startNextWeek", "2025-01-14"),
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

    //let plan = Mealplan::from(&resp);

    //println!("{}", plan.unwrap());

    let menu = lib::parse_menu(&resp);
    lib::display_menu(&menu);
}
