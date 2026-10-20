"""Render assets/terminal.svg: a terminal window showing real dax-glossary output (copied from a run of the binary).
Run from the crate root:  python gen/make_terminal_svg.py"""
from html import escape

NB = " "   # SVG collapses plain spaces; non-breaking spaces keep the terminal columns

# (kind, text)  kind: p = prompt command, h = heading line, d = dim, k = keyword, s = string/ref, o = output, b = blank
LINES = [
    ("p", "dax-glossary get function TOTALYTD"),
    ("h", "TOTALYTD  [Time intelligence]  returns scalar"),
    ("k", "  TOTALYTD(<expression>,<dates> or <calendar>[,<filter>][,<year_end_date>])"),
    ("o", "  Year-to-date value of an expression."),
    ("d", "  tags: requires-date-table"),
    ("s", "  https://learn.microsoft.com/en-us/dax/totalytd-function-dax"),
    ("b", ""),
    ("p", "dax-glossary search rolling"),
    ("o", "measure  rolling-12m     Total of the last 12 months ending at the last date in context."),
    ("o", "measure  rolling-days    Total of the last N days ending at the last date in context."),
    ("b", ""),
    ("p", "dax-glossary render yoy-pct --param 'measure_name=Sales YoY %' \\"),
    ("p2", "    --param 'measure=[Sales]' --param \"dates='Date'[Date]\""),
    ("h", "Sales YoY % ="),
    ("k", "VAR _prior = CALCULATE([Sales], SAMEPERIODLASTYEAR('Date'[Date]))"),
    ("k", "RETURN DIVIDE([Sales] - _prior, _prior)"),
]
COLORS = {"p": "#e2e8f0", "p2": "#e2e8f0", "h": "#f0abfc", "d": "#94a3b8", "k": "#7dd3fc", "s": "#6ee7b7", "o": "#cbd5e1"}
LH, X0, Y0 = 24, 28, 78
H = Y0 + LH * len(LINES) + 30
out = [f'<svg xmlns="http://www.w3.org/2000/svg" width="960" height="{H}" viewBox="0 0 960 {H}" role="img" aria-label="dax-glossary command line demo">',
       '<defs><linearGradient id="g" x1="0" y1="0" x2="1" y2="1"><stop offset="0" stop-color="#0b1020"/><stop offset="1" stop-color="#1a1238"/></linearGradient></defs>',
       f'<rect width="960" height="{H}" rx="14" fill="url(#g)" stroke="#ffffff" stroke-opacity="0.12"/>',
       '<rect width="960" height="44" rx="14" fill="#ffffff" fill-opacity="0.05"/>',
       '<circle cx="26" cy="22" r="6.5" fill="#ff5f57"/><circle cx="48" cy="22" r="6.5" fill="#febc2e"/><circle cx="70" cy="22" r="6.5" fill="#28c840"/>',
       '<text x="480" y="27" text-anchor="middle" font-family="Segoe UI,Arial,sans-serif" font-size="13" fill="#94a3b8">dax-glossary - terminal</text>',
       '<g font-family="Cascadia Code,JetBrains Mono,Consolas,Courier New,monospace" font-size="15" style="white-space:pre">']
for i, (kind, text) in enumerate(LINES):
    y = Y0 + i * LH
    if kind == "b":
        continue
    if kind == "p":
        out.append(f'<text x="{X0}" y="{y}"><tspan fill="#34d399">$</tspan><tspan fill="{COLORS[kind]}">{NB}{escape(text).replace(" ", NB)}</tspan></text>')
    else:
        out.append(f'<text x="{X0}" y="{y}" fill="{COLORS[kind]}">{escape(text).replace(" ", NB)}</text>')
out.append("</g></svg>")
open("assets/terminal.svg", "w", encoding="utf-8").write("\n".join(out))
print("assets/terminal.svg written")
