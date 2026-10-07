// Create functions at the addresses listed (hex, one per line) in call_targets.txt beside this
// script: call targets and code addresses passed as immediates that Ghidra's analysis missed
// (made by the gap scan over decompiled/xbe/default.text.asm; see tools/README.md).
//@category Analysis

import java.io.*;
import java.nio.file.*;

import ghidra.app.cmd.disassemble.DisassembleCommand;
import ghidra.app.cmd.function.CreateFunctionCmd;
import ghidra.app.script.GhidraScript;
import ghidra.program.model.address.*;
import ghidra.program.model.listing.*;

public class CreateFunctions extends GhidraScript {
    @Override
    protected void run() throws Exception {
        File list = new File(getSourceFile().getParentFile().getAbsolutePath(), "call_targets.txt");
        int made = 0, skipped = 0;
        for (String line : Files.readAllLines(list.toPath())) {
            line = line.trim();
            if (line.isEmpty()) continue;
            Address a = toAddr(Long.parseLong(line, 16));
            if (getFunctionAt(a) != null) { skipped++; continue; }
            Data d = getDataAt(a);
            if (d != null && d.isDefined()) { skipped++; continue; }
            if (getInstructionAt(a) == null) {
                new DisassembleCommand(a, null, true).applyTo(currentProgram, monitor);
            }
            if (getInstructionAt(a) == null) { skipped++; continue; }
            // inside another function's body: split it off as its own function
            if (new CreateFunctionCmd(a).applyTo(currentProgram, monitor)) made++; else skipped++;
        }
        println("created " + made + " functions, skipped " + skipped);
    }
}
