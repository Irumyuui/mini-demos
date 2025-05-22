use std::{thread, time::Duration};

use selected_text::get_selected_text;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    thread::sleep(Duration::from_secs(2));

    let text = get_selected_text()?;
    println!("Selected text: {}", text);

    Ok(())
}
