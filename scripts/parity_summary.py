#!/usr/bin/env python3 -I
"""Rewrite the Done / Total column of docs/parity/checklist.md for the REST inventories and the ADF/UI-kit tables.

  parity_summary.py          rewrite
  parity_summary.py --check  fail if the checklist is stale
"""
import os, re, sys

D = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "docs/parity")
SOURCES = {  # checklist link target -> status column index counted from the row's cells
    "confluence-rest-v2.md": "rest", "confluence-rest-v1.md": "rest",
    "confluence-adf.md": "adf", "ui-kit.md": "kit",
}

def count(fname, kind):
    done = part = total = 0
    for l in open(os.path.join(D, fname)):
        if not l.startswith("| ") or l.startswith("| ---") or l.startswith("|---"):
            continue
        cells = [c.strip() for c in l.strip().strip("|").split("|")]
        if kind == "rest":
            if cells[0] not in ("GET", "POST", "PUT", "DELETE", "PATCH"): continue
            st = cells[-2]
        elif kind == "kit":
            if len(cells) < 3 or cells[0] == "Component": continue
            st = cells[-1]
        else:
            if not cells[0].startswith("`"): continue
            st = cells[1:-1]
            st = "done" if all(s == "done" for s in st) else "partial" if any(s != "not started" for s in st) else "not started"
        total += 1
        done += st == "done"; part += st.startswith("partial")
    return done, part, total

p = os.path.join(D, "checklist.md"); s = open(p).read(); out = s
for f, kind in SOURCES.items():
    d, pa, t = count(f, kind)
    out = re.sub(rf"(\({re.escape(f)}\) \| )[^|]*\|", rf"\g<1>{d} done, {pa} partial / {t} |", out)
if "--check" in sys.argv: sys.exit(0 if out == s else "checklist.md is stale; run scripts/parity_summary.py")
open(p, "w").write(out); print(out[out.index("## Summary"):])
