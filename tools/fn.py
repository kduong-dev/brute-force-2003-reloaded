"""Print decompiled functions from decompiled/xbe/ghidra.
usage: python tools/fn.py ADDR_OR_NAME ...   (an address inside a function finds that function)"""
import bisect, csv, glob, os, re, sys
G = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..', 'decompiled', 'xbe', 'ghidra')
rows = sorted(((int(r['address'], 16), r['name'], int(r['size'])) for r in csv.DictReader(open(os.path.join(G, 'functions.csv')))))
starts = [r[0] for r in rows]
files = sorted(glob.glob(os.path.join(G, 'c', '*.c')))
file_starts = [int(os.path.basename(f)[:-2], 16) for f in files]
for q in sys.argv[1:]:
    if q.startswith('FUN_') or not re.fullmatch(r'(0x)?[0-9a-fA-F]+', q):
        hit = [r for r in rows if r[1] == q]
    else:
        a = int(q, 16); i = bisect.bisect_right(starts, a) - 1
        hit = [rows[i]] if i >= 0 else []
    if not hit:
        print('not found:', q); continue
    addr, name, size = hit[0]
    f = files[bisect.bisect_right(file_starts, addr) - 1]
    text = open(f, encoding='utf-8', errors='replace').read()
    m = re.search(r'^// ---- %s @ 0x%08x.*?(?=^// ---- |\Z)' % (re.escape(name), addr), text, re.M | re.S)
    print(m.group(0) if m else f'{name} @ {addr:#x}: not in {f}')
