use md5::compute;
use rand::{distributions::Alphanumeric, Rng};

use crate::Result;

/// Machine ID first 3 bytes
pub type MachineIdBytes = [u8; 3];

/// Retrieves a Machine ID using system based approach
pub fn machine_id() -> Result<MachineIdBytes> {
    let mut bytes: MachineIdBytes = [0_u8; 3];
    let host_id = host_id()?;

    bytes.copy_from_slice(&compute(host_id)[0..3]);

    Ok(bytes)
}

// https://github.com/rs/xid/blob/e6fb919be3fc74f2b846a6d174e57e076a38b1c1/id.go#L124
// #[cfg(not(target_os = "macos"))]
pub fn host_id() -> Result<String> {
    Ok(rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(3)
        .map(char::from)
        .collect::<String>())
}

// #[cfg(any(target_os = "macos"))]
// pub fn host_id() -> Result<String> {
//     #[cfg(any(target_os = "macos"))]
//     use sysctl::Sysctl;

//     let machine_id: String = sysctl::Ctl::new("kern.uuid")
//         .map_err(|err| Error::MachineID(err.to_string()))?
//         .value()
//         .map(|v| v.to_string())
//         .map_err(|err| Error::MachineID(err.to_string()))?;

//     Ok(machine_id)
// }
