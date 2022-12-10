//!

pub trait Point<const D: usize>: Copy + std::fmt::Debug {
    fn get_axis(&self, d: usize) -> f32;

    #[inline(always)]
    fn distance_squared(self, b: Self) -> f32 {
        (0..D)
            .into_iter()
            .map(|d| {
                let delta = self.get_axis(d) - b.get_axis(d);
                delta * delta
            })
            .sum::<f32>() as f32
    }
}

pub mod point_implementations;
pub use point_implementations::*;

pub mod kdtree;
pub use kdtree::*;

pub mod utils;
