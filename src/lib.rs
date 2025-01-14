use scraper::{Element, Html, Selector};

#[derive(Debug)]
struct NutritionalInfo {
    energy: String,
    protein: String,
    fat: String,
    carbohydrates: String,
    salt: String,
}

#[derive(Debug)]
struct Prices {
    student: f32,
    employee: f32,
    guest: f32,
}

#[derive(Debug)]
struct Dish {
    name: String,
    dietary_info: Vec<String>,
    prices: Prices,
    nutrition: NutritionalInfo,
}

#[derive(Debug)]
pub struct Section {
    name: String,
    dishes: Vec<Dish>,
}

fn extract_prices(price_text: &str) -> Prices {
    let price_parts: Vec<f32> = price_text
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

fn extract_nutritional_info(element: scraper::ElementRef) -> NutritionalInfo {
    let table_selector = Selector::parse("table").unwrap();
    let row_selector = Selector::parse("tr").unwrap();
    let cell_selector = Selector::parse("td").unwrap();

    let table = element.select(&table_selector).next().unwrap();
    let mut nutrition_data = std::collections::HashMap::new();

    for row in table.select(&row_selector).skip(1) {
        // Skip header
        let cells: Vec<String> = row
            .select(&cell_selector)
            .map(|cell| cell.text().collect::<String>().trim().to_string())
            .collect();

        if cells.len() >= 2 {
            nutrition_data.insert(cells[0].clone(), cells[1].clone());
        }
    }

    NutritionalInfo {
        energy: nutrition_data
            .get("Energie")
            .unwrap_or(&String::new())
            .clone(),
        protein: nutrition_data
            .get("Protein")
            .unwrap_or(&String::new())
            .clone(),
        fat: nutrition_data.get("Fett").unwrap_or(&String::new()).clone(),
        carbohydrates: nutrition_data
            .get("Kohlenhydrate")
            .unwrap_or(&String::new())
            .clone(),
        salt: nutrition_data.get("Salz").unwrap_or(&String::new()).clone(),
    }
}

pub fn parse_menu(html_content: &str) -> Vec<Section> {
    let document = Html::parse_document(html_content);
    let section_selector = Selector::parse("div.gruppenkopf").unwrap();
    let dish_selector = Selector::parse("div.splMeal").unwrap();
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

                    let dietary_info: Vec<String> = elem_ref
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

                    //let nutrition = extract_nutritional_info(elem_ref);
                    let n = NutritionalInfo {
                        energy: "a".to_string(),
                        protein: "a".to_string(),
                        fat: "a".to_string(),
                        carbohydrates: "a".to_string(),
                        salt: "a".to_string(),
                    };

                    if let Some(section) = &mut current_section {
                        section.dishes.push(Dish {
                            name: dish_text,
                            dietary_info,
                            prices,
                            nutrition: n,
                        });
                    }
                }
            }
            next_sibling = node.next_sibling();
        }
    }

    if let Some(section) = current_section {
        menu.push(section);
    }

    menu
}

pub fn display_menu(menu: &[Section]) {
    for section in menu {
        println!("\n{}:", section.name);
        println!("{}", "-".repeat(section.name.len()));

        for dish in &section.dishes {
            let name = dish.name.split_once("\n").unwrap().0;
            println!("\n• {}", name);

            if !dish.dietary_info.is_empty() {
                println!("  Dietary Info: {}", dish.dietary_info.join(", "));
            }

            println!(
                "  Prices: {:.2}€ (Student) | {:.2}€ (Employee) | {:.2}€ (Guest)",
                dish.prices.student, dish.prices.employee, dish.prices.guest
            );

            println!("  Nutritional Information:");
            println!("    - Energy: {}", dish.nutrition.energy);
            println!("    - Protein: {}", dish.nutrition.protein);
            println!("    - Fat: {}", dish.nutrition.fat);
            println!("    - Carbohydrates: {}", dish.nutrition.carbohydrates);
            println!("    - Salt: {}", dish.nutrition.salt);
        }
    }
}
