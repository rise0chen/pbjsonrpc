//! This module contains the actual code generation logic

use std::fmt::{Display, Formatter};
use std::io::{Result, Write};

mod service;

pub use service::generate_service;

#[derive(Debug, Clone, Copy)]
struct Indent(usize);

impl Display for Indent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for _ in 0..self.0 {
            write!(f, "    ")?;
        }
        Ok(())
    }
}

fn write_jsonrpsee_start<W: Write>(
    indent: usize,
    rust_type: &str,
    writer: &mut W,
    server: bool,
    client: bool,
) -> Result<()> {
    let tag = match (server, client) {
        (true, true) => "server, client",
        (true, false) => "server",
        (false, true) => "client",
        (false, false) => "",
    };
    writeln!(
        writer,
        r#"{indent}#[jsonrpsee::proc_macros::rpc({tag})]
{indent}pub trait {rust_type} {{"#,
        indent = Indent(indent),
        rust_type = rust_type
    )
}

fn write_jsonrpsee_end<W: Write>(indent: usize, writer: &mut W) -> Result<()> {
    writeln!(writer, r#"{indent}}}"#, indent = Indent(indent),)
}
