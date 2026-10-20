"""Download the Microsoft Learn DAX category pages into gen/cache/ (git-ignored); input of gen/ms_index.py."""
import os, urllib.request

PAGES = ["aggregation", "date-and-time", "filter", "financial", "information", "logical", "math-and-trig", "other",
         "parent-and-child", "relationship", "statistical", "table-manipulation", "text", "time-intelligence"]
os.makedirs("gen/cache", exist_ok=True)
for p in PAGES + ["info"]:
    url = f"https://learn.microsoft.com/en-us/dax/{p}-functions-dax"
    with urllib.request.urlopen(url, timeout=60) as r:
        open(f"gen/cache/{p}.html", "wb").write(r.read())
    print("downloaded", url)
