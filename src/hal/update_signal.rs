pub trait UpdateSignal {
    // TODO: Return Option<Bank> for specifying which specific bank to boot from.
    fn should_update(&self) -> bool;
}
