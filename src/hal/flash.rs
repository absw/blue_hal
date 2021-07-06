use crate::utilities::memory::Address;
use core::{
    mem::{size_of, MaybeUninit},
    slice,
};

/// Reads and writes a range of bytes, generic over an address
pub trait ReadWrite {
    type Error: Clone + Copy + Copy;
    type Address: Address;

    /// An identifying name for the flash chip.
    fn label() -> &'static str;
    fn read(&mut self, address: Self::Address, bytes: &mut [u8]) -> nb::Result<(), Self::Error>;
    fn write(&mut self, address: Self::Address, bytes: &[u8]) -> nb::Result<(), Self::Error>;
    fn range(&self) -> (Self::Address, Self::Address);
    fn erase(&mut self) -> nb::Result<(), Self::Error>;
    fn bytes(&mut self, address: Self::Address) -> ReadIterator<Self> {
        ReadIterator {
            reader: self,
            address,
            errored: false,
            buffer: [0u8; ITERATOR_BUFFER_SIZE],
            buffer_head: ITERATOR_BUFFER_SIZE,
        }
    }

    fn write_from_blocks<I: Iterator<Item = [u8; N]>, const N: usize>(
        &mut self,
        address: Self::Address,
        blocks: I,
    ) -> Result<(), Self::Error>;
}

/// Serialize an object to flash.
pub trait UnportableSerialize: ReadWrite {
    /// # Safety
    ///
    /// This is a very raw serialization (the bytes are written as-is). Should be
    /// only used with repr(C) types with no internal references. It *will break* if any change
    /// to the struct to serialize is made between serialization and deserialization, and it
    /// *will* cause undefined behaviour. Make sure to erase the flash whenever there is an
    /// update to the serializable types.
    unsafe fn serialize<T: Sized>(
        &mut self,
        item: &T,
        address: Self::Address,
    ) -> nb::Result<(), Self::Error> {
        // Get a view into the raw bytes conforming T
        let bytes = slice::from_raw_parts((item as *const T) as *const u8, size_of::<T>());
        self.write(address, bytes)
    }
}
impl<F: ReadWrite> UnportableSerialize for F {}

/// Deserialize an object from flash.
pub trait UnportableDeserialize: ReadWrite {
    /// # Safety
    ///
    /// This is a very raw serialization (the bytes are written as-is). Should be
    /// only used in repr(C) types. It *will break* if any change to the struct to serialize is
    /// made between serialization and deserialization, and it *will* cause undefined
    /// behaviour. Make sure to erase the flash whenever there is an update to the serializable
    /// types.
    unsafe fn deserialize<T: Sized>(
        &mut self,
        address: Self::Address,
    ) -> nb::Result<T, Self::Error> {
        // Create uninitialized T with a zero repr
        let mut uninit: MaybeUninit<T> = MaybeUninit::uninit();
        let bytes = slice::from_raw_parts_mut(uninit.as_mut_ptr() as *mut _, size_of::<T>());

        // Read its byte representation into it
        self.read(address, bytes)?;
        Ok(uninit.assume_init())
    }
}
impl<F: ReadWrite> UnportableDeserialize for F {}

const ITERATOR_BUFFER_SIZE: usize = 2048;

pub struct ReadIterator<'a, R: ReadWrite + ?Sized> {
    reader: &'a mut R,
    errored: bool,
    buffer: [u8; ITERATOR_BUFFER_SIZE],
    address: R::Address,
    buffer_head: usize,
}

impl<'a, R: ReadWrite + ?Sized> Iterator for ReadIterator<'a, R> {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        if self.errored {
            return None;
        }

        if self.buffer_head == ITERATOR_BUFFER_SIZE {
            if nb::block!(self.reader.read(self.address, &mut self.buffer)).is_err() {
                self.errored = true;
                return None;
            };
            self.buffer_head = 0;
            self.address = self.address + ITERATOR_BUFFER_SIZE;
        }

        let byte = self.buffer[self.buffer_head];
        self.buffer_head += 1;
        Some(byte)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hal::doubles::flash::*;

    #[test]
    fn iterating_over_fake_flash() {
        let mut expected_bytes = Vec::new();
        for i in 0..10000 {
            expected_bytes.push(i as u8);
        }
        let mut flash = FakeFlash::new(Address(0));
        flash.write(Address(0), expected_bytes.as_slice()).unwrap();
        let bytes: Vec<u8> = flash.bytes(Address(0)).take(10000).collect();
        assert_eq!(expected_bytes, bytes);
    }
}
