# Simple Loop (Sloop) Solver

## Description

Simple Loop is a logic puzzle that involves creating a single continuous non-intersecting loop that that enters (and exits) every unshaded cell in the grid.

This project is a hastily written Simple Loop (aka sloop) solver that is currently hardcoded to only solve 6x6 puzzles. The general solver part should be relatively simple to expand to any grid size (other than the grid reprentation being a 64-bit integer), but this was the work of a few hours and it did the job I wanted it to do. It is designed to find all essentially unique ways to clue a 6x6 Simple Loop grid to achieve a unique solution. A "clue" being a shaded cell.

## Installation

This project is built using Rust and Cargo. Follow these steps to get it up and running:

```bash
# Clone the repository
git clone https://github.com/dclamage/simple-loop-solver.git

# Navigate to the directory
cd simple-loop-solver

# Build the project
cargo build --release

# Run the project
cargo run --release
```

## License

This project is dual-licensed under the terms of both the MIT License and the Apache License 2.0. You may choose to use the project under the terms of either license.
