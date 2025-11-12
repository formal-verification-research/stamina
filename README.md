# The Stamina Toolset

The Stamina Toolset is a *work-in-progress* tool for the analysis of highly-complex continuous-time models, especially Chemical Reaction Networks and other Stochastic Vector Addition Systems.

## Quick Start Guide

To build and run from source on a Debian-based machine, run the following commands.

```sh
# Get prerequisites
sudo apt install libz3-dev libclang-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build and run
cargo run -- <commands>
```

This software is tested on Debian-based Linux distributions but is designed to be cross-platform compatible. Make sure you have installed equivalent prerequites on your own machine, then you should be able to build from source.

Command-line arguments can be found with `--help`, and they are more thoroughly documented at [docs/arguments.md](https://github.com/formal-verification-research/stamina/blob/main/docs/arguments.md)

## Implemented Functionality

This checklist shows the status of our tool implementations:

- **BMC Variable Bounding**: Complete
- **Ragtimer 1.0**: In progress, released fully by December 31, 2025
- **Wayfarer 1.0**: In progress
- **Stamina Rust**: In progress

## Input Format

The input format is documented at [docs/input.md](https://github.com/formal-verification-research/stamina/blob/main/docs/input.md)

Additional information about our implementation is found in an intermediate report at [docs/Kickstarting_Practice.pdf](https://github.com/formal-verification-research/stamina/blob/main/docs/Kickstarting_Practice.pdf)
