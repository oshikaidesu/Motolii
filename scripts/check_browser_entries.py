#!/usr/bin/env python3
"""**追加できる語彙は、すべてブラウザに札がある**か(裁定 2026-08-22
「追加するものは Browser の中に全部入れる」の機械照合)。

Lottie 被覆(`lottie_coverage.rs`)と同じ形 — **閉じた集合**を上流から取り、
表(=ブラウザのカタログ)との差を両方向で落とす。上流は
`motolii-vector` の2つの enum:

- `PathSource`  … 新規シェイプレイヤーが持てるパス源(= Create 札)
- `OpKind`      … 選択中シェイプへ積める演算子(= 選択へ適用する札)

型・描画・書き出し・Intent が揃っているのに札だけ無い語彙は、
**利用者から到達不能**(取り込みも無いので試験の中にしか存在できない)。
"""
import re, sys, pathlib

root = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else ".")
vec = (root / "next/engine/motolii-vector/src/lib.rs").read_text(encoding="utf-8")
cards = (root / "next/ui/motolii-browser-pane/src/model/tabs.rs").read_text(encoding="utf-8")

def variants(enum):
    m = re.search(r"pub enum %s \{(.*?)\n\}" % enum, vec, re.S)
    if not m:
        sys.exit(f"{enum} が motolii-vector に見つからない")
    return re.findall(r"^\s{4}([A-Z]\w*)", re.sub(r"//[^\n]*", "", m.group(1)), re.M)

# 札の id は kebab-case(`polystar`/`trim-path`)。variant 名から素直に導く。
def slug(v):
    return re.sub(r"(?<!^)(?=[A-Z])", "-", v).lower()

ids = set(re.findall(r'id:\s*"([^"]+)"', cards))
missing = []
for enum, why in (("PathSource", "Create 札"), ("OpKind", "選択へ適用する札")):
    for v in variants(enum):
        if v == "Bezier":
            continue  # ペン道具は Stage 側の入口(札ではない)— 裁定 待ち
        s = slug(v)
        if s not in ids and s.replace("-", "") not in {i.replace("-", "") for i in ids}:
            missing.append((enum, v, s, why))

print(f"到達できる語彙 {len(ids)}札 / 札の無い語彙 {len(missing)}件")
if missing:
    print("\n型も描画も書き出しも在るのに、ブラウザに札が無い(= 利用者は作れない):")
    for enum, v, s, why in missing:
        print(f"  - {enum}::{v}  → 要 {why}(id 案 `{s}`)")
    sys.exit(1)
