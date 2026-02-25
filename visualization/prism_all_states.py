# This script uses Prism to check every state individually and
# then combine that information into a JSON file.

# Usage: python3 prism_all_states.py <filename from .tra,.sta,.lab> <max csl time bound>

import sys
import shutil
import subprocess

if __name__ == "__main__":
    filename = sys.argv[1]
    timebound = int(sys.argv[2])

    # Get the number of states
    with open(filename + ".tra", "r") as f:
        num_states = int(f.readline().split()[0])

    # Import .tra and .sta files as-is
    shutil.copyfile(filename + ".sta", "vis.sta")
    shutil.copyfile(filename + ".tra", "vis.tra")

    for timestep in range(1,timebound):
        # Property doesn't change from timestep
        property_string = f'P=? [ F={timestep} "interesting" ]'
        print("Checking", property_string)
        with open("vis.prop", "w") as csl_file:
            csl_file.write(property_string)

        for state in range(1,num_states):
            label_string = f"{state}: 2"
            with open("vis.lab", "w") as label_file:
                label_file.write(f'0="init" 1="deadlock" 2="interesting"\n0: 1\n1: 0\n{label_string}')
            print("Prism on state", state, "of", num_states)
            # prism -importmodel <output>.tra,sta,lab <output>.prop -ctmc
            prism_call = ["prism", "-importmodel", "vis.tra,sta,lab", "vis.prop", "-ctmc"]
            output = subprocess.check_output(prism_call)
            # output is bytes; decode to text and split into lines
            split_output = output.decode("utf-8", errors="replace").splitlines()
            for line in split_output:
                if "Result: " in line:
                    result = float(line.split(" ")[1])
                    print(result)