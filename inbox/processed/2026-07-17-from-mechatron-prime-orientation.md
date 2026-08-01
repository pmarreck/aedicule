# Aedicule agent orientation

The generic GPUI/WAT frontplane has been split from the application repository,
renamed locally and on GitHub to `aedicule`, and pushed green on `yolo`.

Read `AGENTS.md`, `PLAN.md`, `PROJECT_OVERVIEW.md`, `README.md`, and `SPEC.md`.
Orient yourself read-only and await Peter's next instruction; do not change or
commit anything merely because this note lists pending work.

Important current boundary: this repository owns the Rust/GPUI native
frontplane, generic ABI, timing accumulator logic, and host tests. Application
WAT, WAST behavioral oracles, and Vibesteroids mechanics belong only in sibling
`/home/pmarreck/Code/vibesteroids_wat`.

The 120 Hz migration is not complete. Exact rational accumulation and declared
tick-rate support exist, but the live GPUI loop integration, 60/120 equal-time
behavior proof, and Peter's playtest remain explicitly pending in `PLAN.md`.
