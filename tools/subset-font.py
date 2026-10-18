#!/usr/bin/env python3
"""Build RWE's static CJK font asset; developer tool, not a Cargo build step.
Requires fonttools==4.67.0. Input is the pinned upstream Noto Sans SC variable TTF.
"""
import argparse
import hashlib
from pathlib import Path
import fontTools
from fontTools import subset
from fontTools.ttLib import TTFont
from fontTools.varLib.instancer import instantiateVariableFont

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("source", type=Path)
parser.add_argument("output", type=Path)
args = parser.parse_args()
if fontTools.__version__ != "4.67.0":
    parser.error("regeneration requires fonttools==4.67.0")
expected = "a3041811a78c361b1de50f953c805e0244951c21c5bd412f7232ef0d899af0da"
if hashlib.sha256(args.source.read_bytes()).hexdigest() != expected:
    parser.error("source hash does not match the reviewed Noto Sans SC revision")
font = TTFont(args.source, recalcTimestamp=False)
instantiateVariableFont(font, {"wght": 400}, inplace=True, optimize=True)
options = subset.Options()
options.hinting = False
options.layout_features = []
options.drop_tables += ["GSUB", "GPOS", "GDEF", "DSIG", "BASE", "vhea", "vmtx", "VORG", "STAT"]
options.recalc_timestamp = False
ranges = [(0x20, 0x24F), (0x2000, 0x206F), (0x3000, 0x30FF), (0x3400, 0x9FFF), (0xFF00, 0xFFEF)]
codepoints = {code for start, end in ranges for code in range(start, end + 1)}
worker = subset.Subsetter(options=options)
worker.populate(unicodes=codepoints)
worker.subset(font)
# Rename the derivative and retain the original copyright and OFL records.
for name in font["name"].names:
    values = {1: "RWE CJK", 2: "Regular", 3: "RWE CJK Regular 1", 4: "RWE CJK Regular", 6: "RWECJK-Regular", 16: "RWE CJK", 17: "Regular"}
    if name.nameID in values:
        name.string = values[name.nameID].encode(name.getEncoding())
font.recalcTimestamp = False
args.output.parent.mkdir(parents=True, exist_ok=True)
font.save(args.output)
print("Glyphs:", len(font.getBestCmap()))
print("Bytes:", args.output.stat().st_size)
print("SHA-256:", hashlib.sha256(args.output.read_bytes()).hexdigest())
