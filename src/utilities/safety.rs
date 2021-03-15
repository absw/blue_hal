use core::ops::Deref;

pub struct LimitedU8<const MIN: u8, const MAX: u8>(u8);

impl<const MIN: u8, const MAX: u8> Deref for LimitedU8<MIN, MAX> {
    type Target = u8;

    fn deref(&self) -> &Self::Target { &self.0 }
}

impl<const MIN: u8, const MAX: u8> LimitedU8<MIN, MAX> {
    pub fn new(value: u8) -> Option<Self> {
        (value >= MIN && value <= MAX).then_some(LimitedU8(value))
    }
}
