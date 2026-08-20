# 旧ステージから持ち込む理想文書の索引

2026-08-20 のリセット後、実装の正本は `next/` に移ったが、**理想・概念レベルの文書は旧ステージ(`../../docs/`)に残っている**。
2.5D 空間モデルの見落とし(正本 `docs/reviews/2026-07-16-m2-comp-camera-decision.md` を知らずに supervisor が誤設計を提案しかけ、利用者に訂正された)が実際に起きたため、この索引を置く(利用者指示 2026-08-20)。

**この索引の役割**: どの旧文書がまだ新世界を拘束するかを1枚で示す。文書自体は動かさない(パスは `docs/` 配下のまま)。
新しい設計判断はここに書かない — 裁定は `../DECISIONS.md`、合否は `../GOALS.md`。

## A: 新世界を拘束する理想(発注前に該当領域の物を読むこと)

| 文書 | 何を拘束するか | 注意 |
|---|---|---|
| `docs/concept.md` | **製品の憲法**。MV制作の一文定義、「映像制作のVST」、プリコンポ廃止→グループ+ベイク、パス演算子ファミリー、「馬鹿正直にシミュレートしない」、8つの設計審判 | 裁定58〜73/113 の親。**未回収の目標あり**(下記) |
| `docs/reviews/2026-07-16-m2-comp-camera-decision.md` | **空間モデルの正本**(単一世界・単一カメラ・z=0既定)。裁定113で回収済み | Spatial 変種の前提リスト(向き・handedness・特異点)はここが正本 |
| `docs/vism-plugin-catalog.md` | **vism 候補の正本**。Glow/Bloom/Displace/Particle Field/Text animators/Transitions 等の候補名・意図・見た目 | 候補一覧は生存。**lane 分類(SINGLE/PORTS/…)は旧trait前提で失効** |
| `docs/vism-package-concept.md` | 配布面の憲法: Vismは名詞と動詞を発明できる、Hostは identity/時間/依存/寿命/Undo/資源/失敗を持つ。first-party特権禁止(裁定72の源流) | Package詳細field・停止線checklistはtrait1本縮約後は過剰 |
| `docs/extensible-core-model.md` | 「小さなコア」= architectural role。Host所有責任の列挙。**§7-8 個体性の4段階**(Particle/Instance設計時に必読) | 裁定13(traitはまだ作らない)とは順序の違いで矛盾ではない |
| `docs/generative-user-boundary.md` | 生成的表現の5正規経路(Materialize/Pure Live/Temporal Window/Bake/External)。p5.js型の翻訳表 | 生成系を扱う日に必読 |
| `docs/ui-quality-bar.md` | Q0〜Q9 + B1〜B7。GOALS の Q0/Q3/B系の親 | 制定時の違反inventoryはegui固有で死蔵 |
| `docs/ui-interaction-language.md` | 6状態の操作文法(Discover→…→Inspect)、Parameter Panel=表現のホーム、Silent disabled禁止 | 実装例のRN/egui記述は読み飛ばす。力学だけ拾う |
| `docs/interaction-simplicity-model.md` | S-1〜S-5(1つの意味に3つの入口、探索を罰しない) | UI束の必読 |
| `docs/ui-concept.md` | 五本柱(結果と時間が見える/密度は資産/軽さは機能)、「最初の結果」北極星 | — |
| `docs/ui-score-model.md` | Timeline の時間面モデル(Laneを所有者にしない、bar一枚packing、Depth Rail、Inbox) | **Timeline pane 束の必読**。旧egui実装への参照は死蔵 |
| `docs/ui-visual-language.md` | 「読む前に分かる」、意味色ロール、G0-6審判プロトコル | 具体token値・DTCG・adapterは死蔵 |
| `docs/ae-pain-points.md` | AE不満の体系分類 = 製品ポジショニングの一次資料。D5(文字列式不要)と直結 | — |
| `docs/community-distribution-model.md` | ガバナンス: 中央marketplaceを主回路にしない、人気を正本にしない、Project Lock=再現 | Kit機構部分は下記Cへ |

## 未回収の目標(concept.md / catalog にあり、GOALS/DECISIONS が未言及)

- **「解析→生成」路線**(色解析→DataTrack→パラメータ駆動。concept.md の最終フェーズ)
- **歌詞/日本語組版を第1号プラグインにする方針**(D11 の隣にあるが明文化されていない)
- catalog の候補 effect 群は effect 束(trait 確定)後の発注候補リストとしてそのまま使える

## C: 部分的に生きる(結論だけ拾い、機構は写さない)

| 文書 | 生きている部分 | 死んだ部分 |
|---|---|---|
| `docs/simulation-model.md` | 時間軸自由度 L0〜L3 の概念、逐次状態はレンダ外でベイク | `SimulationPlugin` trait 等の旧API |
| `docs/performance-model.md` | 律速=メモリ帯域(裁定21が独立に再発見)、Draft/Final、色空間の危険 | `motolii-gpu` API(裁定22で不建立) |
| `docs/memory-model.md` | 容量逼迫と再生期限の別ループ、hard budget、退避はしご | ResourceLedger実装状況 |
| `docs/text-model.md` | スタイルスパン/アニメーター二層の骨格 | 詳細は裁定76〜89と地図が正本に昇格済み |
| `docs/pitfalls-and-roadmap.md` | 普遍教訓(Undo=ジャーナル、VFR正規化、フレームN-1依存禁止、2.5D) | RN/wgpu/egui前提の実装穴 |
| `docs/ux-check-first-ten-minutes.md` | ペルソナ別「最初の10分」台本という手法(Q0/M系検収に転用) | 旧shell前提の手順 |
| `docs/ui-inherited-grammar-gap.md` | 「刻まれた文法」の分類と核の一周(入れる/並べる/切る/見る/書き出す) | 旧実装の配線状況 |
| `docs/vism-kit-model.md` | Preset/Kit の境界定義、BPM Grid の責任分解の思考実験 | **型付きport連鎖のKit機構は裁定72と構造矛盾 — 復元禁止** |

## B: 失効(読まない。再発明・復元の禁止リスト)

- 旧 plugin 機構一式: `plugin-authoring.md` / `plugin-resources.md` / `plugin-ui-model.md`(4trait体系は裁定72で置換)
- `known-implementation-adoption-model.md` の **10欄preflight儀式** — 裁定39/43「保守をしたくない」と正面衝突。復活させない
- M3〜M5 の dispatch 地図・実装台帳・runbook 群(RN+rust-skia+旧Rerun Viewer前提。冒頭に自己失効バナーあり)
- `ui-runtime-architecture.md` 等の旧shell責任境界・用語・反映辞書
- `decision-index.md` — 旧世界の歴史台帳として凍結(自己宣言あり)。ただし「実装でなく意味」の行(Creator/Developer連続体、first/third同一境界の源流、`.vism`名称)は上のA文書経由で生きる
- **要注意の失効例**: 「Motolii は Rerun Spatial Viewer の creator-facing wrapper」という旧記述は裁定3(viewer層を引かない)と正面衝突。この文言を根拠に viewer 層を引き込まない
