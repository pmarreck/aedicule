use std::io::{self, Write as _};

fn main() -> io::Result<()> {
    io::stdout().write_all(aedicule::guide_for_llms_markdown().as_bytes())
}
