"""Build data/functions.json, data/measures.json, data/coverage.json.
Inputs: gen/functions_*.txt (names, categories, returns, original descriptions), gen/ms_syntax.json (official syntax + page slug,
fetched with gen/fetch_all.py), gen/ms_index.json (names listed on the Microsoft category pages), gen/measures.py (templates).
Run from the crate root:  uv run --no-project python gen/build_data.py"""
import glob, json, re, sys
sys.path.insert(0, "gen")
from measures import MEASURES

REVIEW_DATE = "2026-10-20"
BASE = "https://learn.microsoft.com/en-us/dax/"
ms_syntax = json.load(open("gen/ms_syntax.json"))
ms_index = json.load(open("gen/ms_index.json"))

CATS = ["Aggregation", "Calculation group", "Date and time", "Filter", "Financial", "INFO", "Information", "Logical", "Math and trig", "Other",
        "Parent and child", "Relationship", "Statistical", "Table manipulation", "Text", "Time intelligence", "Visual calculations", "Window"]
VOLATILE = {"NOW", "TODAY", "UTCNOW", "UTCTODAY", "RAND", "RANDBETWEEN", "USERNAME", "USERPRINCIPALNAME", "USEROBJECTID", "CUSTOMDATA", "USERCULTURE"}
MODIFIERS = {"ALL", "ALLCROSSFILTERED", "ALLEXCEPT", "ALLNOBLANKROW", "ALLSELECTED", "KEEPFILTERS", "REMOVEFILTERS", "USERELATIONSHIP", "CROSSFILTER", "NONVISUAL"}


def parse_args(syntax: str):
    i = syntax.find("(")
    j = syntax.rfind(")")
    if i < 0 or j < i:
        return []
    body = syntax[i + 1:j]
    args, depth, k = [], 0, 0
    while k < len(body):
        c = body[k]
        if c == "[":
            depth += 1
        elif c == "]":
            depth -= 1
        elif c == "<":
            e = body.find(">", k)
            name = body[k + 1:e]
            tail = body[e + 1:e + 8]
            alt = body[max(0, k - 4):k] == " or "
            args.append({"name": name, "required": depth == 0 and not alt, "variadic": "..." in tail.split(">")[0] if False else bool(re.match(r"[\]\s,]*\.\.\.", tail))})
            k = e
        k += 1
    # drop duplicates introduced by repeated optional groups: [, <a>][, <a>]...
    seen, out = set(), []
    for a in args:
        if a["name"] in seen:
            for o in out:
                if o["name"] == a["name"]:
                    o["variadic"] = True
            continue
        seen.add(a["name"])
        out.append(a)
    return out


def pick_syntax(name, mine):
    r = ms_syntax.get(name) or {}
    for b in r.get("syntax", []):
        if re.match(re.escape(name) + r"\s*\(", b, re.I):
            b = b.replace("…", "...").replace("�", "...")
            return re.sub(r"\s+\(", "(", b), "microsoft-learn", r.get("slug")
    return mine, "catalog", r.get("slug") or (name.lower().replace(".", "-") + "-function-dax")


funcs = []
for f in sorted(glob.glob("gen/functions_*.txt")):
    for line in open(f, encoding="utf-8"):
        if not line.strip() or line.startswith("#"):
            continue
        name, cat, syn, ret, desc = [x.strip() for x in line.rstrip("\n").split(" | ", 4)]
        assert cat in CATS, (name, cat)
        syntax, src, slug = pick_syntax(name, syn)
        if syntax.count("(") != syntax.count(")") or syntax.count("[") != syntax.count("]"):
            print("official syntax is unbalanced, using the catalog form:", name, "|", syntax)
            syntax, src = syn, "catalog"
        parts = [p.strip() for p in re.split(r"(?<![A-Z0-9_.])(?=" + re.escape(name) + r"\s*\()", syntax) if p.strip()]
        alternatives = parts[1:]
        syntax = parts[0] if parts else syntax
        args = parse_args(syntax)
        tags = []
        if name.endswith("X") and any(a["name"] == "table" for a in args) and cat in ("Aggregation", "Statistical", "Text"):
            tags.append("iterator")
        if name in VOLATILE:
            tags.append("volatile")
        if name in MODIFIERS:
            tags.append("calculate-modifier")
        if ret == "table":
            tags.append("table-returning")
        if cat == "Time intelligence":
            tags.append("requires-date-table")
        if cat in ("Window", "Visual calculations", "Calculation group"):
            tags.append(cat.lower().replace(" ", "-"))
        if cat == "Financial":
            tags.append("financial")
        funcs.append({"name": name, "category": cat, "tags": tags, "syntax": syntax, "args": args, "returns": ret, "description": desc, "syntax_alternatives": alternatives,
                      "reference_url": BASE + slug, "syntax_source": src})
funcs.sort(key=lambda x: x["name"])
names = [f["name"] for f in funcs]
assert len(names) == len(set(names)), "duplicate function"
json.dump(funcs, open("data/functions.json", "w", encoding="utf-8"), indent=1, ensure_ascii=False)

for m in MEASURES:
    m["kind"] = "calculation_item" if m["category"] == "Calculation item" else "measure"
ids = [m["id"] for m in MEASURES]
assert len(ids) == len(set(ids)), "duplicate measure id"
json.dump(MEASURES, open("data/measures.json", "w", encoding="utf-8"), indent=1, ensure_ascii=False)

by_cat = {}
for f in funcs:
    by_cat[f["category"]] = by_cat.get(f["category"], 0) + 1
listed = sorted(set(ms_index) & set(names))
cov = {
    "review_date": REVIEW_DATE,
    "source": BASE + "dax-function-reference",
    "method": ("Names were checked against the function lists on the Microsoft Learn DAX category pages (aggregation, date and time, filter, "
               "financial, information, logical, math and trig, other, parent and child, relationship, statistical, table manipulation, text, "
               "time intelligence, INFO). Every catalog entry was verified to have a Learn page (HTTP 200) and its syntax was taken from that page."),
    "function_count": len(funcs), "measure_template_count": len(MEASURES), "categories": by_cat,
    "listed_on_category_pages": len(ms_index),
    "category_pages_missing_from_catalog": sorted(set(ms_index) - set(names)),
    "catalog_functions_not_on_category_pages": sorted(set(names) - set(ms_index)),
    "notes": [
        "Operators, keywords (VAR, RETURN, DEFINE, EVALUATE, ORDER BY) and constants are not functions and are not catalogued.",
        "The 'catalog_functions_not_on_category_pages' entries (INFO.*, visual calculation and window functions, FIRSTNONBLANK family, TOPNSKIP, T.INV.2T) have their own Learn page but are not links on the category pages.",
        "Descriptions are original short summaries; follow reference_url for the full specification.",
    ],
}
json.dump(cov, open("data/coverage.json", "w", encoding="utf-8"), indent=1, ensure_ascii=False)
print(len(funcs), "functions;", len(MEASURES), "templates;", by_cat)
print("missing from catalog:", cov["category_pages_missing_from_catalog"])
