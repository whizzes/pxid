use md5::compute;

use crate::{Error, Result};

/// Machine ID first 3 bytes
pub type MachineIdBytes = [u8; 3];

/// Retrieves a Machine ID using system based approach
pub fn machine_id() -> Result<MachineIdBytes> {
    let mut bytes: MachineIdBytes = [0_u8; 3];
    let host_id = host_id()?;

    if host_id.is_empty() {
        unimplemented!("Fallback Approach not Implemented");
    }

    bytes.copy_from_slice(&compute(host_id)[0..3]);

    Ok(bytes)
}

#[cfg(any(target_os = "macos"))]
pub fn host_id() -> Result<String> {
    #[cfg(any(target_os = "macos"))]
    use sysctl::Sysctl;

    let machine_id: String = sysctl::Ctl::new("kern.uuid")
        .map_err(|err| Error::MachineID(err.to_string()))?
        .value()
        .map(|v| v.to_string())
        .map_err(|err| Error::MachineID(err.to_string()))?;

    Ok(machine_id)
}
