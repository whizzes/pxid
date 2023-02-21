use std::process;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use rand::RngCore;

use crate::host_id::{machine_id, MachineIdBytes};
use crate::id::Pxid;
use crate::Result;

/// Factory of XID instances. Initializes dependencies once to avoid
/// reallocating them on each ID generation.
///
/// # Design Pattern
///
/// You can read more on the _Factory_ design pattern here on [Refactoring.guru][1]
///
/// [1]: https://refactoring.guru/design-patterns/factory-comparison
pub struct Factory {
    counter: AtomicU32,
    process_id: u16,
    machine_id: MachineIdBytes,
}

impl Factory {
    pub fn new() -> Result<Self> {
        let process_id = process::id() as u16;
        let machine_id = machine_id()?;
        let counter = AtomicU32::new(Self::new_counter_seed());

        Ok(Self {
            counter,
            process_id,
            machine_id,
        })
    }

    pub(crate) fn new_counter_seed() -> u32 {
        let mut rand_bytes: [u8; 3] = [0; 3];

        rand::thread_rng().fill_bytes(&mut rand_bytes);
        u32::from_be_bytes([0, rand_bytes[0], rand_bytes[1], rand_bytes[2]])
    }

    pub(crate) fn current_timestamp() -> u32 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to retrive time")
            .as_secs() as u32
    }

    /// Creates a new ID using the current timestamp
    #[inline]
    pub fn new_id(&self, prefix: &str) -> Result<Pxid> {
        let current_timestamp = Self::current_timestamp();

        self.new_with_time(prefix, current_timestamp)
    }

    /// Creates a new ID with the provided `time`
    pub fn new_with_time(&self, prefix: &str, time: u32) -> Result<Pxid> {
        let counter: u32 = self.counter.fetch_add(1, Ordering::SeqCst);

        Pxid::from_parts(prefix, time, self.machine_id, self.process_id, counter)
    }
}
