use std::fmt::{self, Display};
use std::ops::Deref;
use std::process;
use std::str::{from_utf8, FromStr};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rand::RngCore;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::error::{DecodeError, Error};
use crate::host_id::{machine_id, MachineIdBytes};
use crate::Result;

/// Statically creates an array of bytes which is then used to decode a
/// `String` into an Pxid instance.
const fn make_decoding_dec() -> [u8; 256] {
    let mut decoding_bytes = [0_u8; 256];

    decoding_bytes[48] = 0;
    decoding_bytes[49] = 1;
    decoding_bytes[50] = 2;
    decoding_bytes[51] = 3;
    decoding_bytes[52] = 4;
    decoding_bytes[53] = 5;
    decoding_bytes[54] = 6;
    decoding_bytes[55] = 7;
    decoding_bytes[56] = 8;
    decoding_bytes[57] = 9;
    decoding_bytes[97] = 10;
    decoding_bytes[98] = 11;
    decoding_bytes[99] = 12;
    decoding_bytes[100] = 13;
    decoding_bytes[101] = 14;
    decoding_bytes[102] = 15;
    decoding_bytes[103] = 16;
    decoding_bytes[104] = 17;
    decoding_bytes[105] = 18;
    decoding_bytes[106] = 19;
    decoding_bytes[107] = 20;
    decoding_bytes[108] = 21;
    decoding_bytes[109] = 22;
    decoding_bytes[110] = 23;
    decoding_bytes[111] = 24;
    decoding_bytes[112] = 25;
    decoding_bytes[113] = 26;
    decoding_bytes[114] = 27;
    decoding_bytes[115] = 28;
    decoding_bytes[116] = 29;
    decoding_bytes[117] = 30;
    decoding_bytes[118] = 31;
    decoding_bytes
}

/// Pxid encoding character collection
pub const ENCODING_CHARS: &[u8] = "0123456789abcdefghijklmnopqrstuv".as_bytes();

/// Pxid string encoded length
pub const ENCODED_LENGTH: usize = 25;

/// XID Encoded Length
pub const XID_ENCODED_LENGTH: usize = 20;

/// Pxid max prefix length
pub const PREFIX_LENGTH: usize = 4;

/// Pxid binary raw length
pub const BINARY_LENGTH: usize = 16;

/// Xid binary raw length
pub const XID_BINARY_LENGTH: usize = 12;

pub const DECODING_BYTES: [u8; 256] = make_decoding_dec();

/// Total parts found when splitting XID from Prefix on an encoded value
pub const ENCODED_PARTS_LENGTH: usize = 2;

/// Pxid instance Bytes
pub type Bytes = [u8; BINARY_LENGTH];

/// Pxid Instance
///
/// ## Packed Data Layout
///
/// Each Pxid instance bytes uses a "packed bytes" approach.
/// This means that bytes in a XID instance have a layout.
///
/// ```ignore
/// V V V V W W W W X X X Y Y Z Z Z
/// └─────┘ └─────┘ └───┘ └─┘ └───┘
///    |       |      |    |    |
/// Prefix  Timestamp |   PID   |
///                   |       Counter
///                   |
///               Machine ID
/// ```
///
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Pxid(pub(crate) Bytes);

impl Pxid {
    /// Retrieves the Prefix as UTF-8 Encoded characters
    #[inline]
    pub fn prefix(&self) -> Result<String> {
        Ok(from_utf8(&self.prefix_bytes())
            .map_err(|err| Error::Decode(DecodeError::InvalidUtf8(err)))?
            .to_string())
    }

    /// Retrieves the Prefix Bytes
    #[inline]
    pub fn prefix_bytes(&self) -> [u8; 4] {
        [self.0[0], self.0[1], self.0[2], self.0[3]]
    }

    /// Retrieves the Unix Timestamp used to build this Pxid
    #[inline]
    pub fn timestamp(&self) -> SystemTime {
        let secs = u64::from(u32::from_be_bytes([
            self.0[4], self.0[5], self.0[6], self.0[7],
        ]));
        UNIX_EPOCH + Duration::from_secs(secs)
    }

    /// Retrieves the Machine Pxid used to build this Pxid
    #[inline]
    pub fn machine_id(&self) -> [u8; 3] {
        [self.0[8], self.0[9], self.0[10]]
    }

    /// Retrieves the Process Pxid Bytes
    #[inline]
    pub fn process_id_bytes(&self) -> [u8; 2] {
        [self.0[11], self.0[12]]
    }

    /// Retrieves the Process Pxid used to build this Pxid
    #[inline]
    pub fn process_id(&self) -> u16 {
        u16::from_be_bytes(self.process_id_bytes())
    }

    /// Retrieves Counter Bytes
    #[inline]
    pub fn counter_bytes(&self) -> [u8; 3] {
        [self.0[13], self.0[14], self.0[15]]
    }

    /// Retrieves Counter value used to build the Pxid
    #[inline]
    pub fn counter(&self) -> u32 {
        u32::from_be_bytes([0, self.0[9], self.0[10], self.0[11]])
    }

    /// Generates a Pxid instance using the current timestamp.
    /// This is equivalent to calling `new_with_time` providing
    /// `SystemTime::now` timestamp as seconds.
    ///
    /// # Reference
    ///
    /// Follows the authors algorithm writen on Golang in the [following source][1].
    ///
    /// [1]: https://github.com/rs/xid/blob/e6fb919be3fc74f2b846a6d174e57e076a38b1c1/id.go#L142
    pub fn new(prefix: &str) -> Result<Self> {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to retrive time")
            .as_secs() as u32;

        Self::new_with_time(prefix, time)
    }

    /// Creates a new `Pxid` instance using the current timestamp.
    ///
    /// # Panics
    ///
    /// If an error ocurrs creating a Pxid instance either by retrieving a
    /// Timestamp (Clock might be in an invalid state), generating the
    /// Machine Pxid, gathering the Process Pxid (PID) or generating a random value.
    ///
    pub fn new_unchecked(prefix: &str) -> Self {
        let time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Failed to retrive time")
            .as_secs() as u32;

        Self::new_with_time(prefix, time).unwrap()
    }

    /// Generates a Pxid instance using the passed in time seconds as an instance
    /// of `u32`
    ///
    /// # Reference
    ///
    /// Follows the authors algorithm writen on Golang in the [following source][1].
    ///
    /// [1]: https://github.com/rs/xid/blob/e6fb919be3fc74f2b846a6d174e57e076a38b1c1/id.go#L147
    pub fn new_with_time(prefix: &str, time: u32) -> Result<Self> {
        let machine_id = Self::read_machine_id()?;
        let process_id = Self::read_process_id();
        let counter = Self::read_counter();
        let id = Self::from_parts(prefix, time, machine_id, process_id, counter)?;

        Ok(id)
    }

    /// Retrieve the bytes corresponding to a traditional XID instance
    ///
    /// ```ignore
    /// V V V V W W W W X X X Y Y Z Z Z
    /// └─────┘ └─────────────────────┘
    ///    |              |
    /// Prefix            |
    ///                   |
    ///                   |
    ///                  XID
    /// ```
    ///
    pub fn xid_bytes(&self) -> [u8; 12] {
        let b = self.0;

        [
            b[4], b[5], b[6], b[7], b[8], b[9], b[10], b[11], b[12], b[13], b[14], b[15],
        ]
    }

    #[inline]
    pub(crate) fn from_parts(
        prefix: &str,
        time: u32,
        machine_id: MachineIdBytes,
        process_id: u16,
        counter: u32,
    ) -> Result<Pxid> {
        if prefix.len() > 4 {
            return Err(Error::PrefixExceedsMaxLength(prefix.to_string()));
        }

        if prefix.is_empty() {
            return Err(Error::Decode(DecodeError::MissingPrefix(
                prefix.to_string(),
            )));
        }

        let prefix_len = prefix.len();
        let mut bytes: Bytes = [0; BINARY_LENGTH];
        let mut prefix_bytes: Vec<u8> = vec![0; prefix_len];

        prefix_bytes.copy_from_slice(prefix.as_bytes());

        // Copies binary representation of UTF-8 characters as part of the
        // inner slice containing the prefix
        bytes[0..=prefix_len - 1].copy_from_slice(&prefix_bytes[0..=prefix_len - 1]);

        // Copies UNIX Timestamp first 4 bytes to Pxid's first 4 bytes using
        // Big Endian order
        bytes[4..=7].copy_from_slice(&time.to_be_bytes());

        // Copies first 3 bytes from Machine Pxid
        bytes[9..=11].copy_from_slice(&machine_id);

        // Copies first 2 bytes from Process Pxid
        bytes[12..=13].copy_from_slice(&process_id.to_be_bytes());

        // 3 bytes of increment counter (big endian)
        bytes[14..].copy_from_slice(&counter.to_be_bytes()[0..=1]);

        Ok(Self(bytes))
    }

    pub fn encode_xid(xid_bytes: &[u8; 12]) -> Result<String> {
        let mut bytes: [u8; XID_BINARY_LENGTH] = [0; XID_BINARY_LENGTH];
        bytes.copy_from_slice(xid_bytes);

        let mut enc_bytes = [0_u8; XID_ENCODED_LENGTH];

        enc_bytes[19] = ENCODING_CHARS[((bytes[11] << 4) & 31) as usize];
        enc_bytes[18] = ENCODING_CHARS[((bytes[11] >> 1) & 31) as usize];
        enc_bytes[17] = ENCODING_CHARS[(((bytes[11] >> 6) | (bytes[10] << 2)) & 31) as usize];
        enc_bytes[16] = ENCODING_CHARS[(bytes[10] >> 3) as usize];
        enc_bytes[15] = ENCODING_CHARS[(bytes[9] & 31) as usize];
        enc_bytes[14] = ENCODING_CHARS[(((bytes[9] >> 5) | (bytes[8] << 3)) & 31) as usize];
        enc_bytes[13] = ENCODING_CHARS[((bytes[8] >> 2) & 31) as usize];
        enc_bytes[12] = ENCODING_CHARS[(((bytes[8] >> 7) | (bytes[7] << 1)) & 31) as usize];
        enc_bytes[11] = ENCODING_CHARS[(((bytes[7] >> 4) | (bytes[6] << 4)) & 31) as usize];
        enc_bytes[10] = ENCODING_CHARS[((bytes[6] >> 1) & 31) as usize];
        enc_bytes[9] = ENCODING_CHARS[(((bytes[6] >> 6) | (bytes[5] << 2)) & 31) as usize];
        enc_bytes[8] = ENCODING_CHARS[(bytes[5] >> 3) as usize];
        enc_bytes[7] = ENCODING_CHARS[(bytes[4] & 31) as usize];
        enc_bytes[6] = ENCODING_CHARS[(((bytes[4] >> 5) | (bytes[3] << 3)) & 31) as usize];
        enc_bytes[5] = ENCODING_CHARS[((bytes[3] >> 2) & 31) as usize];
        enc_bytes[4] = ENCODING_CHARS[(((bytes[3] >> 7) | (bytes[2] << 1)) & 31) as usize];
        enc_bytes[3] = ENCODING_CHARS[(((bytes[2] >> 4) | (bytes[1] << 4)) & 31) as usize];
        enc_bytes[2] = ENCODING_CHARS[((bytes[1] >> 1) & 31) as usize];
        enc_bytes[1] = ENCODING_CHARS[(((bytes[1] >> 6) | (bytes[0] << 2)) & 31) as usize];
        enc_bytes[0] = ENCODING_CHARS[(bytes[0] >> 3) as usize];

        Ok(String::from(
            from_utf8(&enc_bytes).expect("Invalid UTF-8 value found encoding ID"),
        ))
    }

    pub fn decode_xid(s: &str) -> Result<[u8; XID_BINARY_LENGTH]> {
        if s.len() != XID_ENCODED_LENGTH {
            return Err(Error::Decode(DecodeError::InvalidXidLength(
                s.to_string(),
                s.len(),
            )));
        }

        if let Some(c) = s.chars().find(|&c| !matches!(c, '0'..='9' | 'a'..='v')) {
            return Err(Error::Decode(DecodeError::InvalidChar(s.to_string(), c)));
        }

        let str_bytes = s.as_bytes();
        let mut bytes: [u8; XID_BINARY_LENGTH] = [0; XID_BINARY_LENGTH];

        bytes[11] = DECODING_BYTES[str_bytes[17] as usize] << 6
            | DECODING_BYTES[str_bytes[18] as usize] << 1
            | DECODING_BYTES[str_bytes[19] as usize] >> 4;
        bytes[10] = DECODING_BYTES[str_bytes[16] as usize] << 3
            | DECODING_BYTES[str_bytes[17] as usize] >> 2;
        bytes[9] =
            DECODING_BYTES[str_bytes[14] as usize] << 5 | DECODING_BYTES[str_bytes[15] as usize];
        bytes[8] = DECODING_BYTES[str_bytes[12] as usize] << 7
            | DECODING_BYTES[str_bytes[13] as usize] << 2
            | DECODING_BYTES[str_bytes[14] as usize] >> 3;
        bytes[7] = DECODING_BYTES[str_bytes[11] as usize] << 4
            | DECODING_BYTES[str_bytes[12] as usize] >> 1;
        bytes[6] = DECODING_BYTES[str_bytes[9] as usize] << 6
            | DECODING_BYTES[str_bytes[10] as usize] << 1
            | DECODING_BYTES[str_bytes[11] as usize] >> 4;
        bytes[5] =
            DECODING_BYTES[str_bytes[8] as usize] << 3 | DECODING_BYTES[str_bytes[9] as usize] >> 2;
        bytes[4] =
            DECODING_BYTES[str_bytes[6] as usize] << 5 | DECODING_BYTES[str_bytes[7] as usize];
        bytes[3] = DECODING_BYTES[str_bytes[4] as usize] << 7
            | DECODING_BYTES[str_bytes[5] as usize] << 2
            | DECODING_BYTES[str_bytes[6] as usize] >> 3;
        bytes[2] =
            DECODING_BYTES[str_bytes[3] as usize] << 4 | DECODING_BYTES[str_bytes[4] as usize] >> 1;
        bytes[1] = DECODING_BYTES[str_bytes[1] as usize] << 6
            | DECODING_BYTES[str_bytes[2] as usize] << 1
            | DECODING_BYTES[str_bytes[3] as usize] >> 4;
        bytes[0] =
            DECODING_BYTES[str_bytes[0] as usize] << 3 | DECODING_BYTES[str_bytes[1] as usize] >> 2;

        Ok(bytes)
    }

    /// Retrieves the Platform's Machine Pxid
    ///
    /// # Reference
    ///
    /// Follows the authors algorithm writen on Golang in the [following source][1].
    ///
    /// [1]: https://github.com/rs/xid/blob/e6fb919be3fc74f2b846a6d174e57e076a38b1c1/id.go#L113
    #[inline]
    fn read_machine_id() -> Result<MachineIdBytes> {
        machine_id()
    }

    /// Retrieves `process::id` as `u16` value
    #[inline]
    fn read_process_id() -> u16 {
        process::id() as u16
    }

    /// Retrieves the next value from the Atomic Counter
    ///
    /// # Reference
    ///
    /// Follows the authors algorithm writen on Golang in the [following source][1].
    ///
    /// [1]: https://github.com/rs/xid/blob/e6fb919be3fc74f2b846a6d174e57e076a38b1c1/id.go#L159
    fn read_counter() -> u32 {
        let mut rand_bytes: [u8; 3] = [0; 3];
        rand::thread_rng().fill_bytes(&mut rand_bytes);
        let seed = u32::from_be_bytes([0, rand_bytes[0], rand_bytes[1], rand_bytes[2]]);

        AtomicU32::new(seed).fetch_add(1, Ordering::SeqCst)
    }
}

impl Default for Pxid {
    fn default() -> Self {
        Self([0_u8; BINARY_LENGTH])
    }
}

impl Deref for Pxid {
    type Target = Bytes;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Pxid {
    /// Encodes the XID instance using a subset of Base32 characters where only
    /// lowercase characters are included
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = self.xid_bytes();
        let mut enc_bytes = [0_u8; XID_ENCODED_LENGTH];

        enc_bytes[19] = ENCODING_CHARS[((bytes[11] << 4) & 31) as usize];
        enc_bytes[18] = ENCODING_CHARS[((bytes[11] >> 1) & 31) as usize];
        enc_bytes[17] = ENCODING_CHARS[(((bytes[11] >> 6) | (bytes[10] << 2)) & 31) as usize];
        enc_bytes[16] = ENCODING_CHARS[(bytes[10] >> 3) as usize];
        enc_bytes[15] = ENCODING_CHARS[(bytes[9] & 31) as usize];
        enc_bytes[14] = ENCODING_CHARS[(((bytes[9] >> 5) | (bytes[8] << 3)) & 31) as usize];
        enc_bytes[13] = ENCODING_CHARS[((bytes[8] >> 2) & 31) as usize];
        enc_bytes[12] = ENCODING_CHARS[(((bytes[8] >> 7) | (bytes[7] << 1)) & 31) as usize];
        enc_bytes[11] = ENCODING_CHARS[(((bytes[7] >> 4) | (bytes[6] << 4)) & 31) as usize];
        enc_bytes[10] = ENCODING_CHARS[((bytes[6] >> 1) & 31) as usize];
        enc_bytes[9] = ENCODING_CHARS[(((bytes[6] >> 6) | (bytes[5] << 2)) & 31) as usize];
        enc_bytes[8] = ENCODING_CHARS[(bytes[5] >> 3) as usize];
        enc_bytes[7] = ENCODING_CHARS[(bytes[4] & 31) as usize];
        enc_bytes[6] = ENCODING_CHARS[(((bytes[4] >> 5) | (bytes[3] << 3)) & 31) as usize];
        enc_bytes[5] = ENCODING_CHARS[((bytes[3] >> 2) & 31) as usize];
        enc_bytes[4] = ENCODING_CHARS[(((bytes[3] >> 7) | (bytes[2] << 1)) & 31) as usize];
        enc_bytes[3] = ENCODING_CHARS[(((bytes[2] >> 4) | (bytes[1] << 4)) & 31) as usize];
        enc_bytes[2] = ENCODING_CHARS[((bytes[1] >> 1) & 31) as usize];
        enc_bytes[1] = ENCODING_CHARS[(((bytes[1] >> 6) | (bytes[0] << 2)) & 31) as usize];
        enc_bytes[0] = ENCODING_CHARS[(bytes[0] >> 3) as usize];

        write!(
            f,
            "{}_{}",
            self.prefix().unwrap(),
            from_utf8(&enc_bytes).expect("Invalid UTF-8 value found encoding Pxid")
        )
    }
}

impl FromStr for Pxid {
    type Err = crate::Error;

    fn from_str(s: &str) -> Result<Self> {
        let encoded_length = s.to_string().len();

        if encoded_length > ENCODED_LENGTH {
            return Err(Error::Decode(DecodeError::InvalidLength(
                s.to_string(),
                s.len(),
            )));
        }

        if encoded_length < (ENCODED_LENGTH - (PREFIX_LENGTH + 1)) {
            return Err(Error::Decode(DecodeError::InvalidLength(
                s.to_string(),
                s.len(),
            )));
        }

        let parts = s.split('_').collect::<Vec<&str>>();

        if parts.len() != ENCODED_PARTS_LENGTH {
            return Err(Error::Decode(DecodeError::MissingPrefix(s.to_string())));
        }
        let prefix = parts.first().unwrap().to_string();
        let xid = parts.get(1).unwrap().to_string();

        if prefix.len() > 4 {
            return Err(Error::Decode(DecodeError::InvalidPrefixLength(
                prefix.to_string(),
                prefix.len(),
            )));
        }

        if xid.len() > XID_ENCODED_LENGTH {
            return Err(Error::Decode(DecodeError::InvalidXidLength(
                xid.to_string(),
                xid.len(),
            )));
        }

        let mut id: [u8; 16] = [0; 16];
        let prefix_bytes = prefix.as_bytes();
        let xid_bytes = Self::decode_xid(&xid)?;

        // Assign Prefix UTF-8 Bytes
        id[0] = prefix_bytes[0];
        id[1] = prefix_bytes[1];
        id[2] = prefix_bytes[2];
        id[3] = prefix_bytes[3];

        // Assign Timestamp Bytes
        id[4] = xid_bytes[0];
        id[5] = xid_bytes[1];
        id[6] = xid_bytes[2];
        id[7] = xid_bytes[3];

        // Assign Machine ID Bytes
        id[8] = xid_bytes[4];
        id[9] = xid_bytes[5];
        id[10] = xid_bytes[6];

        // Assign PID Bytes
        id[11] = xid_bytes[7];
        id[12] = xid_bytes[8];

        // Assign Counter Bytes
        id[13] = xid_bytes[9];
        id[14] = xid_bytes[10];
        id[15] = xid_bytes[11];

        Ok(Self(id))
    }
}

impl From<Bytes> for Pxid {
    fn from(value: Bytes) -> Self {
        Pxid(value)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "serde")]
    use serde_test::{assert_ser_tokens, Configure, Token};

    use crate::{DecodeError, Error};

    use super::*;

    // https://github.com/rs/xid/blob/e6fb919be3fc74f2b846a6d174e57e076a38b1c1/id_test.go#L120
    #[test]
    fn encodes_an_id_as_a_string() {
        assert_eq!(
            Pxid([
                0x61, 0x63, 0x63, 0x74, 0x4d, 0x88, 0xe1, 0x5b, 0x60, 0xf4, 0x86, 0xe4, 0x28, 0x41,
                0x2d, 0xc9
            ])
            .to_string(),
            "acct_9m4e2mr0ui3e8a215n4g"
        );
    }

    // https://github.com/rs/xid/blob/e6fb919be3fc74f2b846a6d174e57e076a38b1c1/id_test.go#L127
    // https://github.com/rs/xid/blob/e6fb919be3fc74f2b846a6d174e57e076a38b1c1/id_test.go#L135
    #[test]
    fn decodes_a_string_into_an_pxid() {
        assert_eq!(
            Pxid::from_str("acct_9m4e2mr0ui3e8a215n4g").unwrap(),
            Pxid([
                0x61, 0x63, 0x63, 0x74, 0x4d, 0x88, 0xe1, 0x5b, 0x60, 0xf4, 0x86, 0xe4, 0x28, 0x41,
                0x2d, 0xc9
            ])
        );
    }

    // https://github.com/rs/xid/blob/e6fb919be3fc74f2b846a6d174e57e076a38b1c1/id_test.go#L146
    #[test]
    fn from_invalid_string_complains() {
        assert_eq!(
            Pxid::from_str("invalid"),
            Err(Error::Decode(DecodeError::InvalidLength(
                "invalid".into(),
                7
            )))
        );
    }

    // https://github.com/rs/xid/blob/e6fb919be3fc74f2b846a6d174e57e076a38b1c1/id_test.go#L305
    #[test]
    fn from_string_with_invalid_char() {
        assert_eq!(
            Pxid::from_str("acct_9m4e2mr0ui3e8a215n4x"),
            Err(Error::Decode(DecodeError::InvalidChar(
                "9m4e2mr0ui3e8a215n4x".into(),
                'x'
            )))
        );
    }

    #[test]
    fn retrieves_machine_id_from_xid_instance() {
        let id: Bytes = [
            0x61, 0x63, 0x63, 0x74, 0x4d, 0x88, 0xe1, 0x5b, 0x60, 0xf4, 0x86, 0xe4, 0x28, 0x41,
            0x2d, 0xc9,
        ];
        let xid = Pxid::from(id);

        assert_eq!(xid.machine_id(), [0x60, 0xf4, 0x86]);
    }

    #[test]
    fn retrieves_process_id_from_xid_instance() {
        let id: Bytes = [
            0x61, 0x63, 0x63, 0x74, 0x4d, 0x88, 0xe1, 0x5b, 0x60, 0xf4, 0x86, 0xe4, 0x28, 0x41,
            0x2d, 0xc9,
        ];
        let xid = Pxid::from(id);

        assert_eq!(xid.process_id_bytes(), [0xe4, 0x28]);
    }

    #[test]
    fn retrieves_counter_from_xid_instance() {
        let id: Bytes = [
            0x61, 0x63, 0x63, 0x74, 0x4d, 0x88, 0xe1, 0x5b, 0x60, 0xf4, 0x86, 0xe4, 0x28, 0x41,
            0x2d, 0xc9,
        ];
        let xid = Pxid::from(id);

        assert_eq!(xid.counter_bytes(), [0x41, 0x2d, 0xc9]);
    }

    #[test]
    fn retrives_xid_bytes() {
        let id: Bytes = [
            0x61, 0x63, 0x63, 0x74, 0x4d, 0x88, 0xe1, 0x5b, 0x60, 0xf4, 0x86, 0xe4, 0x28, 0x41,
            0x2d, 0xc9,
        ];
        let xid = Pxid::from(id);

        assert_eq!(
            xid.xid_bytes(),
            [0x4d, 0x88, 0xe1, 0x5b, 0x60, 0xf4, 0x86, 0xe4, 0x28, 0x41, 0x2d, 0xc9]
        );
    }

    #[test]
    fn encodes_a_xid_as_a_string() {
        assert_eq!(
            Pxid::encode_xid(&[
                0x4d, 0x88, 0xe1, 0x5b, 0x60, 0xf4, 0x86, 0xe4, 0x28, 0x41, 0x2d, 0xc9
            ])
            .unwrap(),
            "9m4e2mr0ui3e8a215n4g"
        );
    }

    #[test]
    fn decodes_a_xid_from_a_string() {
        assert_eq!(
            Pxid::decode_xid("9m4e2mr0ui3e8a215n4g").unwrap(),
            [0x4d, 0x88, 0xe1, 0x5b, 0x60, 0xf4, 0x86, 0xe4, 0x28, 0x41, 0x2d, 0xc9],
        );
    }

    #[test]
    fn validates_invalid_prefix() {
        assert_eq!(
            Pxid::from_str("account_9m4e2mr0ui3e8a21"),
            Err(Error::Decode(DecodeError::InvalidPrefixLength(
                String::from("account"),
                7
            ))),
        );
    }

    #[test]
    fn validates_invalid_xid() {
        assert_eq!(
            Pxid::from_str("user_9m4e2mr0ui3e8a21s5n4g"),
            Err(Error::Decode(DecodeError::InvalidLength(
                String::from("user_9m4e2mr0ui3e8a21s5n4g"),
                26
            ))),
        );
    }

    #[test]
    fn creates_pxid_with_prefix() {
        let value = Pxid::new("acct");

        assert!(value.is_ok());
    }

    #[test]
    fn creates_pxid_with_prefix_and_encodes_decodes() {
        let value = Pxid::new("acct");

        assert!(value.is_ok());
        let id = value.unwrap();
        let encoded = id.to_string();

        assert!(encoded.starts_with("acct_"));

        let decoded = Pxid::from_str(&encoded);
        assert!(decoded.is_ok());

        assert_eq!(id, decoded.unwrap());
    }

    #[test]
    fn creates_pxid_with_smaller_prefixes() {
        let value = Pxid::new("dog");

        assert!(value.is_ok());
        let id = value.unwrap();
        let encoded = id.to_string();

        assert!(encoded.starts_with("dog"));

        let decoded = Pxid::from_str(&encoded);
        assert!(decoded.is_ok());

        assert_eq!(id, decoded.unwrap());
    }

    #[test]
    fn complains_in_too_large_prefixes() {
        let value = Pxid::new("account");

        assert!(
            value.is_err(),
            "must complain because `account` has more than 4 chars"
        );
        assert_eq!(
            value.err().unwrap(),
            Error::PrefixExceedsMaxLength("account".to_string())
        );
    }

    #[test]
    #[cfg(feature = "serde")]
    fn pxid_serialization() {
        let pxid = Pxid::from_str("acct_9m4e2mr0ui3e8a215n4g").unwrap();

        assert_ser_tokens(
            &pxid.compact(),
            &[
                Token::NewtypeStruct { name: "Pxid" },
                Token::Tuple { len: 16 },
                Token::U8(97),
                Token::U8(99),
                Token::U8(99),
                Token::U8(116),
                Token::U8(77),
                Token::U8(136),
                Token::U8(225),
                Token::U8(91),
                Token::U8(96),
                Token::U8(244),
                Token::U8(134),
                Token::U8(228),
                Token::U8(40),
                Token::U8(65),
                Token::U8(45),
                Token::U8(201),
                Token::TupleEnd,
            ],
        );
    }
}
