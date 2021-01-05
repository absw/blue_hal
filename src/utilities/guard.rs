//! RAII guard that calls a given function when constructed,
//! and another when it drops out of scope.
//!
//! Useful for ensuring resource cleanup no matter the return
//! path.
//!
//! Example
//! ```
//! # use blue_hal::hal::led::*;
//! # use blue_hal::drivers::led::*;
//! # use blue_hal::hal::doubles::gpio::*;
//! # use blue_hal::utilities::guard::*;
//! # let pin = MockPin::default();
//! # let mut led = MonochromeLed::new(pin, LogicLevel::Direct);
//! {
//!     // Led is toggled on as soon as guard is constructed, and
//!     // held protected by the guard (as it has exclusive access
//!     // to it)
//!     let _guard = Guard::new(&mut led, Toggle::on, Toggle::off);
//! }
//! // Guard has dropped out of scope here, so led is toggled off
//! assert!(!led.is_on());
//! # assert_eq!(led.pin().changes.len(), 3);
//! # assert_eq!(led.pin().changes[1], true);
//! # assert_eq!(led.pin().changes[2], false);
//! ```

use core::marker::PhantomData;

pub struct Guard<'a, T, F, G>
where
    F: FnOnce(&mut T),
    G: FnOnce(&mut T),
{
    item: &'a mut T,
    on_exit: Option<G>,
    _marker: PhantomData<F>,
}

impl<'a, T, F, G> Guard<'a, T, F, G>
where
    F: FnOnce(&mut T),
    G: FnOnce(&mut T),
{
    #[must_use]
    pub fn new(item: &'a mut T, on_entry: F, on_exit: G) -> Self {
        on_entry(item);
        Self { item, on_exit: Some(on_exit), _marker: PhantomData::default() }
    }
}

impl<'a, T, F, G> Drop for Guard<'a, T, F, G>
where
    F: FnOnce(&mut T),
    G: FnOnce(&mut T),
{
    fn drop(&mut self) { self.on_exit.take().unwrap()(self.item); }
}
