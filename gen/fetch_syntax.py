"""Fetch the Syntax block and first description sentence of Microsoft Learn function pages: fetch_syntax.py NAME [NAME...]"""
import html, re, sys, urllib.request
def slug(n):
    return n.lower().replace(".", "-") + "-function-dax"
for n in sys.argv[1:]:
    try:
        t = urllib.request.urlopen(f"https://learn.microsoft.com/en-us/dax/{slug(n)}", timeout=30).read().decode("utf-8")
    except Exception as e:
        print(n, "ERR", e); continue
    m = re.search(r"<pre[^>]*>\s*<code[^>]*>(.*?)</code>", t, re.S)
    syn = html.unescape(re.sub(r"<[^>]+>", "", m.group(1))).strip().replace("\n", " ") if m else "?"
    d = re.search(r'<meta name="description" content="([^"]*)"', t)
    print(f"{n} | {syn} | {html.unescape(d.group(1)) if d else ''}")
