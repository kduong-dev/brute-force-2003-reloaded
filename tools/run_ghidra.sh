#!/bin/sh
# Headless Ghidra: import default.xbe (XBE loader), auto-analyse, export decompiled C.
#   sh tools/run_ghidra.sh            (from the project root; takes a while)
# Project: tools/ghidra_projects/BruteForce (open it in the Ghidra GUI to browse).
# Output:  decompiled/xbe/ghidra/{functions.csv,c/*.c}
# Ghidra's .bat launcher breaks on the space in "XBE Mod": run it through the 8.3 short path.
ROOT='C:\Users\Kevin\projects\github\XBEMOD~1'
cmd //c "$ROOT\tools\ghidra_12.0.3_PUBLIC\support\analyzeHeadless.bat" "$ROOT\tools\ghidra_projects" BruteForce \
  -import "$ROOT\BRUTEF~1\default.xbe" -overwrite \
  -scriptPath "$ROOT\tools\ghidra_scripts" \
  -postScript ExportDecomp.java "$ROOT\decompiled\xbe\ghidra" \
  -max-cpu 12
