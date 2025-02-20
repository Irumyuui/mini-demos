mod arena;

pub mod error;

pub mod prelude {
    pub use crate::arena::BlockArena;
    pub use crate::error::*;
}
