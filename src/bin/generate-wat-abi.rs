use std::io::{self, Write as _};

fn main() -> io::Result<()> {
    io::stdout().write_all(aedicule::wat_abi_markdown().as_bytes())
}
