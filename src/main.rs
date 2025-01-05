use std::{borrow::Cow, fmt::Display};

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
}

pub struct Mealplan {
    meals: Vec<Meal>,
}

impl Mealplan {
    pub fn from(input: &str) -> Option<Self> {
        let dom = tl::parse(input, tl::ParserOptions::default()).unwrap();
        let parser = dom.parser();

        let mut all_meals = Vec::new();
        for el in dom.get_elements_by_class_name("fltl") {
            let node = el.get(parser).unwrap();
            let n = node.inner_text(parser);
            let meals = n
                .split("&nbsp")
                .filter(|&x| !x.is_empty() && x != ";")
                .filter_map(Meal::from)
                .collect::<Vec<Meal>>();
            all_meals.extend(meals);
        }

        if all_meals.is_empty() {
            return None;
        }

        Some(Mealplan { meals: all_meals })
    }
}

pub struct Meal {
    name: String,
    co2: i32,
    nutritional_value: NutritionalValue,
}

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

        let name_co2 = split[0].split_once("CO2").unwrap();
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
                f
            })
            .filter_map(|f| f.parse::<f64>().ok())
            .collect::<Vec<f64>>();

        let carbs = re_gram
            .captures_iter(split[4])
            .map(|c| {
                let (_, [f]) = c.extract();
                f
            })
            .filter_map(|f| f.parse::<f64>().ok())
            .collect::<Vec<f64>>();

        let (_, [salt]) = re_gram.captures(split[5])?.extract();
        let salt = salt.parse::<f64>().unwrap();

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
