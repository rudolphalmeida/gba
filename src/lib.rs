#![allow(dead_code)]
#![allow(unused_variables)]

pub mod cpu;
pub mod gamepak;
pub mod gba;
pub mod system_bus;

#[macro_export]
macro_rules! test_mask {
    ($value:expr, $mask:expr) => {
        $value & $mask == $mask
    };
}

#[macro_export]
macro_rules! test_bit {
    ($value:expr, $idx:expr) => {
        $value & (1 << $idx) == (1 << $idx)
    };
}

#[macro_export]
macro_rules! extract_mask {
    ($value:expr, $mask:expr) => {
        ($value & $mask) >> ($mask.trailing_zeros())
    };
}

#[cfg(test)]
mod tests {}
