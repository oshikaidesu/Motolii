#!/usr/bin/env python3
"""「普通のモーショングラフィックが出来る」までを段階化する。

2026-08-23 利用者裁定:
> 先回りで無い機能を作るより、実際にペルソナや手順書を書かせて、普通のモーション
> グラフィックが出来るまでを step 化し、目標を段階化すべき。それなら進捗も分かり
> やすく、私の確認も認知負荷が少ない、今後の拡張性にも強みが出る。

**段階は発明しない。** 手順書(`next/reference/procedures/P*.md`)が既に節
(`### A. 新規プロジェクトを開く` 等)で区切っており、各手順に判定
(書ける / 【対象外】 / 【穴】入口が無い / 【穴】意味が無い / 【未確認】)が付いている。
節をそのまま段階とし、**その節の全手順が「書ける」または「対象外」になったら段階が通る**。

**次にやる仕事は「まだ通っていない最も早い段階」から選ぶ。** これが
「先回りで作らない」の機械的な担保になる。
"""
import io, os, re, sys, glob, collections

ROOT = sys.argv[1] if len(sys.argv) > 1 else "."
PROC = os.path.join(ROOT, "next/reference/procedures")
ROW = re.compile(r'^\|\s*(\d+)\s*\|(.+?)\|(.+?)\|(.+?)\|(.+?)\|\s*$')

def verdict_of(cell):
    c = cell.strip()
    if "【対象外】" in c: return "対象外"
    if "書ける" in c: return "書ける"
    if "入口が無い" in c: return "入口が無い"
    if "意味が無い" in c: return "意味が無い"
    if "未確認" in c: return "未確認"
    return "?"

books = []
for path in sorted(glob.glob(os.path.join(PROC, "P*.md"))):
    title = os.path.basename(path)[:-3]
    stages, cur = [], None
    for line in io.open(path, encoding="utf-8"):
        # 手順書は3本とも別レーンが書いたので見出しの深さが揃っていない
        # (P1 は `## 0. …`、P3 は `### A. …`、P2 は見出し無しの通し表)。
        # **どの深さでも段階として拾い**、手順行を持たない節は後で捨てる。
        h = re.match(r'^#{2,3}\s+(.+?)\s*$', line)
        if h:
            cur = {"name": h.group(1), "rows": []}
            stages.append(cur); continue
        m = ROW.match(line)
        if m and m.group(1).isdigit():
            v = verdict_of(m.group(5))
            if v == "?":
                continue          # 集計表など、判定を持たない表は段階ではない
            if cur is None:       # P2: 見出しの前に表が始まる
                cur = {"name": "(通し)", "rows": []}
                stages.append(cur)
            cur["rows"].append({"n": int(m.group(1)), "what": m.group(2).strip(),
                                "verdict": v})
    books.append({"title": title, "stages": [s for s in stages if s["rows"]]})

out = ["# 段階(機械導出)", "",
       "`scripts/plan_steps.py` が生成。**手で編集しない。**", "",
       "**段階は発明していない** — 手順書(`procedures/P*.md`)の節をそのまま段階とし、",
       "各手順の判定を数えた。**その節の全手順が「書ける」または「対象外」になったら段階が通る。**", "",
       "**次にやる仕事は「まだ通っていない最も早い段階」から選ぶ。**",
       "これが「先回りで作らない」の機械的な担保。", "",
       "**「通る」は2種類。** 私(supervisor/レーン)が静的に到達できるのは **静通** まで —",
       "`【未確認】` は窓を開けないと判定できないので、**利用者の検分でしか実通にならない**。",
       "静通を先に全段階そろえ、実機確認は最後にまとめて1回にするのが認知負荷が低い。", ""]
first_open = None
for b in books:
    total = sum(len(s["rows"]) for s in b["stages"])
    ok = sum(1 for s in b["stages"] for r in s["rows"] if r["verdict"] == "書ける")
    out += [f"## {b['title']} — {ok}/{total} 手順が書ける", "",
            "| 段階 | 手順 | 書ける | 対象外 | 入口が無い | 意味が無い | 未確認 | 静通 | 実通 |",
            "|---|---|---|---|---|---|---|---|---|"]
    for s in b["stages"]:
        c = collections.Counter(r["verdict"] for r in s["rows"])
        # 静通 = 穴がゼロ(未確認は許す)。実通 = 未確認もゼロ。
        static_ok = c["入口が無い"] == 0 and c["意味が無い"] == 0
        real_ok = static_ok and c["未確認"] == 0
        if not static_ok and first_open is None:
            first_open = (b["title"], s["name"])
        out.append(f"| {s['name']} | {len(s['rows'])} | {c['書ける']} | {c['対象外']} | "
                   f"{c['入口が無い']} | {c['意味が無い']} | {c['未確認']} | "
                   f"{'○' if static_ok else '—'} | {'○' if real_ok else '—'} |")
    out.append("")
    for s in b["stages"]:
        blockers = [r for r in s["rows"] if r["verdict"] in ("入口が無い", "意味が無い")]
        if blockers:
            out += [f"### {b['title']} / {s['name']} を通すには", ""]
            for r in blockers:
                out.append(f"- **{r['verdict']}** #{r['n']} {r['what'][:88]}")
            out.append("")
if first_open:
    out.insert(9, f"**いま最も早い静通していない段階: {first_open[0]} / {first_open[1]}**\n")
os.makedirs(os.path.join(ROOT, "next/reference/generated"), exist_ok=True)
io.open(os.path.join(ROOT, "next/reference/generated/steps.md"), "w", encoding="utf-8").write("\n".join(out) + "\n")
tot = sum(len(s['rows']) for b in books for s in b['stages'])
okk = sum(1 for b in books for s in b['stages'] for r in s['rows'] if r['verdict']=="書ける")
print(f"手順 {tot} / 書ける {okk} / 段階 {sum(len(b['stages']) for b in books)}")
stat = sum(1 for b in books for s in b['stages']
           if not [r for r in s['rows'] if r['verdict'] in ("入口が無い","意味が無い")])
print(f"静通 {stat}/{sum(len(b['stages']) for b in books)} 段階")
if first_open: print(f"最も早い静通していない段階: {first_open[0]} / {first_open[1]}")
