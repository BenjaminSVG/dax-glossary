"""Authoritative function names + doc slugs scraped from the Microsoft Learn category pages (downloaded by gen/download_pages.py to gen/cache) -> gen/ms_index.json"""
import glob, html, json, os, re
idx = {}
for f in sorted(glob.glob("gen/cache/*.html")):
    cat = os.path.basename(f)[:-5]
    t = open(f, encoding="utf-8").read()
    for m in re.finditer(r'<a href="([a-z0-9-]+)" data-linktype="relative-path">([^<]+)</a>', t):
        slug, txt = m.group(1), html.unescape(m.group(2)).strip()
        mm = re.match(r"([A-Z][A-Z0-9_.]*)(\s|$)", txt)
        if mm and (slug.endswith("-function-dax") or slug.startswith("info-") or slug.endswith("-dax")):
            e = idx.setdefault(mm.group(1), {"slug": slug, "pages": []})
            if cat not in e["pages"]:
                e["pages"].append(cat)
json.dump(idx, open("gen/ms_index.json", "w"), indent=1, sort_keys=True)
print(len(idx))
