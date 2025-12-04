use std::fmt::{self, Write};

pub mod function;

pub trait DotOutput {
    fn write_dot<W: Write>(&self, writer: &mut W) -> fmt::Result;

    fn to_dot(&self) -> String {
        let mut s = String::new();
        self.write_dot(&mut s).unwrap();
        s
    }
}
