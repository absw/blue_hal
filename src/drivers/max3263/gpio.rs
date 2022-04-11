use core::marker::PhantomData;

pub struct AF0 {}
pub struct AF1 {}
pub struct AF2 {}
pub struct AF3 {}
pub struct AF4 {}


pub struct P10<MODE> {
    _phantom: PhantomData<MODE>,
}

pub struct P11<MODE> {
    _phantom: PhantomData<MODE>,
}

pub struct P12<MODE> {
    _phantom: PhantomData<MODE>,
}

pub struct P13<MODE> {
    _phantom: PhantomData<MODE>,
}

pub struct P14<MODE> {
    _phantom: PhantomData<MODE>,
}

pub struct P15<MODE> {
    _phantom: PhantomData<MODE>,
}
