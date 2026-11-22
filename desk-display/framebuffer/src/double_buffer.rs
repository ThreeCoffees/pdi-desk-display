use embassy_sync::{
    blocking_mutex::raw::{CriticalSectionRawMutex, RawMutex},
    mutex::Mutex,
    rwlock::{RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use crate::FrameBuffer;

struct State {
    front_index: usize,
    version: u32,
}

/// Contains 2 buffers:
/// FRONT — currently displayed buffer (read only)
/// BACK — not displayed, used to prepare the next front
///
/// buffers are swapped on commit
pub struct DoubleBuffer<M: RawMutex = CriticalSectionRawMutex> {
    buffers: [RwLock<M, FrameBuffer>; 2],
    state: Mutex<M, State>,
}

impl<M: RawMutex> DoubleBuffer<M> {
    pub async fn read_front(&self) -> RwLockReadGuard<'_, M, FrameBuffer> {
        let state = self.state.lock().await;
        let guard = self.buffers[state.front_index].read().await;

        drop(state);

        guard
    }

    pub async fn read_back(&self) -> RwLockReadGuard<'_, M, FrameBuffer> {
        let state = self.state.lock().await;
        let back_index = 1 - state.front_index;

        let guard = self.buffers[back_index].read().await;

        drop(state);

        guard
    }

    pub async fn write_back(&self) -> RwLockWriteGuard<'_, M, FrameBuffer> {
        let state = self.state.lock().await;
        let back_index = 1 - state.front_index;

        let guard = self.buffers[back_index].write().await;

        drop(state);

        guard
    }

    /// acquire both buffers:
    /// FRONT at 0, BACK at 1
    pub async fn read_both(
        &self,
    ) -> (
        RwLockReadGuard<'_, M, FrameBuffer>,
        RwLockReadGuard<'_, M, FrameBuffer>,
    ) {
        let state = self.state.lock().await;

        let front_index = state.front_index;
        let back_index = 1 - front_index;

        let (first_index, second_index) = if front_index < back_index {
            (front_index, back_index)
        } else {
            (back_index, front_index)
        };

        let first = self.buffers[first_index].read().await;
        let second = self.buffers[second_index].read().await;

        drop(state);

        if front_index == first_index {
            (first, second)
        } else {
            (second, first)
        }
    }

    pub async fn commit(&self) -> u32 {
        let mut state = self.state.lock().await;

        let old_front = state.front_index;

        // Prevent new readers from acquiring the old FRONT.
        let old_front_guard = self.buffers[old_front].write().await;

        state.front_index = 1 - old_front;
        state.version = state.version.wrapping_add(1);

        let version = state.version;

        drop(old_front_guard);
        drop(state);

        version
    }

    pub async fn version(&self) -> u32 {
        self.state.lock().await.version
    }
}

impl DoubleBuffer<CriticalSectionRawMutex> {
    pub const fn new(initial_color: u8) -> Self {
        Self {
            buffers: [
                RwLock::new(FrameBuffer::new(initial_color)),
                RwLock::new(FrameBuffer::new(initial_color)),
            ],
            state: Mutex::new(State {
                front_index: 0,
                version: 0,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_graphics_core::pixelcolor::BinaryColor;
    use futures::executor::block_on;

    #[test]
    fn both_buffers_are_initialized() {
        block_on(async {
            let buffers = DoubleBuffer::new(0xFF);

            let front = buffers.read_front().await;
            assert!(front.as_bytes().iter().all(|&b| b == 0xFF));
            drop(front);

            let back = buffers.read_back().await;
            assert!(back.as_bytes().iter().all(|&b| b == 0xFF));
        });
    }

    #[test]
    fn back_is_independent_from_front() {
        block_on(async {
            let buffers = DoubleBuffer::new(0xFF);

            {
                let mut back = buffers.write_back().await;
                back.clear(0x00);
            }

            let front = buffers.read_front().await;
            assert!(front.as_bytes().iter().all(|&b| b == 0xFF));
            drop(front);

            let back = buffers.read_back().await;
            assert!(back.as_bytes().iter().all(|&b| b == 0x00));
        });
    }

    #[test]
    fn commit_swaps_buffers() {
        block_on(async {
            let buffers = DoubleBuffer::new(0xFF);

            {
                let mut back = buffers.write_back().await;
                back.clear(0x00);
            }

            assert_eq!(buffers.commit().await, 1);

            let front = buffers.read_front().await;
            assert!(front.as_bytes().iter().all(|&b| b == 0x00));
        });
    }

    #[test]
    fn old_front_becomes_back() {
        block_on(async {
            let buffers = DoubleBuffer::new(0xFF);

            {
                let mut back = buffers.write_back().await;
                back.clear(0x00);
            }

            buffers.commit().await;

            let back = buffers.read_back().await;
            assert!(back.as_bytes().iter().all(|&b| b == 0xFF));
        });
    }

    #[test]
    fn version_increments() {
        block_on(async {
            let buffers = DoubleBuffer::new(0xFF);

            assert_eq!(buffers.version().await, 0);

            buffers.commit().await;
            assert_eq!(buffers.version().await, 1);

            buffers.commit().await;
            assert_eq!(buffers.version().await, 2);
        });
    }

    #[test]
    fn read_both_returns_front_and_back_in_correct_order() {
        block_on(async {
            let buffers = DoubleBuffer::new(0xFF);

            {
                let mut back = buffers.write_back().await;
                back.set_pixel(10, 20, BinaryColor::On);
            }

            let (front, back) = buffers.read_both().await;

            assert_eq!(
                front.get_pixel(10, 20),
                Some(BinaryColor::Off),
                "FRONT should still contain the initial white frame"
            );
            assert_eq!(
                back.get_pixel(10, 20),
                Some(BinaryColor::On),
                "BACK should contain the modified frame"
            );
        });
    }

    #[test]
    fn read_both_keeps_both_buffers_locked_for_reading() {
        block_on(async {
            let buffers = DoubleBuffer::new(0xFF);

            let (front, back) = buffers.read_both().await;

            // Both guards can be used simultaneously.
            let _ = front.as_bytes();
            let _ = back.as_bytes();

            // Release both locks.
            drop(front);
            drop(back);

            // Both buffers should be writable again.
            let mut back = buffers.write_back().await;
            back.set_pixel(10, 20, BinaryColor::On);
        });
    }

    #[test]
    fn read_both_tracks_front_and_back_after_commit() {
        block_on(async {
            let buffers = DoubleBuffer::new(0xFF);

            {
                let mut back = buffers.write_back().await;
                back.set_pixel(10, 20, BinaryColor::On);
            }

            buffers.commit().await;

            let (front, back) = buffers.read_both().await;

            assert_eq!(front.get_pixel(10, 20), Some(BinaryColor::On));
            assert_eq!(back.get_pixel(10, 20), Some(BinaryColor::Off));
        });
    }
}
