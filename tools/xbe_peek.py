"""Read values from default.xbe by virtual address.
usage: python tools/xbe_peek.py f|i|x ADDR[-END] ...   (f floats, i ints, x hex dwords)"""
import struct, sys
D = open('Brute Force/default.xbe', 'rb').read()
base = struct.unpack_from('<I', D, 0x104)[0]
n = struct.unpack_from('<I', D, 0x11c)[0]; sh = struct.unpack_from('<I', D, 0x120)[0] - base
SECS = [struct.unpack_from('<5I', D, sh + i * 56)[1:5] for i in range(n)]
def off(v):
    for va, vs, ra, rs in SECS:
        if va <= v < va + rs: return ra + v - va
def rd(v, fmt):
    o = off(v); return None if o is None else struct.unpack_from(fmt, D, o)[0]
if __name__ == '__main__':
    kind = sys.argv[1]
    for a in sys.argv[2:]:
        lo, hi = (a.split('-') + [None])[:2]
        lo = int(lo, 16); hi = int(hi, 16) if hi else lo + 4
        for v in range(lo, hi, 4):
            x = rd(v, {'f': '<f', 'i': '<i', 'x': '<I'}[kind])
            print(f'{v:#x}: {x:.6g}' if kind == 'f' and x is not None else f'{v:#x}: {x if kind=="i" else hex(x)}')
