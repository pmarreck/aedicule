use std::io::{self, Write as _};

fn main() -> io::Result<()> {
    io::stdout().write_all(aedicule::abi_reference_html().as_bytes())
}
