# Motolii

**After Effects の手つきで使える、rerun の上に建てた映像制作ソフト。** 文字・画像・動画・形・立体・点群を同じ舞台に置き、1 本の曲のタイムラインで MV を作るための物です。

<p align="center">
  <img src="docs/assets/rgb-trail.gif" alt="実写の上で動く 3 つの形に、群ごと色ごとに減衰する残像。上の長方形は少し前の背景を映す" width="800">
</p>
<p align="center"><em>群に残像、その上に「少し前の背景」— どちらも数十行の shader。スクラブしても同じ絵になる</em></p>

> **After Effects に渡した物は、全部、平らな長方形になる。** 3D スキャンも、点の群も、カメラの道も、音の波形も、合成に入った瞬間に「元が何だったか」を忘れた板に押し潰される。

Motolii は、どちらも起こりかけて起こらなかった **2 つの if** の上に建っています。

**もし合成が、意味を潰さなかったら。** Motolii の舞台は [rerun](https://rerun.io) — ロボットとコンピュータビジョンのために作られた、点群が点群のまま、網が網のまま、線が線のままでいる器です。その上で合成するとは、絵ではなく**意味**を重ねること。だから Motolii では、**1 個の効果を 1 度書けば、動画にも、文字の輪郭にも、網にも、点群にも同じ札のまま乗る**。乱流で歪ませれば、板の絵ではなくシルエットそのものが曲がる。

**もし AviUtl の文化が、After Effects の文法と出会っていたら。** 20 年、無料でローカルな日本の編集ソフトが、作者の想像を越える形に拡張され、その隙間に MV/MAD の文化が育った。一方で AE の深さは vendor の SDK の奥に封じられ、2 つの島に橋は架からなかった。Motolii はその架からなかった橋です。手が覚えている AE 系の編集の文法と、拡張の自由を憲法として。

## 約束していること

- **編集の文法は AE 系。** 層、キーフレーム、群、マット、クリッピング、カメラ、文字、形。手が覚えている操作を、覚え直させない
- **素材は意味のまま舞台に居る。** 動画も点群も網も線も、板に焼かれてから合成されるのではなく、そのものとして置かれる
- **効果は file である。** SDK も build も要らない。shader を 1 file 保存すれば棚に並び、Shadertoy を貼ればそのまま動く。壊れていれば理由が出て、前の物が残る
- **1 個の効果はどの素材にも乗る。** 素材ごとの版を書かせない
- **時間は host が持つ。** 別の時刻の絵も、前のフレームを保つ効果（残像・軌跡・蓄積）も、効果が自分で覚えるのではなく host が渡す。だから同じ時刻は、何度描いても、どの順で辿っても、同じ絵になる。プレビューと書き出しも同じ 1 本の道
- **作品は開いた形式。** rerun の `.rrd`。裁定も試験も公開で、fork できる

## 約束していないこと

- After Effects の代替であること。文法は借りるが、AE のプラグイン・プロジェクト・表現の互換は無い
- Motolii の効果（Vism）が他のソフトで動くこと。万能の plugin 形式ではなく、この host と、同じ契約を採る fork のための物
- 全ての OS で動くこと。今は macOS の開発用 build だけで、配布物は無い
- 完成していること。UI は変わり続け、未完の機能がある。現在地と未完の一覧は [docs/stage5/README.md](docs/stage5/README.md)

## 効果を書く人へ

`vism/` に file を置く。ISF（`.fs`）、Shadertoy（`.frag`、または Export の JSON）、WGSL（`.wgsl`、場・面・pass）。取説は [場の取説](docs/vism-field-model.md)、[Shadertoy の取り込み](docs/vism-shadertoy-import.md)、[時間の法](docs/plugin-resources.md)。

## 動かす・加わる

動かし方と現在地は [docs/stage5/README.md](docs/stage5/README.md)。設計と裁定の入口は [docs/README.md](docs/README.md)、[CONTRIBUTING.md](CONTRIBUTING.md)、[motolii/AGENTS.md](motolii/AGENTS.md)。なぜ作るかの長い版は [MANIFESTO.ja.md](MANIFESTO.ja.md)、要約は [VISION.ja.md](VISION.ja.md)。

## License

[Apache-2.0](LICENSE-APACHE) または [MIT](LICENSE-MIT)。

---

## English

**Motolii is a layer-based video editor with After Effects' grammar, built on [rerun](https://rerun.io).** Text, images, video, shapes, meshes and point clouds share one stage and one song-length timeline.

Two *what ifs*: a compositor that never flattens meaning — a point cloud stays a point cloud, so **one effect written once lands on video, text outlines, meshes and clouds alike** and bends their silhouettes — and the bridge that was never built between AviUtl's extension culture and After Effects' depth.

What it promises: AE-family editing grammar; materials that stay what they are; effects that are files (ISF, Shadertoy, WGSL — save and it is on the shelf); one effect for every material; time owned by the host, so effects that read other times or keep a history give the same picture however you scrub, and preview equals export; an open project format. What it does not promise: replacing After Effects, running its plugins, running on every OS, or being finished. macOS development build only, no binaries. Docs: [field model](docs/vism-field-model.md), [Shadertoy import](docs/vism-shadertoy-import.md), [the laws of time](docs/plugin-resources.md), [current status](docs/stage5/README.md). Dual-licensed Apache-2.0 / MIT.
