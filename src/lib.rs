//! # pxid
//!
//! Prefixed Globally Unique Identifier
//!
//! ## Motivation
//!
//! Extend the [rs/xid][1] implementation by adding capability to have
//! a prefix and at the same time have a `u16` type support by fitting prefix bits.
//!
//! This library is inspired in Stripe IDs which have a friendly notation and are
//! very short IDs. These IDs are prefixed with a maximum of 4 bytes belonging to
//! the entity behind them.
//!
//! ## Usage
//!
//! ```ignore
//! use pxid::Pxid;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Given that some of the dependencies to build
//!     // an instance of the Pxid may fail.
//!     // - Getting current timestamp
//!     // - Getting machine id
//!     // - Getting process id
//!     //
//!     // A `Result<Pxid, Error>` is returned.
//!     let id = Pxid::new("acct".as_bytes())?;
//!
//!     println!("{}", id); //! acct_9m4e2mr0ui3e8a215n4g
//! }
//! ```
//!
//! To improve memory usage (reduce allocations), and reuse dependencies required,
//! the `Factory` struct can also be used to build `Pxid` instances.
//!
//! This is the recommended way to build `Pxid` instances, given that resources are
//! initialized once, and then reused.
//!
//! ```ignore
//! use pxid::Factory;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let factory = Factory::new_without_prefix()?;
//!     let id = factory.with_prefix("acct");
//!
//!     println!("{}", id); //! acct_9m4e2mr0ui3e8a215n4g
//!
//!     let factory_with_prefix = Factory::new("acct")?;
//!     let id = factory_with_prefix.generate();
//!
//!     println!("{}", id); //! acct_9m4e2mr0ui3e8a215n4g
//! }
//! ```
//!
//! ## Layout
//!
//! A prefixed XID fits nicely on a 16 bytes slice thanks to its packed data format.
//!
//! ```text
//! V V V V W W W W X X X Y Y Z Z Z
//! └─────┘ └─────┘ └───┘ └─┘ └───┘
//!    |       |      |    |    |
//! Prefix  Timestamp |   PID   |
//!                   |       Counter
//!                   |
//!               Machine ID
//! ```
//!
//! The prefix allows up to 4 UTF-8 Characters, this allows the ID to provide some
//! context about its scope.
//!
//! ```txt
//! acct_9m4e2mr0ui3e8a215n4g
//! ordr_9m4e2mr0ui3e8a215n4g
//! usr_9m4e2mr0ui3e8a215n4g
//! ```
//!
//! This way IDs are not only even harder to collide, but they also provides a bit
//! of context on record association.
//!
//! ## License
//!
//! This project is licensed under the MIT License
//!
//! [1]: https://github.com/rs/xid

pub const PXID: &str = "pxid";
