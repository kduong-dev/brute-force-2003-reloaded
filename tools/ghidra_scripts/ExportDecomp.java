// Export every function's decompiled C, and an index, after analysis.
//   analyzeHeadless ... -postScript ExportDecomp.java <out dir>
// Writes <out>/functions.csv (address, name, size, called-by count, string refs) and
// <out>/c/<0xADDR range>.c files of 500 functions each, in address order.
//@category Export

import java.io.*;
import java.util.*;

import ghidra.app.decompiler.*;
import ghidra.app.script.GhidraScript;
import ghidra.program.model.listing.*;
import ghidra.program.model.symbol.*;
import ghidra.program.model.address.*;
import ghidra.program.model.data.StringDataInstance;

public class ExportDecomp extends GhidraScript {
    @Override
    protected void run() throws Exception {
        String[] args = getScriptArgs();
        File out = new File(args.length > 0 ? args[0] : "decomp_out");
        File cdir = new File(out, "c");
        cdir.mkdirs();

        DecompInterface di = new DecompInterface();
        DecompileOptions opt = new DecompileOptions();
        di.setOptions(opt);
        di.toggleCCode(true);
        di.toggleSyntaxTree(false);
        di.setSimplificationStyle("decompile");
        di.openProgram(currentProgram);

        Listing listing = currentProgram.getListing();
        ReferenceManager refs = currentProgram.getReferenceManager();
        List<Function> funcs = new ArrayList<>();
        for (Function f : listing.getFunctions(true)) funcs.add(f);
        println("functions: " + funcs.size());

        try (PrintWriter idx = new PrintWriter(new FileWriter(new File(out, "functions.csv")))) {
            idx.println("address,name,size,callers,strings");
            PrintWriter c = null;
            int n = 0;
            for (Function f : funcs) {
                if (monitor.isCancelled()) break;
                if (n % 500 == 0) {
                    if (c != null) c.close();
                    c = new PrintWriter(new FileWriter(new File(cdir, String.format("%08x.c", f.getEntryPoint().getOffset()))));
                }
                n++;
                // strings the function refers to (hints for naming)
                List<String> strs = new ArrayList<>();
                for (Address a : f.getBody().getAddresses(true)) {
                    for (Reference r : refs.getReferencesFrom(a)) {
                        Data d = listing.getDataAt(r.getToAddress());
                        if (d != null && d.hasStringValue()) {
                            String s = String.valueOf(d.getValue()).replace("\"", "'").replace("\n", " ").replace(",", ";");
                            if (s.length() > 60) s = s.substring(0, 60);
                            if (!strs.contains(s)) strs.add(s);
                        }
                    }
                    if (strs.size() > 8) break;
                }
                int callers = f.getCallingFunctions(monitor).size();
                idx.printf("0x%08x,%s,%d,%d,\"%s\"%n", f.getEntryPoint().getOffset(), f.getName(), f.getBody().getNumAddresses(),
                           callers, String.join(" | ", strs));
                DecompileResults res = di.decompileFunction(f, 60, monitor);
                c.printf("// ---- %s @ 0x%08x (%d bytes, %d callers)%s%n", f.getName(), f.getEntryPoint().getOffset(),
                         f.getBody().getNumAddresses(), callers, strs.isEmpty() ? "" : "  strings: " + String.join(" | ", strs));
                if (res != null && res.decompileCompleted()) {
                    c.println(res.getDecompiledFunction().getC());
                } else {
                    c.println("// decompile failed: " + (res == null ? "null" : res.getErrorMessage()));
                }
                if (n % 1000 == 0) println("decompiled " + n + " / " + funcs.size());
            }
            if (c != null) c.close();
        }
        di.dispose();
        println("done: " + out.getAbsolutePath());
    }
}
