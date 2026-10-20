import json, glob
ms = json.load(open("gen/ms_index.json"))
mine = {}
for f in sorted(glob.glob("gen/functions_*.txt")):
    for l in open(f, encoding="utf-8"):
        if l.strip() and not l.startswith("#"):
            p = [x.strip() for x in l.split(" | ")]
            mine[p[0]] = p[1]
print("ms", len(ms), "mine", len(mine))
print("MISSING in mine:", sorted(set(ms) - set(mine)))
print("EXTRA in mine:", sorted(set(mine) - set(ms)))
