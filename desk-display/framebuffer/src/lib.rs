#![no_std]
#![forbid(unsafe_code)]

pub mod buffer;
pub mod double_buffer;

pub use buffer::{
    FrameBuffer,
    BYTES_PER_ROW,
    FRAME_BYTES,
    HEIGHT,
    WIDTH,
};

pub use double_buffer::DoubleBuffer;
