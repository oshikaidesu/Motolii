#!/usr/bin/env python3
"""依存の地図 — 出来上がった dylib を分解して、どこから来た code がどれだけ載っているか、
そのうち Motolii が自分で名前を書いて使っているのはどれかを 1 枚の HTML にする。

大きさは `llvm-nm` で text を番地順に並べ、隣との差を crate に付ける
(`cargo-bloat` が中でやっているのと同じ原理。Motolii が出すのは cdylib なので直接は使えない)。
位相は `cargo metadata` の**一次の依存だけ** — rerun の crate は速度を担保したラッパーなので、
その内側まで直結して描くと構造を誤って見せる。

    scripts/maps.sh          # 両方作る
    python3 scripts/map-dependencies.py [出力先.html]
"""
import sys

import json, os, re, subprocess, sys, collections

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
DYLIB = f"{ROOT}/motolii/target/debug/libmotolii_ui.dylib"

# --- 載っている量 -----------------------------------------------------------
LEGACY = re.compile(r'^_{1,2}ZN(\d+)([A-Za-z0-9_$.]+)')
V0 = re.compile(r'^_{1,2}R[A-Za-z]*Cs[0-9A-Za-z]+_(\d+)([A-Za-z0-9_]+)')
INNER = re.compile(r'\$LT\$([A-Za-z0-9_]+)')

def crate_of(name):
    m = V0.match(name)
    if m:
        return m.group(2)[: int(m.group(1))]
    m = LEGACY.match(name)
    if m:
        head = m.group(2)[: int(m.group(1))]
        if head.startswith("_$LT$") or head.startswith("$LT$"):
            inner = INNER.search(head)          # `<alloc..vec..Vec<T> as …>` は alloc の物
            return inner.group(1) if inner else "(不明)"
        return head
    return "(rust 以外 / C・asm)"

def bytes_by_crate():
    out = subprocess.run(["xcrun", "llvm-nm", "-n", "--defined-only", DYLIB],
                         capture_output=True, text=True).stdout
    rows = []
    for line in out.splitlines():
        p = line.split(None, 2)
        if len(p) < 3 or p[1] not in ("t", "T"):
            continue
        try:
            rows.append((int(p[0], 16), p[2].strip()))
        except ValueError:
            pass
    rows.sort()
    size = collections.Counter()
    for i in range(len(rows) - 1):
        n = rows[i + 1][0] - rows[i][0]
        if 0 < n <= 4_000_000:
            size[crate_of(rows[i][1])] += n
    return size

# --- 使っている量 -----------------------------------------------------------
SKIP = {"target", ".git", "build", ".dart_tool", "Pods", "DerivedData"}

def sources():
    for base, dirs, files in os.walk(f"{ROOT}/motolii"):
        dirs[:] = [d for d in dirs if d not in SKIP]
        for f in files:
            if f.endswith(".rs"):
                yield os.path.join(base, f)

def usage(crates):
    known = {c for c in crates if len(c) > 2}
    path = re.compile(r'\b([a-z][a-z0-9_]{2,})::((?:[A-Za-z0-9_]+)(?:::[A-Za-z0-9_]+)?)')
    sites = collections.Counter()
    items = collections.defaultdict(set)
    files = collections.defaultdict(set)
    for f in sources():
        try:
            text = open(f, errors="replace").read()
        except OSError:
            continue
        for crate, rest in path.findall(text):
            if crate in known:
                sites[crate] += 1
                items[crate].add(rest)
                files[crate].add(f)
    return sites, items, files

def family(name):
    if name.startswith("motolii"):
        return "自前"
    if name.startswith("re_") or name == "rerun":
        return "rerun"
    if name in ("core", "alloc", "std") or name.startswith("(rust"):
        return "Rust 本体"
    return "他の借り物"


TEMPLATE = r'''<!DOCTYPE html>
<html lang="ja"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Motolii は何でできているか</title>
<script src="https://cdnjs.cloudflare.com/ajax/libs/d3/7.9.0/d3.min.js"></script>
<style>
:root{--bg:#141517;--fg:#e6e4e1;--dim:#8f8b86;--faint:#6a6762;--line:#2b2d31;--card:#1a1c1f;
 --mine:#6fb3d2;--rerun:#7fc08a;--borrow:#c98a6a;--rust:#6b6e74;--warn:#e0a45e}
*{box-sizing:border-box}
body{margin:0;background:var(--bg);color:var(--fg);
 font:15px/1.75 -apple-system,"Hiragino Sans",sans-serif;padding:0 0 80px}
.page{max-width:900px;margin:0 auto;padding:0 24px}
header{padding:44px 0 8px}
h1{font-size:26px;font-weight:500;margin:0 0 10px;letter-spacing:.01em}
.lede{color:var(--dim);font-size:15px;margin:0 0 6px;max-width:64ch}
.stamp{color:var(--faint);font-size:12px;font-variant-numeric:tabular-nums}
h2{font-size:17px;font-weight:500;margin:44px 0 6px}
h2+.note{color:var(--dim);font-size:13px;margin:0 0 18px;max-width:66ch}
.big{display:flex;gap:34px;flex-wrap:wrap;margin:26px 0 8px}
.big div{display:flex;flex-direction:column}
.big b{font-size:30px;font-weight:500;line-height:1.15;font-variant-numeric:tabular-nums}
.big span{color:var(--dim);font-size:12px}
.stack{display:flex;height:34px;border-radius:6px;overflow:hidden;margin:6px 0 10px}
.stack i{display:block;height:100%}
.keys{display:flex;flex-wrap:wrap;gap:18px;font-size:13px;color:var(--dim)}
.keys b{color:var(--fg);font-weight:500}
.dot{width:9px;height:9px;border-radius:50%;display:inline-block;margin-right:6px}
.band{border:1px solid var(--line);border-radius:12px;padding:16px 18px;margin:14px 0;background:var(--card)}
.band h3{font-size:15px;font-weight:500;margin:0 0 2px;display:flex;align-items:center}
.band .what{color:var(--dim);font-size:13px;margin:0 0 14px}
.band .tot{margin-left:auto;font-size:13px;color:var(--dim);font-variant-numeric:tabular-nums}
.row{display:grid;grid-template-columns:150px 1fr 74px;gap:12px;align-items:center;
 padding:3px 0;font-size:13px}
.row .nm{white-space:nowrap;overflow:hidden;text-overflow:ellipsis}
.row .nm.un{color:var(--faint)}
.track{background:#222429;border-radius:3px;height:9px;position:relative}
.track i{display:block;height:100%;border-radius:3px}
.use{font-size:11px;color:var(--dim);text-align:right;font-variant-numeric:tabular-nums}
.use.un{color:var(--warn)}
table{width:100%;border-collapse:collapse;font-size:13px;margin-top:6px}
th{text-align:left;font-weight:500;color:var(--dim);font-size:12px;
 border-bottom:1px solid var(--line);padding:6px 8px 6px 0;cursor:pointer;user-select:none}
th.n,td.n{text-align:right;font-variant-numeric:tabular-nums}
td{padding:5px 8px 5px 0;border-bottom:1px solid #1f2125}
tr.un td:first-child{color:var(--faint)}
.tag{font-size:11px;padding:1px 7px;border-radius:99px;border:1px solid var(--line);color:var(--dim)}
.ctl{display:flex;gap:10px;align-items:center;margin:14px 0 0;flex-wrap:wrap}
input[type=search]{flex:1;min-width:160px;background:#0f1012;border:1px solid var(--line);
 color:var(--fg);padding:7px 10px;border-radius:7px;font:inherit;font-size:13px}
label.chk{display:flex;gap:7px;align-items:center;color:var(--dim);font-size:13px;cursor:pointer}
ul.find{padding-left:0;list-style:none;margin:8px 0 0}
ul.find li{border-left:2px solid var(--line);padding:2px 0 14px 16px;margin:0}
ul.find b{font-weight:500}
ul.find .n{font-variant-numeric:tabular-nums;color:var(--fg)}
.graphbox{position:relative;height:540px;border:1px solid var(--line);border-radius:12px;
 background:var(--card);overflow:hidden;margin-top:10px}
.graphbox svg{width:100%;height:100%;display:block;cursor:grab}
.gctl{position:absolute;top:10px;left:12px;display:flex;gap:14px;align-items:center;
 background:rgba(20,21,23,.8);padding:6px 10px;border-radius:8px;border:1px solid var(--line)}
.gctl button{background:none;border:1px solid var(--line);color:var(--dim);
 border-radius:6px;padding:3px 9px;font:inherit;font-size:12px;cursor:pointer}
.gpick{position:absolute;left:12px;bottom:10px;right:12px;font-size:12px;color:var(--dim);
 background:rgba(20,21,23,.86);padding:7px 11px;border-radius:8px;border:1px solid var(--line)}
.gpick b{color:var(--fg);font-weight:500}
.how{color:var(--faint);font-size:12px;margin-top:40px;border-top:1px solid var(--line);padding-top:16px}
code{background:#0f1012;padding:1px 5px;border-radius:4px;font-size:12px}
</style></head><body><div class="page">

<header>
  <h1>Motolii は何でできているか</h1>
  <p class="lede">出来上がった 1 つの dylib を分解して、どこから来た code がどれだけ載っているか、
     そのうち Motolii が自分で名前を書いて使っているのはどれかを出したもの。</p>
  <div class="stamp" id="stamp"></div>
</header>

<div class="big" id="big"></div>
<div class="stack" id="stack"></div>
<div class="keys" id="keys"></div>

<h2>4 つの層</h2>
<p class="note">左の層ほど Motolii に近い。各行の棒は dylib に載っている大きさ、
   右の数は <b>Motolii が名前を書いて使っている API の種類</b>。
   <span style="color:var(--warn)">0</span> は一度も名指ししていない = その上の層が連れてきた物。
   自前の層は自分の code を <code>crate::</code> で書くので、この数では測れない(—)。</p>
<div id="bands"></div>

<h2>ひとつの絵で</h2>
<p class="note">同じ数字を 1 枚に重ねたもの。<b>丸の大きさ</b> = dylib に載っている量、
   <b>白い縁の太さ</b> = Motolii が使っている API の種類、<b>横の位置</b> = 層、
   <b>線</b> = 一次の依存だけ(ラッパーの内側は繋がない)。掴んで動かせます。</p>
<div class="graphbox"><svg id="gsvg"></svg>
  <div class="gctl">
    <label class="chk"><input type="checkbox" id="gRust" checked> Rust 本体を隠す</label>
    <label class="chk"><input type="checkbox" id="gBehind"> 背後の重さを足す</label>
    <button id="gFit">収める</button>
  </div>
  <div class="gpick" id="gpick">丸を押すと説明が出ます</div>
</div>

<h2>全部の一覧</h2>
<p class="note">見出しを押すと並べ替わります。</p>
<div class="ctl">
  <input type="search" id="q" placeholder="crate を探す">
  <label class="chk"><input type="checkbox" id="onlyUn"> 名指ししていない物だけ</label>
  <label class="chk"><input type="checkbox" id="hideRust" checked> Rust 本体を隠す</label>
</div>
<table><thead><tr>
  <th data-k="crate">crate</th><th data-k="family">層</th>
  <th data-k="bytes" class="n">大きさ</th><th data-k="behind" class="n">背後</th>
  <th data-k="items" class="n">使った種類</th><th data-k="sites" class="n">名指し</th>
  <th data-k="parents">引き込んでいるのは</th>
</tr></thead><tbody id="tb"></tbody></table>

<h2>ここから読み取れること</h2>
<ul class="find" id="find"></ul>

<div class="how">
  数の作り方 — 大きさ: <code>llvm-nm</code> で dylib の text を番地順に並べ、隣との差を crate に付ける
  (<code>cargo-bloat</code> が中でやっているのと同じ。Motolii が出すのは cdylib なので直接は使えない)。
  位相: <code>cargo metadata</code> の一次の依存だけ。使った種類: Motolii の <code>.rs</code> が書いている
  <code>crate::…</code> の異なり数。<br>
  これは <b>debug build</b> の値。release + LTO では全体も比率も動く。<br>
  作り直し: <code>python3 collect.py data.json && python3 build.py</code>
</div>
</div>
<script>
const D = __DATA__;
const FAM = [
  ["自前","--mine","Motolii 自身が書いた code"],
  ["rerun","--rerun","Rerun の部品。速度を担保したラッパー"],
  ["他の借り物","--borrow","その内側と、Motolii が別に引いた物"],
  ["Rust 本体","--rust","core / alloc / std"],
];
const col = f => getComputedStyle(document.documentElement)
  .getPropertyValue(FAM.find(x=>x[0]===f)[1]).trim();
// 自前の crate は自分の code を `crate::` で書くので、名指しの数では測れない。
const unused = n => !n.items && n.family !== '自前';
const mib = b => (b/1048576).toFixed(2);
const kib = b => b>=1048576 ? (b/1048576).toFixed(2)+' MiB' : (b/1024).toFixed(0)+' KiB';

document.getElementById('stamp').textContent =
  `${D.head} · ${D.generated} · dylib ${mib(D.total)} MiB · crate ${D.nodes.length} · package ${D.facts.packages}`;

const sum = {}; D.nodes.forEach(n => sum[n.family] = (sum[n.family]||0) + n.bytes);
document.getElementById('big').innerHTML = `
  <div><b>${(sum['自前']*100/D.total).toFixed(1)}%</b><span>自前</span></div>
  <div><b>${(D.facts.rerun_pkgs)}</b><span>rerun とその下流の package</span></div>
  <div><b>${D.facts.direct}</b><span>motolii-ui の直接依存</span></div>
  <div><b>${D.facts.duplicates}</b><span>同じ crate の版が重複</span></div>`;
document.getElementById('stack').innerHTML = FAM.map(([f]) =>
  `<i style="width:${sum[f]*100/D.total}%;background:${col(f)}" title="${f}"></i>`).join('');
document.getElementById('keys').innerHTML = FAM.map(([f,,desc]) =>
  `<span><span class="dot" style="background:${col(f)}"></span><b>${f} ${(sum[f]*100/D.total).toFixed(1)}%</b> ${desc}</span>`).join('');

const maxB = Math.max(...D.nodes.map(n=>n.bytes));
document.getElementById('bands').innerHTML = FAM.map(([f,,desc]) => {
  const list = D.nodes.filter(n=>n.family===f).slice(0,12);
  const all = D.nodes.filter(n=>n.family===f);
  return `<div class="band"><h3><span class="dot" style="background:${col(f)}"></span>${f}
    <span class="tot">${mib(sum[f])} MiB · ${all.length} crate</span></h3>
    <p class="what">${desc}</p>
    ${list.map(n=>`<div class="row">
      <span class="nm${unused(n)?' un':''}">${n.crate}</span>
      <span class="track"><i style="width:${Math.max(1.2,n.bytes*100/maxB)}%;background:${col(f)}"></i></span>
      <span class="use${unused(n)?' un':''}">${n.items ? n.items+' 種類' : (n.family==='自前' ? '—' : '0')}</span></div>`).join('')}
    ${all.length>12 ? `<div class="what" style="margin:8px 0 0">ほか ${all.length-12} 個</div>`:''}</div>`;
}).join('');

let sortK='bytes', desc=true;
function draw(){
  const q=document.getElementById('q').value.trim().toLowerCase();
  const un=document.getElementById('onlyUn').checked;
  const hr=document.getElementById('hideRust').checked;
  let rows=D.nodes.filter(n=>(!q||n.crate.toLowerCase().includes(q))
    && (!un||unused(n)) && (!hr||n.family!=='Rust 本体'));
  rows.sort((a,b)=>{const x=a[sortK],y=b[sortK];
    const c = typeof x==='string' ? x.localeCompare(y) : x-y; return desc?-c:c;});
  document.getElementById('tb').innerHTML = rows.map(n=>`<tr class="${unused(n)?'un':''}">
    <td>${n.crate}</td>
    <td><span class="tag" style="border-color:${col(n.family)}55;color:${col(n.family)}">${n.family}</span></td>
    <td class="n">${kib(n.bytes)}</td>
    <td class="n">${n.behind?kib(n.behind):'—'}</td>
    <td class="n" ${unused(n)?'style="color:var(--warn)"':''}>${n.items || (n.family==='自前'?'—':0)}</td>
    <td class="n">${n.sites||'—'}</td>
    <td style="color:var(--dim)">${n.parents.slice(0,3).join(' · ')||'—'}</td></tr>`).join('');
}
document.querySelectorAll('th[data-k]').forEach(th=>th.onclick=()=>{
  const k=th.dataset.k; desc = k===sortK ? !desc : true; sortK=k; draw();});
['q','onlyUn','hideRust'].forEach(id=>document.getElementById(id).addEventListener('input',draw));
draw();

const held = D.nodes.filter(n=>unused(n) && n.family!=='Rust 本体')
  .reduce((a,n)=>a+n.bytes,0);
const arrow = D.nodes.filter(n=>n.crate.startsWith('arrow')).reduce((a,n)=>a+n.bytes,0);
document.getElementById('find').innerHTML = `
<li><b>自前は <span class="n">${(sum['自前']*100/D.total).toFixed(1)}%</span> しかない。</b>
  残りは借り物で、そのうち <span class="n">${D.facts.rerun_pkgs}</span> package が rerun とその下流。
  Motolii は「Rerun を AE にする」ソフトなので、これは狙い通りの姿。</li>
<li><b>切れる枝がほとんど無い。</b> 直接依存は <span class="n">${D.facts.direct}</span> 本だけで、
  1 本消して実際に消える重さはほぼ全部ゼロ。重い物は例外なく複数の道から来ている
  — 絡まっているのではなく共有されている。</li>
<li><b>名指ししていない借り物が <span class="n">${mib(held)} MiB</span>。</b>
  無駄という意味ではなく、借りている物の下敷き。ただし「何にいくら払っているか」はここで見える。</li>
<li><b>Arrow だけで <span class="n">${mib(arrow)} MiB</span> 払っている。</b>
  列指向の store の値段を払いながら、書類はまだ JSON の文字列で持っている。
  <b>使い切れていないのはここ</b>。</li>
<li><b>同じ crate の版が <span class="n">${D.facts.duplicates}</span> 組で重複。</b>
  避けられる複雑さとしては、いちばん素直に減らせる場所。</li>`;

/* ---- ひとつの絵 ---- */
const X = {"自前":-560,"rerun":-190,"他の借り物":250,"Rust 本体":640};
let gBehind = false;
const gr = n => 4.5 + Math.sqrt(n.bytes + (gBehind ? n.behind : 0)) / 62;
const ring = n => n.items ? Math.min(4, 0.8 + Math.log2(1 + n.items) * 0.55) : 0;
const gnodes = D.nodes.map(n => ({...n}));
const byName = Object.fromEntries(gnodes.map(n => [n.crate, n]));
const glinks = D.links.filter(l => byName[l.source] && byName[l.target])
                      .map(l => ({source: l.source, target: l.target}));
const gsvg = d3.select('#gsvg'), gg = gsvg.append('g');
const gzoom = d3.zoom().scaleExtent([0.15, 5]).on('zoom', e => gg.attr('transform', e.transform));
gsvg.call(gzoom).on('dblclick.zoom', null);

const glink = gg.append('g').attr('stroke','#24262a').attr('stroke-width',0.5)
  .attr('stroke-opacity',0.6).selectAll('line').data(glinks).join('line');
const gnode = gg.append('g').selectAll('circle').data(gnodes).join('circle')
  .attr('r', gr).attr('fill', d => col(d.family))
  .attr('fill-opacity', d => unused(d) ? 0.32 : 0.92)
  .attr('stroke','#e6e4e1').attr('stroke-width', ring).style('cursor','pointer')
  .call(d3.drag()
    .on('start',(e,d)=>{if(!e.active)gsim.alphaTarget(0.25).restart();d.fx=d.x;d.fy=d.y;})
    .on('drag', (e,d)=>{d.fx=e.x;d.fy=e.y;})
    .on('end',  (e,d)=>{if(!e.active)gsim.alphaTarget(0);d.fx=null;d.fy=null;}))
  .on('click', (e,d) => pick(d));
const glabel = gg.append('g').selectAll('text').data(gnodes).join('text')
  .text(d => d.bytes > 60000 ? d.crate : '')
  .attr('font-size', d => Math.max(10, Math.min(15, gr(d)*0.62)))
  .attr('fill','#9a9691').attr('text-anchor','middle').attr('dy', d => -gr(d)-3)
  .style('pointer-events','none');

const gsim = d3.forceSimulation(gnodes)
  .force('link', d3.forceLink(glinks).id(d=>d.crate).distance(58).strength(0.22))
  .force('charge', d3.forceManyBody().strength(-180).distanceMax(520))
  .force('collide', d3.forceCollide().radius(d => gr(d)+7))
  .force('x', d3.forceX(d => X[d.family]).strength(0.55))
  .force('y', d3.forceY(0).strength(0.14))
  .on('tick', () => {
    glink.attr('x1',d=>d.source.x).attr('y1',d=>d.source.y)
         .attr('x2',d=>d.target.x).attr('y2',d=>d.target.y);
    gnode.attr('cx',d=>d.x).attr('cy',d=>d.y);
    glabel.attr('x',d=>d.x).attr('y',d=>d.y);
  });

function gfit(){
  const b = document.querySelector('.graphbox').getBoundingClientRect();
  const vis = gnodes.filter(n => isFinite(n.x) && (!gHideRust() || n.family!=='Rust 本体'));
  if (vis.length < 4) return;
  const q=(a,p)=>{const z=[...a].sort((x,y)=>x-y);return z[Math.floor((z.length-1)*p)];};
  const xs=vis.map(n=>n.x), ys=vis.map(n=>n.y);
  const x0=q(xs,.02),x1=q(xs,.98),y0=q(ys,.02),y1=q(ys,.98), pad=44;
  const k=Math.max(0.25, Math.min((b.width-pad*2)/Math.max(1,x1-x0),
                                  (b.height-pad*2)/Math.max(1,y1-y0), 1.9));
  gsvg.transition().duration(420).call(gzoom.transform,
    d3.zoomIdentity.translate(b.width/2,b.height/2).scale(k).translate(-(x0+x1)/2,-(y0+y1)/2));
}
const gHideRust = () => document.getElementById('gRust').checked;
gsim.on('end', gfit); setTimeout(gfit, 1700);

function pick(d){
  document.getElementById('gpick').innerHTML =
    `<b>${d.crate}</b> · ${d.family} · ${kib(d.bytes)}（${d.share.toFixed(1)}%）`
    + (d.behind ? ` · 背後 ${kib(d.behind)}` : '')
    + ` · ${d.family==='自前' ? '自分の code' :
           d.items ? `API を <b>${d.items} 種類</b>・${d.sites} 箇所で使用`
                   : '<span style="color:var(--warn)">一度も名指ししていない</span>'}`
    + (d.parents.length ? ` · 引き込んでいるのは ${d.parents.slice(0,3).join(' · ')}` : '');
}
function gApplyRust(){
  const off = gHideRust();
  const hid = d => off && d.family === 'Rust 本体';
  gnode.attr('display', d => hid(d) ? 'none' : null);
  glabel.attr('display', d => hid(d) ? 'none' : null);
  glink.attr('display', l => hid(l.source)||hid(l.target) ? 'none' : null);
  setTimeout(gfit, 60);
}
document.getElementById('gRust').addEventListener('change', gApplyRust);
document.getElementById('gBehind').addEventListener('change', e => {
  gBehind = e.target.checked;
  gnode.attr('r', gr); glabel.attr('dy', d=>-gr(d)-3);
  gsim.force('collide', d3.forceCollide().radius(d=>gr(d)+7)).alpha(0.4).restart();
});
document.getElementById('gFit').addEventListener('click', gfit);
gApplyRust();
</script></body></html>
'''


# --- 位相と、見出しの数 ------------------------------------------------------
def graph():
    meta = json.loads(subprocess.run(
        ["cargo","metadata","--format-version","1","--filter-platform","aarch64-apple-darwin"],
        cwd=ROOT, capture_output=True, text=True).stdout)
    name = {p["id"]: p["name"] for p in meta["packages"]}
    res = {n["id"]: n for n in meta["resolve"]["nodes"]}
    def deps(pid):
        out = []
        for d in res.get(pid, {}).get("deps", []):
            kinds = {k.get("kind") for k in d.get("dep_kinds", [{}])}
            if kinds and kinds != {None}:     # build / dev は数えない
                continue
            out.append(d["pkg"])
        return out
    def reach(starts):
        seen, stack = set(), list(starts)
        while stack:
            cur = stack.pop()
            if cur in seen: continue
            seen.add(cur); stack.extend(deps(cur))
        return seen
    root = [i for i, n in name.items() if n == "motolii-ui"][0]
    allr = reach([root])
    edges = sorted({(name[a], name[b]) for a in allr for b in deps(a)})
    dup = subprocess.run(["cargo","tree","--duplicates"], cwd=ROOT,
                         capture_output=True, text=True).stdout
    facts = {"packages": len(allr), "direct": len(deps(root)),
             "rerun_pkgs": len(reach([i for i in allr if name[i].startswith("re_")])),
             "duplicates": len([l for l in dup.splitlines() if l and l[0].isalpha()])}
    return edges, facts


def main():
    out = sys.argv[1] if len(sys.argv) > 1 else f"{ROOT}/motolii/target/maps/dependencies.html"
    os.makedirs(os.path.dirname(out), exist_ok=True)
    size = bytes_by_crate()
    sites, items, files = usage(size.keys())
    total = sum(size.values())
    rows = {}
    for crate, n in size.items():
        if n < 8192: continue
        rows[crate] = {"crate": crate, "bytes": n, "share": n * 100.0 / total,
                       "family": family(crate), "sites": sites.get(crate, 0),
                       "items": len(items.get(crate, ())), "files": len(files.get(crate, ()))}
    edges_raw, facts = graph()
    norm = lambda s: s.replace("-", "_")
    adj = collections.defaultdict(set)
    for a, b in edges_raw:
        adj[norm(a)].add(norm(b))
    links, parents = [], collections.defaultdict(set)
    for a, targets in adj.items():
        if a not in rows: continue
        for b in targets:
            if b in rows and b != a:
                links.append({"source": a, "target": b})
                parents[b].add(a)
    # 背後の重さ: その crate を通らないと届かない物の合計(ラッパーが抱えている量)。
    def reachable(starts, block=None):
        seen, stack = set(), list(starts)
        while stack:
            cur = stack.pop()
            if cur in seen or cur == block: continue
            seen.add(cur); stack.extend(adj[cur])
        return seen
    everything = reachable(["motolii_ui"])
    nodes = []
    for crate, r in rows.items():
        lost = everything - reachable(["motolii_ui"], block=crate) - {crate}
        nodes.append({**r, "parents": sorted(parents.get(crate, ()))[:8],
                      "behind": sum(rows[c]["bytes"] for c in lost if c in rows)})
    nodes.sort(key=lambda n: -n["bytes"])
    payload = {"facts": facts, "total": total, "nodes": nodes, "links": links,
               "generated": subprocess.run(["date","+%Y-%m-%d %H:%M"], capture_output=True, text=True).stdout.strip(),
               "head": subprocess.run(["git","-C",ROOT,"rev-parse","--short","HEAD"], capture_output=True, text=True).stdout.strip()}
    open(out, "w").write(TEMPLATE.replace("__DATA__", json.dumps(payload, ensure_ascii=False)))
    print(f"{len(nodes)} crate / {len(links)} 辺 / text {total/1048576:.1f} MiB → {out}")


main()
