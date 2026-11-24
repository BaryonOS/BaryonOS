"""Static stack bound from objdump: per-function frame (pushes + sub rsp + 8 return address), max over direct-call paths.

Direct tail jumps (`jmp <other function>`) are followed conservatively as if they were calls, so the
jumping frame is counted on top of the target's (an over-estimate, never an under-estimate).
Indirect calls and jumps are not followed. A recursive cycle or an unknown root makes the result
incomplete: the script then prints the reason and exits 1 instead of reporting a bound.
Usage: python3 -I stackbound.py <elf> [root]
"""
import re, subprocess, sys, functools
dis = subprocess.run(["objdump", "-d", "--no-show-raw-insn", "-M", "intel", sys.argv[1]], capture_output=True, text=True, check=True).stdout
funcs, calls, cur, indirect = {}, {}, None, set()
for line in dis.splitlines():
    m = re.match(r"^([0-9a-f]+) <(.+)>:$", line)
    if m: cur = m.group(2); funcs[cur] = 8; calls[cur] = set(); continue
    if cur is None: continue
    if re.search(r"\tpush\s", line): funcs[cur] += 8
    m = re.search(r"\tsub\s+rsp,(0x[0-9a-f]+|\d+)", line)
    if m: funcs[cur] += int(m.group(1), 0)
    m = re.search(r"\t(call|jmp)\s+[0-9a-f]+ <([^>+]+)(?:\+0x[0-9a-f]+)?>", line)
    if m:
        # A same-function jmp is a local loop; a same-function call is recursion and must stay.
        if m.group(1) == "call" or m.group(2) != cur: calls[cur].add(m.group(2))
    elif re.search(r"\tcall\s", line): indirect.add(cur)
root = sys.argv[2] if len(sys.argv) > 2 else "_start"
if root not in funcs:
    sys.exit(f"incomplete: root {root!r} is not a function in {sys.argv[1]}")
cycles = set()
sys.setrecursionlimit(10000)
@functools.lru_cache(None)
def depth(f, seen=frozenset()):
    if f in seen:
        cycles.add(f)
        return 0
    return funcs.get(f, 0) + max([depth(c, seen | {f}) for c in calls.get(f, ())] or [0])
bound = depth(root)
if cycles:
    sys.exit(f"incomplete: recursion through {sorted(cycles)[:3]}; no bound reported")
print(f"{root} static bound (direct calls and tail jumps) = {bound} bytes; functions with indirect calls: {len(indirect)}")
top = sorted(funcs.items(), key=lambda x: -x[1])[:5]
print("largest frames:", ", ".join(f"{n[:60]}={s}" for n, s in top))
