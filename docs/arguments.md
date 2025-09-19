# Stamina Toolset Features & Commands

This document describes the basic features and commands for the Stamina Toolset.

## Running the Tool

All commands are run using the `staminats <commands>` command. If you are building/running from source, use `cargo run -- <commands>` instead.

## Quick Top-Level Reference

| Tool | Description | Command |
| --- | --- | --- |
| Benchmark | Runs a benchmark set | `benchmark` |
| BMC | Outputs a BMC-unrolled SMT encoding of the model | `bmc` |
| Bounder | Uses BMC to generate variable bounds | `bounds` |
| Cycle & Commute | Expands an input trace set | `cycle-commute` |
| Dependency Graph | Outputs a dependency graph | `dependency-graph` |
| **Ragtimer** | The Ragtimer partial state space tool | `ragtimer` |
| **Stamina** | The Stamina partial state space tool | `stamina` |
| **Wayfarer** | The Wayfarer partial state space tool | `wayfarer` |


## In-Depth Tool Notes

### Benchmark

*This mode is under development. Currently, it benchmarks only Ragtimer and its dependencies.*

**Command**: `staminats benchmark <options>`

**Options**:

| Command | Description |
| --- | --- |
| `--model <>` or `-m <>`   | Set the input model (required) |
| `--dir <>` or `-d <>`     | Set a directory with multiple models (alternative to `--model`) |
| `--num-traces <>`         | Set the number of traces for Ragtimer to generate (default 10K) |
| `--cycle-length <>`       | Set the maximum Cycle & Commute cycle length (default 3) |
| `--commute-depth <>`      | Set the maximum Cycle & Commute recursion depth (default 3) |
| `--timeout <>` or `-t <>` | Set the time limit per-model in seconds (default 10 minutes) |

The `--dir` command allows subfolders to be used. It will attempt to parse and benchmark *every* file with extension `.crn` or `.vas`.

At the moment, this command runs the following sequence for each model, timing every step.

1. Parse the model
2. Build a dependency graph
3. Use Ragtimer's RL trace generation to build a partial state space
4. Output the partial state space just from Ragtimer
5. Use Cycle & Commute to expand this partial state space
6. Output the partial state space from the full approach
7. Generate a shell script to run Prism and/or Storm on both partial state spaces

Results are stored in the `benchmark_results/<timestamp>` directory.

### BMC

**Command**: `staminats bmc <options>`

**Options**:

| Command | Description |
| --- | --- |
| `--model <>` or `-m <>`   | Set the input model (required) |
| `--steps <>`              | Set the number of unrolling steps for BMC (required) |
| `--output <>`             | Set the output directory. (default `<model>.smt2`) |
| `--timeout <>` or `-t <>` | Set the time limit per-model in seconds (default 10 minutes) |


### Bounder

**Command**: `staminats bounds <options>`

**Options**:

| Command | Description |
| --- | --- |
| `--model <>` or `-m <>`   | Set the input model (required) |
| `--bits <>` or `-b <>`    | Set the number of bits to use for BMC (default 16) |
| `--max-steps <>`          | Set the limit on the number of steps (default 1K) |
| `--trim`                  | Use the trimmed model based on dependency graph (false if absent) |
| `--timeout <>` or `-t <>` | Set the time limit per-model in seconds (default 10 minutes) |

This command will run BMC with bit vectors of the specified number of bits for each variable. It will unroll the model to the required number of steps to reach a satisfying/target state, then it uses a binary search to determine tightest and loosest upper and lower bounds for each variable along traces that reach a target state.

### Cycle & Commute

**Command**: `staminats cycle-commute <options>`

**Options**:

| Command | Description |
| --- | --- |
| `--model <>` or `-m <>`   | Set the input model (required) |
| ``-trace <>`              | Provide a tab-separated list of transitions (required) |
| `--cycle-length <>`       | Set the maximum Cycle & Commute cycle length (default 3) |
| `--commute-depth <>`      | Set the maximum Cycle & Commute recursion depth (default 3) |
| `--output <>` or `-o <>`  | Set the output file name *without extensions* (default `output`) |
| `--timeout <>` or `-t <>` | Set the time limit per-model in seconds (default 10 minutes) |

This command will build an explicit state space from the input trace(s), then use Cycle & Commute with specified depth and cycle length to expand the state space. It outputs an explicit transition system `<output>.tra,sta,lab` that can be fed to Prism as follows:

```
prism -importmodel <output>.tra,sta,lab <output>.prop -ctmc
```

### Dependency Graph

**Command**: `staminats dependency-graph <options>`

**Options**:

| Command | Description |
| --- | --- |
| `--model <>` or `-m <>`   | Set the input model (required) |
| `--output <>` or `-o <>`  | Set the output file name *without extensions* (default `output`) |
| `--timeout <>` or `-t <>` | Set the time limit per-model in seconds (default 10 minutes) |

This command will build a dependency graph from the specified model. It outputs the graph in plain text to the command line, as well as to `<output>.txt`.

### Ragtimer

**Command**: `staminats ragtimer <options>`

**Options**:

| Command | Description |
| --- | --- |
| `--model <>` or `-m <>`    | Set the input model (required) |
| `--approach <>` or `-a <>` | Set the mode for trace generation (default RL) |
| `--num-traces <>`          | Set the number of traces for Ragtimer to generate (default 10K) |
| `--cycle-length <>`        | Set the maximum Cycle & Commute cycle length (default 3) |
| `--commute-depth <>`       | Set the maximum Cycle & Commute recursion depth (default 3) |
| `--timeout <>` or `-t <>`  | Set the time limit per-model in seconds (default 10 minutes) |

The `--approach` value may be one of the following (more coming soon):
- `RL` uses reinforcement learning to attempt to generate the most effective traces
- `shortest` generates a random collection of the shortest possible traces
- `random` generates purely-random traces (this may never terminate)

This command will build an explicit state space from the input model, then use Cycle & Commute with specified depth and cycle length to expand the state space. It outputs an explicit transition system `<output>.tra,sta,lab` that can be fed to Prism as follows:


```
prism -importmodel <output>.tra,sta,lab <output>.prop -ctmc
```

### Wayfarer

*Coming soon*

### Stamina 

*Coming less soon*

