#!/bin/bash

cargo build -r

# MODEL_FILES=("ModifiedYeastPolarization")
# NUMS_TRACES=(10 100)
# COMMUTE_DEPTHS=(0 1)
# CYCLE_LENGTHS=(0)
# APPROACHES=("RL" "shortest")

MODEL_FILES=("ModifiedYeastPolarization" "EnzymaticFutileCycle" "ReversibleIsomerization" "SimplifiedMotilityRegulation" "SingleSpeciesProductionDegradation")
NUMS_TRACES=(10 100 1000 10000 100000 1000000)
COMMUTE_DEPTHS=(0 1 2 3 4)
CYCLE_LENGTHS=(0 2 3 4 5 6)
APPROACHES=("RL" "shortest")

OUTPUT_FOLDER="$(date +%Y%m%d-%H%M%S)"

for MODEL in "${MODEL_FILES[@]}"; do
  for NUM_TRACES in "${NUMS_TRACES[@]}"; do
    for COMMUTE_DEPTH in "${COMMUTE_DEPTHS[@]}"; do
      for CYCLE_LENGTH in "${CYCLE_LENGTHS[@]}"; do
        for APPROACH in "${APPROACHES[@]}"; do
          echo "Running benchmark for model: $MODEL, num_traces: $NUM_TRACES, commute_depth: $COMMUTE_DEPTH, cycle_length: $CYCLE_LENGTH, approach: $APPROACH"
          ./target/release/stamina-toolset benchmark --model "models/$MODEL/$MODEL.crn" --num-traces "$NUM_TRACES" --commute-depth "$COMMUTE_DEPTH" --cycle-length "$CYCLE_LENGTH" --approach "$APPROACH" --output "$OUTPUT_FOLDER"
        done
      done
    done
  done
done

echo "All benchmarks completed. Running PRISM script in output/$OUTPUT_FOLDER"

chmod +x output/"$OUTPUT_FOLDER"/run_prism.sh
cd "output/$OUTPUT_FOLDER" || { echo "Failed to cd to output/$OUTPUT_FOLDER" >&2; exit 1; }
exec ./run_prism.sh
# ./output/"$OUTPUT_FOLDER"/run_prism.sh