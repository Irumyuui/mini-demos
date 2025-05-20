#![allow(unused)]

pub mod uring;
pub mod utils;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

// #[global_allocator]
// static GLOBAL: jemalloc::Jemalloc = jemalloc::Jemalloc;
