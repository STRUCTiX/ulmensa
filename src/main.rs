use std::{borrow::Cow, collections::BTreeSet, fmt::Display};

use prettytable::{row, Cell, Row, Table};
use regex::Regex;

#[tokio::main]
async fn main() {
    let client = reqwest::Client::new();
    let form = [
        ("func", "make_spl"),
        ("locId", "1"),
        ("date", "2025-01-07"),
        ("lang", "de"),
        ("startThisWeek", "2025-01-07"),
        ("startNextWeek", "2025-01-07"),
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

    let plan = Mealplan::from(&resp);

    println!("{}", plan.unwrap());
}

pub struct Mealplan {
    meals: BTreeSet<Meal>,
    filter: Filter,
}

impl Mealplan {
    pub fn from(input: &str) -> Option<Self> {
        let dom = tl::parse(input, tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();

        let mut all_meals = BTreeSet::new();
        for el in dom.get_elements_by_class_name("fltl") {
            let node = el.get(parser).unwrap();
            let n = node.inner_text(parser);
            let mut meals = n
                .split("&nbsp")
                .filter(|&x| !x.is_empty() && x != ";")
                .filter_map(Meal::from)
                .collect::<BTreeSet<Meal>>();
            all_meals.append(&mut meals);
        }

        if all_meals.is_empty() {
            return None;
        }

        Some(Mealplan {
            meals: all_meals,
            filter: Filter::default(),
        })
    }

    pub fn add_filter(&mut self, filter: Filter) {
        self.filter = filter;
    }
}

impl Display for Mealplan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut table = Table::new();
        table.add_row(row!["Beschreibung", "CO2"]);

        for d in &self.meals {
            table.add_row(Row::new(vec![
                Cell::new(&d.name),
                Cell::new(&format!("{}", d.co2)).style_spec("r"),
            ]));
        }

        write!(f, "{}", table)
    }
}

#[derive(PartialEq)]
pub struct Meal {
    name: String,
    co2: i32,
    nutritional_value: NutritionalValue,
}

impl Ord for Meal {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

impl PartialOrd for Meal {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Meal {}

impl Meal {
    pub fn from(input: &str) -> Option<Self> {
        let re_co2 = Regex::new(r"abdruck pro Portion ([0-9\.]+)").unwrap();
        let re_energy = Regex::new(r"([0-9,]+) kJ \/ ([0-9,]+) kcal").unwrap();
        let re_gram = Regex::new(r"([0-9,]+) g").unwrap();
        let split = input
            .split("\n")
            .map(|x| x.trim())
            .filter(|&x| !x.is_empty() && !x.starts_with("Nährwertangaben"))
            .collect::<Vec<&str>>();

        let name_co2 = split[0].split_once("CO2")?;
        let name = name_co2.0;
        let co2 = re_co2
            .captures(name_co2.1)?
            .get(1)?
            .as_str()
            .replace(".", "")
            .parse::<i32>()
            .ok()?;

        let (_, [kj, kcal]) = re_energy.captures(split[1])?.extract();
        let kj = kj.replace(",", ".").parse::<f64>().ok()?;
        let kcal = kcal.replace(",", ".").parse::<f64>().ok()?;

        let protein = re_gram
            .captures(split[2])?
            .get(1)?
            .as_str()
            .replace(",", ".")
            .parse::<f64>()
            .ok()?;

        let fats = re_gram
            .captures_iter(split[3])
            .map(|c| {
                let (_, [f]) = c.extract();
                f.replace(",", ".")
            })
            .filter_map(|f| f.parse::<f64>().ok())
            .collect::<Vec<f64>>();

        let carbs = re_gram
            .captures_iter(split[4])
            .map(|c| {
                let (_, [f]) = c.extract();
                f.replace(",", ".")
            })
            .filter_map(|f| f.parse::<f64>().ok())
            .collect::<Vec<f64>>();

        let (_, [salt]) = re_gram.captures(split[5])?.extract();
        let salt = salt.replace(",", ".").parse::<f64>().ok()?;

        Some(Meal {
            name: name.to_string(),
            co2,
            nutritional_value: NutritionalValue {
                energy_kj: kj,
                energy_kcal: kcal,
                protein,
                fat: fats[0],
                fat_saturated: fats[1],
                carb: carbs[0],
                sugar: carbs[1],
                salt,
            },
        })
    }
}

#[derive(PartialEq, PartialOrd)]
pub struct NutritionalValue {
    energy_kj: f64,
    energy_kcal: f64,
    protein: f64,
    fat: f64,
    fat_saturated: f64,
    carb: f64,
    sugar: f64,
    salt: f64,
}

impl Display for NutritionalValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Energy: {} kJ/ {} kcal, Protein: {}, Fat: {} (saturated: {}), Carb: {} (sugar: {}), Salt: {}",
            self.energy_kj, self.energy_kcal, self.protein, self.fat, self.fat_saturated, self.carb, self.sugar, self.salt
        )
    }
}

pub struct Filter {
    vegetarian_only: bool,
    pretty_print: bool,
    oneline: bool,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            vegetarian_only: false,
            pretty_print: true,
            oneline: false,
        }
    }
}
