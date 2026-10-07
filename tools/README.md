# tools/

These are the third-party tools used for reverse engineering, installed locally rather than system-wide.

- `ghidra_12.0.3_PUBLIC/`: Ghidra 12.0.3 (NSA, Apache 2.0), with the XboxDev ghidra-xbe extension in `Ghidra/Extensions/ghidra-xbe` (XBE loader plus XbSymbolDatabase). It needs Java 21 or later (Java 24 is installed).
- `ghidra_projects/BruteForce`: the analysed `default.xbe` project. Open it with `ghidra_12.0.3_PUBLIC/ghidraRun.bat`.
- `ghidra_scripts/`:
  - `ExportDecomp.java`: writes every function as C, plus an index.
  - `CreateFunctions.java`: creates the functions listed in `call_targets.txt`. Those are call targets and code addresses passed as immediates that the analysis missed; they were found by scanning `decompiled/xbe/default.text.asm`.
- `run_ghidra.sh` and `run_ghidra_pass2.sh`: headless runs. Their output and findings are in `decompiled/xbe/ghidra/` (see its README).
