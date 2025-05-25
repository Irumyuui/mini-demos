use first_weekend::vec3::{Color, Wrapper};
use indicatif::{ProgressBar, ProgressStyle};

fn main() {
    let width = 256;
    let height = 256;

    println!("P3");
    println!("{} {}", width, height);
    println!("255");

    let pb = ProgressBar::new(height as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len}")
            .unwrap()
            .progress_chars("#>-"),
    );

    for j in 0..height {
        for i in 0..width {
            let r = i as f32 / (width - 1) as f32;
            let g = j as f32 / (height - 1) as f32;
            let b = 0.;

            let c = Color::new(r, g, b);
            println!("{}", Wrapper::new(&c));
        }
        pb.inc(1);
    }

    pb.finish_with_message("Image done");
}
