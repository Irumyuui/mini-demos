pub type BufResult<T> = (std::io::Result<usize>, T);
pub use crate::uring::rt::default_rt;
