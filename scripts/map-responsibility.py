#!/usr/bin/env python3
"""責任の地図 — Motolii 自身の Rust を module ごとに割って、どれだけ重いか、
互いにどう絡んでいるか、そして**外の何本のコンセントを挿しているか**を 1 枚の HTML にする。

「外」は依存グラフに実在する package 名だけを認める(内部 module や `f64::` を外と数えないため)。
その上で領域を示す言葉と、どこでも使う道具(`glam` `serde`)を分ける — 道具は寄せ先を示さない。
**注意**: crate を名指ししていないことは「外の概念を実装していない」ことを意味しない。
外の規格(CSS・Lottie・OpenType)を依存なしで書いていても、この数には出ない。

    scripts/maps.sh          # 両方作る
    python3 scripts/map-responsibility.py [出力先.html]
"""
import sys

import json, os, re, collections, subprocess

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
CRATES = {
    "motolii-render": "motolii/crates/motolii-render/src",
    "motolii-doc":    "motolii/crates/motolii-doc/src",
    "motolii-ui":     "motolii/ui/native/src",
    "motolii-edit":   "motolii/ui/extensions/edit/src",
    "motolii-jobs":   "motolii/ui/extensions/jobs/src",
    "motolii-script": "motolii/ui/extensions/script/src",
}
# **外の定義。** 依存グラフに実在する package 名だけを「外」と認める
# (`property::` のような内部 module や `f64::` を外と数えないため)。
# その上で、領域を示す言葉と、どこでも使う道具を分ける — 道具は「寄せ先」を示さない。
import subprocess as _sp
_meta = json.loads(_sp.run(["cargo","metadata","--format-version","1","--filter-platform","aarch64-apple-darwin"],
                           cwd=ROOT, capture_output=True, text=True).stdout)
EXTERNAL = ({p["name"].replace("-","_") for p in _meta["packages"]}
             - {c.replace("-","_") for c in CRATES})
TOOL = r"^(glam|serde|serde_json|thiserror|itertools|tempfile|log|bytemuck|smallvec|indexmap|rand|regex|chrono|uuid|rayon|anyhow|once_cell|parking_lot|arrayvec|half|num_traits|ordered_float)$"
WORLD = [
    ("rerun",   r"^(re_[a-z0-9_]+|rerun)$"),
    ("GPU",     r"^(wgpu|naga|wesl|ash)$"),
    ("並べ/字", r"^(taffy|cosmic_text|swash|skrifa|harfrust|read_fonts|unicode_\w+|parley)$"),
    ("素材",    r"^(image|exr|zune_\w+|image_webp|tiff|symphonia|rubato|realfft|rustfft|moxcms|kamadak\w*)$"),
    ("物理",    r"^(rapier\w*|parry\w*|nalgebra)$"),
    ("OS",      r"^(objc2\w*|core_foundation|cpal|dispatch\w*)$"),
]
def world_of(c):
    if c not in EXTERNAL: return None          # 内部の module は外ではない
    for name, pat in WORLD:
        if re.match(pat, c): return name
    if re.match(TOOL, c): return "汎用の道具"
    return "その他の外部"

def unit_of(crate, rel):
    parts = rel.split(os.sep)
    if len(parts) == 1: return f"{crate}/{parts[0][:-3]}"
    # 2 段目まで必ず割る。1 段だと `engine` のような 8,000 行の塊が 1 単位になり、
    # 中で混ざった言葉のうち多い方だけが残って分類を誤る。
    second = parts[1][:-3] if parts[1].endswith(".rs") else parts[1]
    return f"{crate}/{parts[0]}/{second}"


def scan():

    units = collections.defaultdict(lambda: {"lines":0,"files":0,"big":0,"world":collections.Counter(),
                                             "path":"", "crate":""})
    texts = {}   # unit -> 連結した source
    for crate, rel in CRATES.items():
        base = os.path.join(ROOT, rel)
        for b, dirs, fs in os.walk(base):
            for f in fs:
                if not f.endswith(".rs"): continue
                p = os.path.join(b, f)
                r = os.path.relpath(p, base)
                u = unit_of(crate, r)
                src = open(p, errors="replace").read()
                n = src.count("\n") + 1
                d = units[u]
                d["lines"] += n; d["files"] += 1; d["big"] += n > 600
                d["crate"] = crate
                d["path"] = os.path.relpath(os.path.dirname(p) if os.sep in r else base, ROOT)
                texts.setdefault(u, []).append(src)
                for c in set(re.findall(r'\b([a-z][a-z0-9_]{2,})::', src)):
                    w = world_of(c)
                    if w: d["world"][c] += src.count(c + "::")
    # 内からの引き: その module の名前を、外の unit が何箇所書いているか
    joined = {u: "\n".join(v) for u, v in texts.items()}
    for u, d in units.items():
        segs = u.split("/")[1:]
        if segs[-1] in ("lib", "main"): d["fanin"] = 0; continue
        # `render::` のような一般名は誤爆するので、2 段あれば `親::子` で引く。
        tail = "::".join(segs[-2:]) if len(segs) > 1 else segs[-1]
        # 1 段の名前(`core` `store` `render`)は Rust 本体や別物と衝突するので、
        # 手前に `::` がある書き方(`doc::store::`)だけ数える。
        pat = (r'\b' + re.escape(tail) + r'::') if len(segs) > 1 else (r'(?<=::)' + re.escape(tail) + r'::')
        token = re.compile(pat)
        d["fanin"] = sum(len(token.findall(s)) for k, s in joined.items() if k != u)
        d["_tok"] = pat
    # 内側の辺: A の source が B の名前を書いていれば A → B。責任の絡み方そのもの。
    edges = []
    for a, sa in joined.items():
        for b, db in units.items():
            if a == b or "_tok" not in db: continue
            n = len(re.compile(db["_tok"]).findall(sa))
            if n: edges.append({"source": a, "target": b, "n": n})
    rows = []
    for u, d in units.items():
        if d["lines"] < 60: continue
        w = d["world"].most_common(6)
        top = collections.Counter()
        for c, n in d["world"].items(): top[world_of(c)] += n
        # 道具(glam・serde)はどこにでも出るので、領域の言葉に勝たせない。
        domain = collections.Counter({k: v for k, v in top.items()
                                      if k not in ("汎用の道具", None)})
        top = domain if domain else top
        rows.append({"unit": u, "crate": d["crate"], "path": d["path"],
                     "lines": d["lines"], "files": d["files"], "big": d["big"],
                     "fanin": d["fanin"], "speaks": [[c, n] for c, n in w],
                     "toward": top.most_common(1)[0][0] if top else None,
                     "toward_n": top.most_common(1)[0][1] if top else 0})
    rows.sort(key=lambda r: -r["lines"])
    return rows, edges

T = r'''<!DOCTYPE html><html lang="ja"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Motolii の責任の地図</title>
<script src="https://cdnjs.cloudflare.com/ajax/libs/d3/7.9.0/d3.min.js"></script>
<style>
:root{--bg:#141517;--fg:#e6e4e1;--dim:#8f8b86;--faint:#6a6762;--line:#2b2d31;--card:#1a1c1f;--warn:#e0a45e}
*{box-sizing:border-box}
body{margin:0;background:var(--bg);color:var(--fg);
 font:15px/1.75 -apple-system,"Hiragino Sans",sans-serif;padding:0 0 80px}
.page{max-width:920px;margin:0 auto;padding:0 24px}
header{padding:44px 0 8px}
h1{font-size:26px;font-weight:500;margin:0 0 10px}
.lede{color:var(--dim);margin:0 0 6px;max-width:64ch}
.stamp{color:var(--faint);font-size:12px;font-variant-numeric:tabular-nums}
h2{font-size:17px;font-weight:500;margin:44px 0 6px}
h2+.note{color:var(--dim);font-size:13px;margin:0 0 18px;max-width:68ch}
.big{display:flex;gap:34px;flex-wrap:wrap;margin:26px 0 10px}
.big b{font-size:30px;font-weight:500;font-variant-numeric:tabular-nums;display:block;line-height:1.15}
.big span{color:var(--dim);font-size:12px}
.stack{display:flex;height:34px;border-radius:6px;overflow:hidden;margin:6px 0 10px}
.stack i{display:block;height:100%}
.keys{display:flex;flex-wrap:wrap;gap:16px;font-size:13px;color:var(--dim)}
.keys b{color:var(--fg);font-weight:500}
.dot{width:9px;height:9px;border-radius:50%;display:inline-block;margin-right:6px}
.band{border:1px solid var(--line);border-radius:12px;padding:16px 18px;margin:14px 0;background:var(--card)}
.band h3{font-size:15px;font-weight:500;margin:0 0 2px;display:flex;align-items:center}
.band .what{color:var(--dim);font-size:13px;margin:0 0 14px;max-width:62ch}
.band .tot{margin-left:auto;font-size:13px;color:var(--dim);font-variant-numeric:tabular-nums}
.row{display:grid;grid-template-columns:230px 1fr 130px;gap:12px;align-items:center;padding:3px 0;font-size:13px}
.row .nm{white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.track{background:#222429;border-radius:3px;height:9px}
.track i{display:block;height:100%;border-radius:3px}
.side{font-size:11px;color:var(--dim);text-align:right;font-variant-numeric:tabular-nums}
.graphbox{position:relative;height:560px;border:1px solid var(--line);border-radius:12px;
 background:var(--card);overflow:hidden;margin-top:10px}
.graphbox svg{width:100%;height:100%;display:block;cursor:grab}
.gctl{position:absolute;top:10px;left:12px;display:flex;gap:14px;align-items:center;
 background:rgba(20,21,23,.82);padding:6px 10px;border-radius:8px;border:1px solid var(--line)}
.gctl button{background:none;border:1px solid var(--line);color:var(--dim);border-radius:6px;
 padding:3px 9px;font:inherit;font-size:12px;cursor:pointer}
.gpick{position:absolute;left:12px;right:12px;bottom:10px;font-size:12px;color:var(--dim);
 background:rgba(20,21,23,.88);padding:7px 11px;border-radius:8px;border:1px solid var(--line)}
.gpick b{color:var(--fg);font-weight:500}
table{width:100%;border-collapse:collapse;font-size:13px;margin-top:6px}
th{text-align:left;font-weight:500;color:var(--dim);font-size:12px;border-bottom:1px solid var(--line);
 padding:6px 8px 6px 0;cursor:pointer;user-select:none}
th.n,td.n{text-align:right;font-variant-numeric:tabular-nums}
td{padding:5px 8px 5px 0;border-bottom:1px solid #1f2125}
.tag{font-size:11px;padding:1px 7px;border-radius:99px;border:1px solid var(--line)}
ul.find{padding-left:0;list-style:none;margin:8px 0 0}
ul.find li{border-left:2px solid var(--line);padding:2px 0 14px 16px}
ul.find .n{font-variant-numeric:tabular-nums}
.how{color:var(--faint);font-size:12px;margin-top:40px;border-top:1px solid var(--line);padding-top:16px}
code{background:#0f1012;padding:1px 5px;border-radius:4px;font-size:12px}
</style></head><body><div class="page">
<header>
  <h1>Motolii の責任の地図</h1>
  <p class="lede">Motolii 自身の Rust を module ごとに割って、どれだけ重いか、互いにどう絡んでいるか、
     そして<b>すでに外のどの世界の言葉を喋っているか</b>を出したもの。
     喋っている言葉は「どこへ寄せられるか」の手がかりになる。</p>
  <div class="stamp" id="stamp"></div>
</header>
<div class="big" id="big"></div>
<div class="stack" id="stack"></div>
<div class="keys" id="keys"></div>

<h2>どの世界の言葉を喋っているか</h2>
<p class="note">外の crate の API をどれだけ名指ししているかで分けたもの。
  <b>「Motolii 固有」は外の言葉を喋っていない = 吸収できない</b> — そこが製品の中身。
  棒は行数、右は <span style="color:var(--dim)">出 → 入</span>(内部で何 module を呼び、何 module から呼ばれるか)。</p>
<div id="bands"></div>

<h2>絡み方</h2>
<p class="note">丸 = module(大きさは行数)、色 = 寄る先、横の位置 = 寄る先の列、
  白い縁 = 600 行を超えた file の数、線 = Motolii 内部の参照。掴んで動かせます。</p>
<div class="graphbox"><svg id="gsvg"></svg>
  <div class="gctl">
    <label style="display:flex;gap:7px;align-items:center;color:var(--dim);font-size:13px;cursor:pointer">
      <input type="checkbox" id="gBig"> 600行超を持つ module だけ</label>
    <button id="gFit">収める</button></div>
  <div class="gpick" id="gpick">丸を押すと説明が出ます</div>
</div>

<h2>全部の一覧</h2>
<table><thead><tr>
  <th data-k="unit">module</th><th data-k="toward">寄る先</th>
  <th data-k="lines" class="n">行</th><th data-k="files" class="n">file</th>
  <th data-k="big" class="n">600超</th><th data-k="out" class="n">出</th><th data-k="in" class="n">入</th>
  <th>喋っている言葉</th></tr></thead><tbody id="tb"></tbody></table>

<h2>ここから読み取れること</h2>
<ul class="find" id="find"></ul>
<div class="how">
  数の作り方 — 単位は crate + 先頭 2 段の module。行は <code>.rs</code> の実数。
  内部の辺は、ある module の source が別の module の名前(<code>親::子::</code>)を書いていれば 1 本。
  1 段しかない名前(<code>core</code> <code>store</code>)は Rust 本体と衝突するので、
  手前に <code>::</code> がある書き方だけ数えている。<br>
  寄る先は、その module が名指ししている外部 crate を世界ごとにまとめ、一番多い世界。
  <code>glam</code> <code>serde</code> のような全部に出る道具は印にしていない。<br>
  作り直し: <code>python3 resp.py && python3 resp_build.py</code>
</div></div>
<script>
const D = __DATA__;
const W = [
  ["rerun","#7fc08a","Rerun の部品の言葉を喋っている"],
  ["GPU","#6fb3d2","wgpu / naga — Rerun の下の層"],
  ["並べ/字","#c98a6a","taffy・cosmic-text — 並べと文字組み"],
  ["素材","#b98fc0","画像・動画・音の decode"],
  ["物理","#9ec27f","rapier・parry"],
  ["OS","#c9a86a","device・objc2"],
  ["その他の外部","#a0a8b8","上のどれでもない外部 crate"],
  ["汎用の道具","#7d8087","glam・serde — どこでも使う物しか喋っていない"],
  ["Motolii 固有","#8f8b86","外部 crate を 1 つも名指ししていない"],
];
const C = Object.fromEntries(W.map(([k,c])=>[k,c]));
const sum = {}; D.rows.forEach(r => sum[r.toward] = (sum[r.toward]||0) + r.lines);
const pct = n => (n*100/D.total).toFixed(1);
document.getElementById('stamp').textContent =
  `${D.head} · ${D.generated} · ${D.total.toLocaleString()} 行 · ${D.rows.length} module · 内部の辺 ${D.edges.length}`;
const own = sum['Motolii 固有']||0;
const byCrate = {}; D.rows.forEach(r => byCrate[r.crate]=(byCrate[r.crate]||0)+r.lines);
const biggest = Object.entries(byCrate).sort((a,b)=>b[1]-a[1])[0];
document.getElementById('big').innerHTML = `
 <div><b>${pct(biggest[1])}%</b><span>${biggest[0]} 1 つで</span></div>
 <div><b>${pct(own)}%</b><span>外の言葉を喋っていない = 固有</span></div>
 <div><b>${D.rows.reduce((a,r)=>a+r.big,0)}</b><span>600 行を超えた file</span></div>
 <div><b>${D.edges.length}</b><span>module 同士の参照</span></div>`;
document.getElementById('stack').innerHTML = W.map(([k,c]) =>
  sum[k] ? `<i style="width:${sum[k]*100/D.total}%;background:${c}" title="${k}"></i>`:'').join('');
document.getElementById('keys').innerHTML = W.filter(([k])=>sum[k]).map(([k,c,desc]) =>
  `<span><span class="dot" style="background:${c}"></span><b>${k} ${pct(sum[k])}%</b> ${desc}</span>`).join('');

const maxL = Math.max(...D.rows.map(r=>r.lines));
document.getElementById('bands').innerHTML = W.filter(([k])=>sum[k]).map(([k,c,desc])=>{
  const list = D.rows.filter(r=>r.toward===k);
  return `<div class="band"><h3><span class="dot" style="background:${c}"></span>${k}
    <span class="tot">${sum[k].toLocaleString()} 行 · ${list.length} module</span></h3>
    <p class="what">${desc}</p>
    ${list.slice(0,10).map(r=>`<div class="row">
      <span class="nm">${r.unit}</span>
      <span class="track"><i style="width:${Math.max(1.2,r.lines*100/maxL)}%;background:${c}"></i></span>
      <span class="side">${r.lines.toLocaleString()} 行 · ${r.out}→${r.in}${r.big?` · <span style="color:var(--warn)">600超 ${r.big}</span>`:''}</span>
    </div>`).join('')}
    ${list.length>10?`<div class="what" style="margin:8px 0 0">ほか ${list.length-10} 個</div>`:''}</div>`;
}).join('');

let sortK='lines', desc=true;
function draw(){
  const rows=[...D.rows].sort((a,b)=>{const x=a[sortK],y=b[sortK];
    const c=typeof x==='string'?x.localeCompare(y):x-y; return desc?-c:c;});
  document.getElementById('tb').innerHTML = rows.map(r=>`<tr>
    <td>${r.unit}</td>
    <td><span class="tag" style="border-color:${(C[r.toward]||'#8f8b86')}55;color:${C[r.toward]||'#8f8b86'}">${r.toward}</span></td>
    <td class="n">${r.lines.toLocaleString()}</td><td class="n">${r.files}</td>
    <td class="n" ${r.big?'style="color:var(--warn)"':''}>${r.big||'—'}</td>
    <td class="n">${r.out}</td><td class="n">${r.in}</td>
    <td style="color:var(--dim)">${r.speaks.slice(0,3).map(([c,n])=>c+'×'+n).join(' ')||'—'}</td></tr>`).join('');
}
document.querySelectorAll('th[data-k]').forEach(th=>th.onclick=()=>{
  const k=th.dataset.k; desc = k===sortK ? !desc : true; sortK=k; draw();});
draw();

const X = Object.fromEntries(W.map(([k],i)=>[k, -860 + i*215]));
const xOf = d => X[d.toward] ?? 0;   // 一覧に無い分類でも NaN にしない
const nodes = D.rows.map(r=>({...r}));
const links = D.edges.map(e=>({source:e.source,target:e.target,n:e.n}));
const gr = n => 4 + Math.sqrt(n.lines)/7;
const svg = d3.select('#gsvg'), g = svg.append('g');
const zoom = d3.zoom().scaleExtent([0.15,5]).on('zoom',e=>g.attr('transform',e.transform));
svg.call(zoom).on('dblclick.zoom', null);
const link = g.append('g').attr('stroke','#24262a').attr('stroke-opacity',.6)
  .selectAll('line').data(links).join('line').attr('stroke-width',d=>Math.min(2,0.4+Math.log2(1+d.n)*0.25));
const node = g.append('g').selectAll('circle').data(nodes).join('circle')
  .attr('r',gr).attr('fill',d=>C[d.toward]||'#8f8b86').attr('fill-opacity',.9)
  .attr('stroke','#e6e4e1').attr('stroke-width',d=>d.big?Math.min(4,d.big*1.1):0)
  .style('cursor','pointer')
  .call(d3.drag()
    .on('start',(e,d)=>{if(!e.active)sim.alphaTarget(.25).restart();d.fx=d.x;d.fy=d.y;})
    .on('drag',(e,d)=>{d.fx=e.x;d.fy=e.y;})
    .on('end',(e,d)=>{if(!e.active)sim.alphaTarget(0);d.fx=null;d.fy=null;}))
  .on('click',(e,d)=>pick(d));
const label = g.append('g').selectAll('text').data(nodes).join('text')
  .text(d=>d.lines>700?d.unit.split('/').slice(1).join('/'):'')
  .attr('font-size',11).attr('fill','#9a9691').attr('text-anchor','middle')
  .attr('dy',d=>-gr(d)-3).style('pointer-events','none');
const sim = d3.forceSimulation(nodes)
  .force('link',d3.forceLink(links).id(d=>d.unit).distance(70).strength(.18))
  .force('charge',d3.forceManyBody().strength(-230).distanceMax(560))
  .force('collide',d3.forceCollide().radius(d=>gr(d)+8))
  .force('x',d3.forceX(xOf).strength(.5))
  .force('y',d3.forceY(0).strength(.12))
  .on('tick',()=>{link.attr('x1',d=>d.source.x).attr('y1',d=>d.source.y)
      .attr('x2',d=>d.target.x).attr('y2',d=>d.target.y);
    node.attr('cx',d=>d.x).attr('cy',d=>d.y); label.attr('x',d=>d.x).attr('y',d=>d.y);});
function fit(){
  const b=document.querySelector('.graphbox').getBoundingClientRect();
  const vis=nodes.filter(n=>isFinite(n.x));
  if(vis.length<3) return;
  const q=(a,p)=>{const z=[...a].sort((x,y)=>x-y);return z[Math.floor((z.length-1)*p)];};
  const xs=vis.map(n=>n.x),ys=vis.map(n=>n.y),pad=46;
  const x0=q(xs,.02),x1=q(xs,.98),y0=q(ys,.02),y1=q(ys,.98);
  const k=Math.max(.25,Math.min((b.width-pad*2)/Math.max(1,x1-x0),(b.height-pad*2)/Math.max(1,y1-y0),1.8));
  svg.transition().duration(420).call(zoom.transform,
    d3.zoomIdentity.translate(b.width/2,b.height/2).scale(k).translate(-(x0+x1)/2,-(y0+y1)/2));
}
sim.on('end',fit); setTimeout(fit,1700);
document.getElementById('gFit').onclick=fit;
document.getElementById('gBig').addEventListener('change',e=>{
  const only=e.target.checked, hid=d=>only&&!d.big;
  node.attr('display',d=>hid(d)?'none':null);
  label.attr('display',d=>hid(d)?'none':null);
  link.attr('display',l=>hid(l.source)||hid(l.target)?'none':null);
  setTimeout(fit,60);
});
function pick(d){
  document.getElementById('gpick').innerHTML =
   `<b>${d.unit}</b> · ${d.lines.toLocaleString()} 行 / ${d.files} file`
   + (d.big?` · <span style="color:var(--warn)">600 行超 ${d.big} 本</span>`:'')
   + ` · 内部で ${d.out} module を呼び、${d.in} module から呼ばれる · 寄る先 <b>${d.toward}</b>`
   + (d.speaks.length?` — ${d.speaks.slice(0,4).map(([c,n])=>c+'×'+n).join(' / ')}`:'');
}

const detach = D.rows.filter(r=>r.in<=3 && r.lines>600).sort((a,b)=>b.lines-a.lines);
const heavy = D.rows.filter(r=>r.big>0).sort((a,b)=>b.big-a.big);
document.getElementById('find').innerHTML = `
<li><b>${biggest[0]} 1 つで <span class="n">${pct(biggest[1])}%</span>。</b>
  家は 5 つと宣言しているが、実体は 1 つの crate に寄っている。</li>
<li><b>外の言葉を喋っていない module が <span class="n">${pct(own)}%</span>。</b>
  ここは rerun にも Flutter にも吸収できないし、してはいけない — <b>Motolii そのもの</b>。</li>
<li><b>外から呼ばれるのが 3 module 以下で、600 行を超える塊:</b>
  ${detach.slice(0,5).map(r=>`<span class="n">${r.unit}</span>（${r.lines.toLocaleString()} 行・${r.toward}）`).join('、')}。
  絡みが薄いので、出すなら最初に触れる。</li>
<li><b>600 行を超えた file が <span class="n">${D.rows.reduce((a,r)=>a+r.big,0)}</span> 本。</b>
  多い順に ${heavy.slice(0,4).map(r=>`${r.unit}（${r.big}）`).join('、')}。</li>`;
</script></body></html>
'''


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else f"{ROOT}/motolii/target/maps/responsibility.html"
    os.makedirs(os.path.dirname(out), exist_ok=True)
    rows, edges = scan()
    names = {r["unit"] for r in rows}
    edges = [e for e in edges if e["source"] in names and e["target"] in names]
    deg_out = collections.Counter(e["source"] for e in edges)
    deg_in = collections.Counter(e["target"] for e in edges)
    for r in rows:
        r["out"] = deg_out[r["unit"]]
        r["in"] = deg_in[r["unit"]]
        r["toward"] = r["toward"] or "Motolii 固有"
    payload = {"rows": rows, "edges": edges,
               "head": subprocess.run(["git","-C",ROOT,"rev-parse","--short","HEAD"],capture_output=True,text=True).stdout.strip(),
               "generated": subprocess.run(["date","+%Y-%m-%d %H:%M"],capture_output=True,text=True).stdout.strip(),
               "total": sum(r["lines"] for r in rows)}
    open(out, "w").write(T.replace("__DATA__", json.dumps(payload, ensure_ascii=False)))
    print(f'{len(rows)} module / {len(edges)} 辺 / {payload["total"]} 行 → {out}')


main()
