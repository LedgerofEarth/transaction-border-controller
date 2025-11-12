//! TBC Gateway - TGP Implementation

pub mod router;
pub mod agent;

pub use router::Router;
pub use agent::Agent;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}