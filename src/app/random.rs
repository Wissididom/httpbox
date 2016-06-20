extern crate rand;

use self::rand::{Rng, XorShiftRng};


fn to_bytes(val: u32) -> [u32; 4] {
    [val, val, val, val]
}

pub struct RandomGenerator {
    rng: XorShiftRng,
}

impl RandomGenerator {
    pub fn new(seed: Option<u32>) -> Self {
        let seed = seed.unwrap_or_else(|| rand::thread_rng().gen::<u32>());
        RandomGenerator { rng: rand::SeedableRng::from_seed(to_bytes(seed)) }
    }
}

impl Rng for RandomGenerator {
    fn next_u32(&mut self) -> u32 {
        self.rng.next_u32()
    }
}

#[cfg(test)]
mod tests {

    mod rng {
        use super::super::*;
        use super::super::rand::Rng;

        #[test]
        fn rng_no_seed() {

            RandomGenerator::new(None);
        }

        #[test]
        fn rng_seed_consistent() {

            let mut rng = RandomGenerator::new(Some(1234));
            assert_eq!(rng.next_u32(), 2537108u32);
            assert_eq!(rng.next_u32(), 1238u32);
            assert_eq!(rng.next_u32(), 2537104u32);
            assert_eq!(rng.next_u32(), 1234u32);
        }
    }
}
