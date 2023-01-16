use std::rc::Rc;

mod db;
mod io_utils;
mod models;
mod navigator;
mod ui;
use io_utils::*;
use navigator::Navigator;
fn main() {
    let db = Rc::new(db::JiraDatabase::new("data/db.json".to_string()));
    let mut navigator = Navigator::new(db);
    loop {
        clearscreen::clear().unwrap();
        let current_page = navigator.get_current_page();
        if let Some(page) = current_page {
            if let Err(error) = page.draw_page() {
                println!(
                    "Error rendering page: {}\nPress any key to continue...",
                    error
                );
                wait_for_key_press();
            }

            let input = get_user_input();

            if let Ok(Some(action)) = page.handle_input(input.as_str()) {
                match navigator.handle_action(action) {
                    Ok(_) => (),
                    Err(_) => break,
                }
            } else {
                println!("Error getting user input: \nPress any key to continue...");
                wait_for_key_press();
            }
        } else {
            break;
        }
    }
}
