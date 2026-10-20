"""Fill gen/readme/README.{en,es}.tpl.md with live numbers and tables from data/*.json -> README.md, README.es.md.
Run from the crate root:  python gen/build_readme.py"""
import glob, json, re

funcs = json.load(open("data/functions.json", encoding="utf-8"))
meas = json.load(open("data/measures.json", encoding="utf-8"))
tests = sum(len(re.findall(r"#\[test\]", open(f, encoding="utf-8").read())) for f in glob.glob("tests/*.rs"))

ICON = {"Aggregation": "➕", "Calculation group": "🧮", "Date and time": "📅", "Filter": "🔍", "Financial": "💰", "INFO": "ℹ️", "Information": "🧠",
        "Logical": "🔀", "Math and trig": "📐", "Other": "🧰", "Parent and child": "🌳", "Relationship": "🔗", "Statistical": "📊",
        "Table manipulation": "🗃️", "Text": "🔤", "Time intelligence": "⏱️", "Visual calculations": "🎛️", "Window": "🪟"}
EX = {"Aggregation": "SUM, AVERAGEX, DISTINCTCOUNT", "Calculation group": "SELECTEDMEASURE", "Date and time": "DATEDIFF, EOMONTH",
      "Filter": "CALCULATE, FILTER, ALL", "Financial": "PMT, XIRR, XNPV", "INFO": "INFO.TABLES, INFO.MEASURES",
      "Information": "ISBLANK, HASONEVALUE", "Logical": "IF, SWITCH, COALESCE", "Math and trig": "DIVIDE, ROUND", "Other": "BLANK, ERROR",
      "Parent and child": "PATH, PATHITEM", "Relationship": "RELATED, USERELATIONSHIP", "Statistical": "RANKX, MEDIAN, STDEV.S",
      "Table manipulation": "SUMMARIZECOLUMNS, TOPN", "Text": "FORMAT, CONCATENATEX", "Time intelligence": "TOTALYTD, DATEADD",
      "Visual calculations": "RUNNINGSUM, MOVINGAVERAGE", "Window": "INDEX, OFFSET, WINDOW"}


def category_table(lang):
    counts = {}
    for f in funcs:
        counts[f["category"]] = counts.get(f["category"], 0) + 1
    top = max(counts.values())
    h = ("| Category | Functions | | Examples |\n| --- | ---: | --- | --- |\n" if lang == "en"
         else "| Categoría | Funciones | | Ejemplos |\n| --- | ---: | --- | --- |\n")
    rows = []
    for cat, n in sorted(counts.items(), key=lambda kv: (-kv[1], kv[0])):
        bar = "█" * max(1, round(20 * n / top))
        rows.append(f"| {ICON.get(cat, '•')} {cat} | **{n}** | `{bar}` | {EX.get(cat, '')} |")
    return h + "\n".join(rows)


def template_catalog():
    cats = {}
    for m in meas:
        cats.setdefault(m["category"], []).append(m)
    out = []
    for cat in sorted(cats):
        out.append(f"#### {cat}\n\n| id | what it renders | parameters |\n| --- | --- | --- |")
        for m in sorted(cats[cat], key=lambda x: x["id"]):
            ps = ", ".join(f"`{p['name']}`" for p in m["parameters"] if p["name"] != "measure_name")
            out.append(f"| `{m['id']}` | {m['description'].replace('|', '/')} | {ps} |")
        out.append("")
    return "\n".join(out)


for lang, tpl, dst in (("en", "gen/readme/README.en.tpl.md", "README.md"), ("es", "gen/readme/README.es.tpl.md", "README.es.md")):
    t = open(tpl, encoding="utf-8").read()
    for k, v in {"{{FUNCTIONS}}": str(len(funcs)), "{{TEMPLATES}}": str(len(meas)), "{{TESTS}}": str(tests),
                 "{{CATEGORY_TABLE}}": category_table(lang), "{{TEMPLATE_CATALOG}}": template_catalog()}.items():
        t = t.replace(k, v)
    assert "{{" not in re.sub(r"```.*?```", "", t, flags=re.S).replace("{{measure}}", "").replace("{{dates}}", ""), "unresolved placeholder"
    open(dst, "w", encoding="utf-8", newline="\n").write(t)
    print("wrote", dst, len(t))
