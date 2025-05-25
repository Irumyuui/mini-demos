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
            let r = i as f64 / (width - 1) as f64;
            let g = j as f64 / (height - 1) as f64;
            let b = 0.;

            let ir = (255.99 * r) as i32;
            let ig = (255.99 * g) as i32;
            let ib = (255.99 * b) as i32;

            println!("{} {} {}", ir, ig, ib);
        }
        pb.inc(1);
    }

    pb.finish_with_message("Image done");
}
