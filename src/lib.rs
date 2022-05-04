#![doc = include_str!("../README.md")]

#[macro_use]
extern crate combine;
pub mod core;
pub mod expr;
pub mod iter;
pub mod parser;

/// main object it can be convert from &str.
///
/// node expression must be comman or newline separated.
/// Currently it does not support any operations like (or, and, xor, substract)
/// Type of Error and Iterator could be change in the future.
///
/// # Examples
///
/// ```
/// use nodeagg::Nodeagg;
///
/// let hostnames : Vec<String> =
/// Nodeagg::try_from("node[01-02],node03").unwrap().iter().collect();
/// assert_eq!(hostnames, vec!["node01", "node02", "node03"]);
/// ```
///
/// Comemnt line must start with `#`.
///
/// ```
/// # use nodeagg::Nodeagg;
/// let hostnames : Vec<String> =
/// Nodeagg::try_from("node[01-02]
/// #node03
/// node04").unwrap().iter().collect();
/// assert_eq!(hostnames, vec!["node01", "node02", "node04"]);
/// ```
///
pub type Nodeagg = core::Nodeagg;
