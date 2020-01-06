//! Common base for coffer binaries

#[allow(unused_imports)]
use log::{debug, error, info, trace, warn};

pub mod certificate;
pub mod coffer;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
