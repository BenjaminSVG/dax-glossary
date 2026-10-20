"""Fetch official syntax for every catalog function (parallel) -> gen/ms_syntax.json  {NAME: {"status": 200|404|..., "slug":..., "syntax": [...]}}"""
import concurrent.futures as cf, glob, html, json, re, urllib.request, urllib.error
names = []
for f in sorted(glob.glob("gen/functions_*.txt")):
    for l in open(f, encoding="utf-8"):
        if l.strip() and not l.startswith("#"):
            names.append(l.split(" | ")[0].strip())
idx = json.load(open("gen/ms_index.json"))
def slugs(n):
    out = []
    if n in idx:
        out.append(idx[n]["slug"])
    base = n.lower().replace(".", "-")
    out += [base + "-function-dax", base + "-dax"]
    return list(dict.fromkeys(out))
def get(n):
    for s in slugs(n):
        try:
            t = urllib.request.urlopen(f"https://learn.microsoft.com/en-us/dax/{s}", timeout=40).read().decode("utf-8")
        except urllib.error.HTTPError as e:
            if e.code == 404:
                continue
            return n, {"status": e.code, "slug": s, "syntax": []}
        except Exception as e:
            return n, {"status": "ERR", "slug": s, "syntax": [], "err": str(e)[:80]}
        blocks = [html.unescape(re.sub(r"<[^>]+>", "", m)).strip() for m in re.findall(r"<pre[^>]*>\s*<code[^>]*>(.*?)</code>", t, re.S)]
        return n, {"status": 200, "slug": s, "syntax": [re.sub(r"\s+", " ", b) for b in blocks[:3]]}
    return n, {"status": 404, "slug": slugs(n)[0], "syntax": []}
res = {}
with cf.ThreadPoolExecutor(8) as ex:
    for n, r in ex.map(get, names):
        res[n] = r
json.dump(res, open("gen/ms_syntax.json", "w"), indent=1, sort_keys=True)
bad = {n: r["status"] for n, r in res.items() if r["status"] != 200}
print(len(res), "ok", len(res) - len(bad), "not ok:", bad)
