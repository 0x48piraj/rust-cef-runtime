//! Shared memory helper for binary IPC.

use shared_memory::{Shmem, ShmemConf};

pub const SHM_THRESHOLD: usize = 64 * 1024;

pub struct SharedBuffer {
    shmem: Shmem,
    size: usize,
}

impl SharedBuffer {

    /// Create a new shared memory region.
    pub fn create(size: usize) -> Self {
        let shmem = ShmemConf::new()
            .size(size)
            .create()
            .expect("failed to create shared memory");

        Self { shmem, size }
    }

    /// Open an existing shared memory region.
    pub fn open(name: &str, size: usize) -> Self {
        let shmem = ShmemConf::new()
            .os_id(name)
            .open()
            .expect("failed to open shared memory");

        Self { shmem, size }
    }

    /// OS identifier used for cross-process sharing.
    pub fn name(&self) -> String {
        self.shmem.get_os_id().to_string()
    }

    /// Immutable view of the memory.
    pub fn as_slice(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.shmem.as_ptr(),
                self.size,
            )
        }
    }

    /// Mutable view of the memory.
    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.shmem.as_ptr(),
                self.size,
            )
        }
    }

    /// Copy data into the shared memory.
    pub fn write(&mut self, data: &[u8]) {
        self.as_slice_mut()[..data.len()].copy_from_slice(data);
    }
}
