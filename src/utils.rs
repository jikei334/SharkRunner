use bevy::prelude::*;

use js_sys;


pub struct Rng(u64);

impl Rng {
    pub fn new(seed: &Seed) -> Self {
        Rng(seed.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407) | 1)
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    pub fn range(&mut self, n: usize) -> usize {
        (self.next_u64() % n as u64) as usize
    }
}

#[derive(Resource)]
pub struct Seed(u64);

impl Seed {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub fn new_with_time() -> Self {
        Self({
            #[cfg(target_arch = "wasm32")]
            {
                js_sys::Date::now() as u64
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(1)
            }
        })
    }
}
