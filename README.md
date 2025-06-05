# PRACTICE with BMC Tools

This README temporarily documents the procedure for repeating experiments from the report "Kickstarting PRACTICE with a Suite of Analysis Tools for Stochastic Vector Addition Systems" by Landon Taylor.

## Prerequisites

The following are required for correct execution:

0. Suitable operating system. PRACTICE is designed to be cross-compatible, but current features have been designed and tested on Ubuntu 24.
1. Correct installation of `z3` including `clang`:
	```
	sudo apt-get install z3 clang libclang-dev
	```
2. Correct installation of Rust:
	```
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
	```

Test your `z3` installation with `z3 --version` and your Rust installation with `rustc --version`.

## The PRACTICE Tool

Use the following commands to obtain the PRACTICE tool with BMC features at the correct commit:
```
git clone https://github.com/formal-verification-research/practice.git
cd practice
git checkout bmc
cargo run > benchmark.txt
```

Then, check `benchmark.txt` for results. Some lines may be interspersed due to the threading used to implement a timeout of 10 minutes for each process. The most interesting results are the variable bound tables.


