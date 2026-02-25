# This script uses Prism to check every state individually and
# then combine that information into a JSON file.

# Usage: python3 prism_all_states.py <filename from .tra,.sta,.lab> <max csl time bound>

import sys
import shutil
import subprocess
import json

class CVASResult:
    dim: int
    frames: list
    def __init__(self, dim):
        self.dim = dim
        self.frames = []

    def to_dict(self):
        return {
            "dim": self.dim,
            "frames": [self.frame_to_dict(frame) for frame in self.frames]
        }

    @staticmethod
    def frame_to_dict(frame):
        return {
            "timestep": frame.timestep,
            "stateframes": [
                {
                    "state_index": sf.state_index,
                    "state_vector": sf.state_vector,
                    "probability": sf.probability
                } for sf in frame.stateframes
            ]
        }

class Frame:
    timestep: int
    stateframes: list
    def __init__(self, timestep):
        self.timestep = timestep
        self.stateframes = []

class StateFrame:
    state_index: int
    state_vector: str
    probability: float
    def __init__(self, state_index, state_vector, probability):
        self.state_index = state_index
        self.state_vector = state_vector
        self.probability = probability

if __name__ == "__main__":
    filename = sys.argv[1]
    timebound = int(sys.argv[2])

    # Get the number of states
    with open(filename + ".tra", "r") as f:
        num_states = int(f.readline().split()[0])

    # Get the number of variables
    with open(filename + ".sta", "r") as f:
         num_variables = len(f.readline().split(","))

    # Import .tra and .sta files as-is
    shutil.copyfile(filename + ".sta", "vis.sta")
    shutil.copyfile(filename + ".tra", "vis.tra")

    # Prepare result dump
    results = CVASResult(num_variables)

    for timestep in range(1,timebound+1):
        # Property doesn't change from timestep
        property_string = f'P=? [ F={timestep} "interesting" ]'
        print("Checking", property_string)
        with open("vis.prop", "w") as csl_file:
            csl_file.write(property_string)
        
        frame = Frame(timestep)

        with open("vis.sta", "r") as state_list:
            state_list.readline()
            for state_index in range(1,num_states):
                state_vector = state_list.readline().split(" ")[1]
                label_string = f"{state_index}: 2"
                with open("vis.lab", "w") as label_file:
                    label_file.write(f'0="init" 1="deadlock" 2="interesting"\n0: 1\n1: 0\n{label_string}')
                print("Prism on state", state_index, "of", num_states)
                # prism -importmodel <output>.tra,sta,lab <output>.prop -ctmc
                prism_call = ["prism", "-importmodel", "vis.tra,sta,lab", "vis.prop", "-ctmc"]
                output = subprocess.check_output(prism_call)
                # output is bytes; decode to text and split into lines
                split_output = output.decode("utf-8", errors="replace").splitlines()
                for line in split_output:
                    if "Result: " in line:
                        probability = float(line.split(" ")[1])
                        stateframe = StateFrame(state_index, state_vector, probability)
                        frame.stateframes.append(stateframe)
        
        results.frames.append(frame)
    
    results_json = json.dumps(results.to_dict())
    with open("output.json", "w") as output_file:
        output_file.write(results_json)