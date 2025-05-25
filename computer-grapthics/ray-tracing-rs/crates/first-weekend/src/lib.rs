pub mod ray;
pub mod vec3;

// fn main() {
//     let width = 256;
//     let height = 256;

//     println!("P3");
//     println!("{} {}", width, height);
//     println!("255");

//     let pb = ProgressBar::new(height as u64);
//     pb.set_style(
//         ProgressStyle::default_bar()
//             .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len}")
//             .unwrap()
//             .progress_chars("#>-"),
//     );

//     for j in 0..height {
//         for i in 0..width {
//             let r = i as f32 / (width - 1) as f32;
//             let g = j as f32 / (height - 1) as f32;
//             let b = 0.;

//             let c = Color::new(r, g, b);
//             println!("{}", Wrapper::new(&c));
//         }
//         pb.inc(1);
//     }

//     pb.finish_with_message("Image done");
// }

// / y-axis go up, x-axis go right, negative z-axis pointing in the viewing direction.
// /
// / Yeah, this is commonly referred to as right-handed coordinate.
// #[allow(unused)]
// fn camera() {
//     let aspect_radio = 16.0 / 9.0;
//     let image_width = 400;

//     let image_height = match (image_width as f32 / aspect_radio) as i32 {
//         n if n < 1 => 1,
//         n => n,
//     };

//     let viewport_height = 2.0;
//     let viewport_width = viewport_height * (image_width as f32 / image_height as f32);
// }
