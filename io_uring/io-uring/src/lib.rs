#[macro_export]
macro_rules! calc_time {
    ($info:expr, $block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let elapsed = start.elapsed();
        println!("{}: {:?}", $info, elapsed);
        result
    }};

    ($block:block) => {{
        let start = std::time::Instant::now();
        let result = $block;
        let elapsed = start.elapsed();
        println!("use time: {:?}", elapsed);
        result
    }};
}
