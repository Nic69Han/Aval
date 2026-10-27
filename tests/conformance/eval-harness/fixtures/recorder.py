#!/usr/bin/env python3
# Command-provider stand-in: saves the prompt read on stdin as <record_dir>/prompt_<n>.txt and prints
# <answers_dir>/<n>.txt (the last file is repeated when n is larger than the number of files).
import os, sys
record_dir, answers_dir = sys.argv[1], sys.argv[2]
n = len([f for f in os.listdir(record_dir) if f.startswith("prompt_")]) + 1
data = sys.stdin.read()
open(os.path.join(record_dir, "prompt_%d.txt" % n), "w", encoding="utf-8").write(data)
names = sorted(os.listdir(answers_dir))
pick = "%d.txt" % n if "%d.txt" % n in names else names[-1]
sys.stdout.write(open(os.path.join(answers_dir, pick), encoding="utf-8").read())
