use crate::types::*;
use rand::{rngs::StdRng, Rng, SeedableRng};

pub struct Dice {
    rng: StdRng
}
impl Dice {
    pub fn new(seed: u64) -> Self { Self { rng: StdRng::seed_from_u64(seed) } }
    pub fn roll_2d6(&mut self) -> u8 {
        let a: u8 = self.rng.gen_range(1..=6);
        let b: u8 = self.rng.gen_range(1..=6);
        a + b
    }
}