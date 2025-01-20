use std::collections::HashSet;

use once_cell::sync::Lazy;
use prettytable::{row, Cell, Row, Table};
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime};

// Compile Regex one time only.
static RE_CO2: Lazy<Regex> = Lazy::new(|| Regex::new(r"abdruck pro Portion ([0-9\.]+)").unwrap());
static RE_ENERGY: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"([0-9,]+) kJ \/ ([0-9,]+) kcal").unwrap());
static RE_GRAM: Lazy<Regex> = Lazy::new(|| Regex::new(r"([0-9,]+) g").unwrap());

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize)]
struct NutritionalInfo {
    energy_kj: f64,
    energy_kcal: f64,
    protein: f64,
    fat: Vec<f64>,
    carbohydrates: Vec<f64>,
    salt: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Prices {
    student: f64,
    employee: f64,
    guest: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct Dish {
    name: String,
    co2: i32,
    dietary_info: HashSet<String>,
    prices: Prices,
    nutrition: NutritionalInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Section {
    name: String,
    dishes: Vec<Dish>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mealplan {
    menu: Vec<Section>,
}

pub enum Format {
    Table,
    TableNutrition,
    Json,
}

impl Mealplan {
    pub async fn from(day_offset: u8) -> Self {
        let days_offset = Duration::days(day_offset as i64);
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

        let client = reqwest::Client::new();
        let resp = client
            .post("https://sw-ulm-spl51.maxmanager.xyz/inc/ajax-php_konnektor.inc.php")
            .form(&form)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();

        parse_menu(&resp)
    }

    pub fn display(&self, format: Format) -> String {
        match format {
            Format::Table => display_menu_table(&self.menu),
            Format::TableNutrition => display_menu_table_nutrition(&self.menu),
            Format::Json => serde_json::to_string_pretty(&self.menu).unwrap(),
        }
    }
}

fn display_menu_table(menu: &[Section]) -> String {
    let mut table = Table::new();
    table.add_row(row!["Kategorie", "Gericht", "CO2", "Info", "kcal", "Preis"]);
    for section in menu {
        for dish in &section.dishes {
            table.add_row(Row::new(vec![
                Cell::new(&section.name),
                Cell::new(&dish.name),
                Cell::new(&format!("{}g", dish.co2)),
                Cell::new(
                    &dish
                        .dietary_info
                        .iter()
                        .fold(String::new(), |acc, el| acc + el + ", "),
                ),
                Cell::new(&format!("{:.2}", dish.nutrition.energy_kcal)),
                Cell::new(&format!(
                    "{:.2}€|{:.2}€|{:.2}€",
                    dish.prices.student, dish.prices.employee, dish.prices.guest
                )),
            ]));
        }
    }

    table.to_string()
}

fn display_menu_table_nutrition(menu: &[Section]) -> String {
    let mut table = Table::new();
    table.add_row(row![
        "Kategorie",
        "Gericht",
        "CO2",
        "Info",
        "kcal",
        "Kohlenhydrate (Zucker)",
        "Protein",
        "Fett (ges.)",
        "Salz",
        "Preis"
    ]);
    for section in menu {
        for dish in &section.dishes {
            table.add_row(Row::new(vec![
                Cell::new(&section.name),
                Cell::new(&dish.name),
                Cell::new(&format!("{}g", dish.co2)),
                Cell::new(
                    &dish
                        .dietary_info
                        .iter()
                        .fold(String::new(), |acc, el| acc + el + ", "),
                ),
                Cell::new(&format!("{:.2}", dish.nutrition.energy_kcal)),
                Cell::new(&format!(
                    "{:.2}g ({:.2}g)",
                    dish.nutrition.carbohydrates[0], dish.nutrition.carbohydrates[1]
                )),
                Cell::new(&format!("{:.2}g", dish.nutrition.protein)),
                Cell::new(&format!(
                    "{:.2}g ({:.2}g)",
                    dish.nutrition.fat[0], dish.nutrition.fat[1]
                )),
                Cell::new(&format!("{:.2}g", dish.nutrition.salt)),
                Cell::new(&format!(
                    "{:.2}€|{:.2}€|{:.2}€",
                    dish.prices.student, dish.prices.employee, dish.prices.guest
                )),
            ]));
        }
    }

    table.to_string()
}

fn extract_prices(price_text: &str) -> Prices {
    let price_parts: Vec<f64> = price_text
        .split('|')
        .filter_map(|part| {
            part.trim()
                .trim_start_matches('€')
                .trim()
                .replace(',', ".")
                .parse()
                .ok()
        })
        .collect();

    Prices {
        student: price_parts[0],
        employee: price_parts[1],
        guest: price_parts[2],
    }
}

fn extract_name_co2_nutritional_info(input: &str) -> Option<(String, i32, NutritionalInfo)> {
    //let re_co2 = Regex::new(r"abdruck pro Portion ([0-9\.]+)").unwrap();
    //let re_energy = Regex::new(r"([0-9,]+) kJ \/ ([0-9,]+) kcal").unwrap();
    //let re_gram = Regex::new(r"([0-9,]+) g").unwrap();
    let split = input
        .split("\n")
        .map(|x| x.trim())
        .filter(|&x| !x.is_empty() && !x.starts_with("Nährwertangaben"))
        .collect::<Vec<&str>>();

    let name_co2 = split[0].split_once("CO2")?;
    let name = name_co2.0;
    let co2 = RE_CO2
        .captures(name_co2.1)?
        .get(1)?
        .as_str()
        .replace(".", "")
        .parse::<i32>()
        .ok()?;

    let (_, [kj, kcal]) = RE_ENERGY.captures(split[1])?.extract();
    let kj = kj.replace(",", ".").parse::<f64>().ok()?;
    let kcal = kcal.replace(",", ".").parse::<f64>().ok()?;

    let protein = RE_GRAM
        .captures(split[2])?
        .get(1)?
        .as_str()
        .replace(",", ".")
        .parse::<f64>()
        .ok()?;

    let fat = RE_GRAM
        .captures_iter(split[3])
        .map(|c| {
            let (_, [f]) = c.extract();
            f.replace(",", ".")
        })
        .filter_map(|f| f.parse::<f64>().ok())
        .collect::<Vec<f64>>();

    let carbohydrates = RE_GRAM
        .captures_iter(split[4])
        .map(|c| {
            let (_, [f]) = c.extract();
            f.replace(",", ".")
        })
        .filter_map(|f| f.parse::<f64>().ok())
        .collect::<Vec<f64>>();

    let (_, [salt]) = RE_GRAM.captures(split[5])?.extract();
    let salt = salt.replace(",", ".").parse::<f64>().ok()?;

    Some((
        name.to_string(),
        co2,
        NutritionalInfo {
            energy_kj: kj,
            energy_kcal: kcal,
            protein,
            fat,
            carbohydrates,
            salt,
        },
    ))
}

pub fn parse_menu(html_content: &str) -> Mealplan {
    let document = Html::parse_document(html_content);
    let section_selector = Selector::parse("div.gruppenkopf").unwrap();
    let name_selector = Selector::parse("div.gruppenname").unwrap();
    let dish_text_selector = Selector::parse("div[style*='width:92%']").unwrap();
    let icon_selector = Selector::parse("img").unwrap();
    let price_selector = Selector::parse("span").unwrap();

    let mut menu = Vec::new();
    let mut current_section: Option<Section> = None;

    for element in document.select(&section_selector) {
        // If we have a previous section, push it to the menu
        if let Some(section) = current_section {
            menu.push(section);
        }

        // Create new section
        let section_name = element
            .select(&name_selector)
            .next()
            .unwrap()
            .text()
            .collect::<String>();

        current_section = Some(Section {
            name: section_name,
            dishes: Vec::new(),
        });

        // Find all dishes until next section
        let mut next_sibling = element.next_sibling();
        while let Some(node) = next_sibling {
            if let Some(elem) = node.value().as_element() {
                if elem.has_class(
                    "gruppenkopf",
                    scraper::CaseSensitivity::AsciiCaseInsensitive,
                ) {
                    break;
                }

                if elem.has_class("splMeal", scraper::CaseSensitivity::AsciiCaseInsensitive) {
                    let elem_ref = scraper::ElementRef::wrap(node).unwrap();

                    let dish_text = elem_ref
                        .select(&dish_text_selector)
                        .next()
                        .unwrap()
                        .text()
                        .collect::<String>();

                    let dietary_info: HashSet<String> = elem_ref
                        .select(&icon_selector)
                        .filter_map(|icon| icon.value().attr("title"))
                        .map(String::from)
                        .collect();

                    let prices = elem_ref
                        .select(&price_selector)
                        .find(|span| span.text().collect::<String>().contains('€'))
                        .map(|span| extract_prices(&span.text().collect::<String>()))
                        .unwrap_or(Prices {
                            student: 0.0,
                            employee: 0.0,
                            guest: 0.0,
                        });

                    if let Some((dish, co2, nutrition)) =
                        extract_name_co2_nutritional_info(&dish_text)
                    {
                        if let Some(section) = &mut current_section {
                            section.dishes.push(Dish {
                                name: dish,
                                co2,
                                dietary_info,
                                prices,
                                nutrition,
                            });
                        }
                    }
                }
            }
            next_sibling = node.next_sibling();
        }
    }

    if let Some(section) = current_section {
        menu.push(section);
    }

    Mealplan { menu }
}
