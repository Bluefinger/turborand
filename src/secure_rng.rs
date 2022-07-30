use crate::{
    entropy::generate_entropy, source::chacha::ChaCha8, Rc, SecureCore, SeededCore, TurboCore,
    TurboRand,
};

/// A Random Number generator, powered by the `ChaCha8` algorithm.
#[derive(PartialEq, Eq)]
#[repr(transparent)]
pub struct SecureRng(ChaCha8);

impl SecureRng {
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self(ChaCha8::with_seed(SECURE.with(|rng| rng.gen::<40>())))
    }

    #[inline]
    pub fn reseed_local(seed: [u8; 40]) {
        SECURE.with(|rng| rng.reseed(seed));
    }
}

impl TurboCore for SecureRng {
    #[inline]
    fn gen<const SIZE: usize>(&self) -> [u8; SIZE] {
        self.0.rand::<SIZE>()
    }

    #[inline]
    fn fill_bytes<B: AsMut<[u8]>>(&self, buffer: B) {
        self.0.fill(buffer);
    }
}

impl SeededCore for SecureRng {
    type Seed = [u8; 40];

    #[inline]
    #[must_use]
    fn with_seed(seed: Self::Seed) -> Self {
        Self(ChaCha8::with_seed(seed))
    }

    #[inline]
    fn reseed(&self, seed: Self::Seed) {
        self.0.reseed(seed);
    }
}

impl SecureCore for SecureRng {}
impl TurboRand for SecureRng {}

thread_local! {
    static SECURE: Rc<SecureRng> = Rc::new(SecureRng::with_seed(generate_entropy::<40>()));
}
