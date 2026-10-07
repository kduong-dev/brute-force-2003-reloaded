#!/bin/sh
# Second pass on the saved project: create the functions the first analysis missed
# (tools/ghidra_scripts/call_targets.txt), re-analyse, re-export decompiled C.
ROOT='C:\Users\Kevin\projects\github\XBEMOD~1'
cmd //c "$ROOT\tools\ghidra_12.0.3_PUBLIC\support\analyzeHeadless.bat" "$ROOT\tools\ghidra_projects" BruteForce \
  -process default.xbe \
  -scriptPath "$ROOT\tools\ghidra_scripts" \
  -preScript CreateFunctions.java \
  -postScript ExportDecomp.java "$ROOT\decompiled\xbe\ghidra" \
  -max-cpu 12
