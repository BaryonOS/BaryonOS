"""Static stack bound from objdump: per-function frame (pushes + sub rsp + 8 return address), max over direct-call paths. Indirect calls are not followed."""
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
    m = re.search(r"\tcall\s+[0-9a-f]+ <([^>+]+)>", line)
    if m: calls[cur].add(m.group(1))
    elif re.search(r"\tcall\s", line): indirect.add(cur)
sys.setrecursionlimit(10000)
@functools.lru_cache(None)
def depth(f, seen=frozenset()):
    if f in seen: return 0  # recursion: bound reported as unbounded below
    return funcs.get(f, 0) + max([depth(c, seen | {f}) for c in calls.get(f, ())] or [0])
root = sys.argv[2] if len(sys.argv) > 2 else "_start"
print(f"{root} static bound (direct calls) = {depth(root)} bytes; functions with indirect calls: {len(indirect)}")
top = sorted(funcs.items(), key=lambda x: -x[1])[:5]
print("largest frames:", ", ".join(f"{n[:60]}={s}" for n, s in top))
