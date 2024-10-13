pub use clap;

/// TODO
pub trait Application {
    type Error: std::error::Error;
    type Args: clap::Parser;

    fn main(&self, args: Self::Args) -> Result<(), Self::Error>;
}

pub use application_macro::main;
