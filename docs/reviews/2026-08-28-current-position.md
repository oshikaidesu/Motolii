# 現在地(2026-08-28) — 引き継ぎ台帳

**この文書は現在地であって歴史ではない。** 進んだら書き換える。次のセッションはここから読む。

## 第一目標

**GUI だけで、普通の編集の輪が回ること**(2026-08-27 利用者裁定)。

> 素材の配置 → エフェクトの適用 → プレビューカメラの分離 → xyz などパラメータの駆動 →
> 区間イージング GUI。**一通り普通に使えるようになるまで。**

書き出しも配布も、その後。まず**作れる状態**になること。目的は
[裁定273](../decision-index.md)(自分の MV を作る)であり、優先順位の根は
**「自分が1本作るのに要る物が先」**。

## いま何が在って、何が無いか

**背骨は端から端まで通っている** — 作り物のドキュメント → `motolii-engine` →
`motolii-compositor`(re_renderer)→ 共有 Surface → Stage。絵は出る。

**入口と出口が無い** — `app/` の実行可能バイナリは front 1つだけ。`motolii-export` を
呼ぶ者はテスト以外に居ない。front が読むのは `motolii_fixture::build()` の作り物で、
**実素材が1つも入っていない**。

| 第一目標の5つ | store(意味) | front(操作) |
|---|---|---|
| 素材の配置 | `AddLayer`/`SetSource`/`SetTiming`/`SetOrder` | **ファイルを開く口が無い**。Browser は8枚の作り物 |
| エフェクトの適用 | `EffectId`/`EffectInstance`/`SetEffects` | FX STACK は**見た目だけ** |
| カメラ分離 | `ResolvedCamera`(compositor が使う) | **タブのラベルだけ**(`"User View"`) |
| xyz の駆動 | プロパティ track・`SetTrack`・`value_at` | `ScrubValue` は入ったが store へ届かない |
| 区間イージング | 補間は track の中 | **`easing` も `keyframe` も1行も無い** |

### 5つが共有している、たった1本の欠落

**front から store へ書く経路(`Intent`)。**

`Intent` は既に27種ある(`app/core/motolii-store/src/document.rs:32`)。
第一目標に必要な物はほぼ全部揃っている。**意味の側は待っている** — front が呼んでいないだけ。

だから最初の仕事は**この経路を1本ちゃんと通すこと**。1本通れば残りは同じ型の反復で、
今日のように5レーンが別々に「store へ届かない」と報告する事態も消える。

**front 側の先例**: `main.rs` の `toggle_lane_flag_from_timeline` と
`restack_from_timeline` は実データへ書いている。新しい経路を発明せず、この形に倣う。

## 発注の順序

1. **front→store の経路を1本**(`SetTiming` = トリム)— 型を作る。**ここだけ設計が要る**
2. **素材の配置** — ファイルを開く・probe・`AddLayer`+`SetSource`。**ここから実物が入る**
3. **xyz の駆動**(`SetTrack`)→ **区間イージング** — 同じ track の上なので続き
4. **カメラ分離** — 空間で作業するなら要る。store を触らないので独立
5. **エフェクトの適用**(`SetEffects`)— ここから Vism が効き始める。LLM が並列で書ける場所

1 が終われば 2〜5 は**並列に出せる**(write-set が互いに素)。

## 直前の状態(2026-08-27 夜)

- **21件の UX 欠陥**: 5レーン並列実装が完了、PASS 17 / PARTIAL 4 / FAIL 0。
  worktree にあり、**本体へは未統合**。統合と実窓検収を Sonnet 5 に発注済み(走行中)
- **PARTIAL 4件の正体**: A1(トリム)・A5(クリック選択)・D1(数値ドラッグ)は
  front で動くが store へ届かず、次のモデル再設置で黙って戻る。**「消える虚報」**。
  裁定 = **一度隠す**(効いたように見えて戻るのは、できないより悪い。一度嘘をつくと
  利用者は他のどの操作が本物かも分からなくなる)。E5(色見本)も同じ理由で受け口を外す
- **これらは上記「発注の順序」の 1 が通った時点で本物として戻る**

## 決まっている事(読む順)

1. [CANON](../CANON.md) — 憲法5条・何のために作るか・hero・自作の範囲
2. [裁定271-273](../decision-index.md) — 意図論 / Lottie が審判 / 目的は自分の MV
3. [注意の失敗と世界の分断](2026-08-27-attention-failures-and-the-partition.md) — 裁定270
4. [レイヤー UI 欠陥台帳](2026-08-27-layer-ui-parity-defects.md) — 21件の一覧
5. [AGENTS.md](../../AGENTS.md) — 運転(製品は `app/` に居る・ホットリロード運転)

## 効いている柵

- **AE parity の裁定は利用者に聞かず Lottie に聞く**(裁定272)。審判は
  `app/reference/lottie.schema.json` と `lottie-coverage.tsv`。利用者が要るのは
  Lottie が黙っている所と、意図的に AE から離れる所だけ
- **UX の分岐は仕様でなく意図論**(裁定271)。「利用者は何を求めてこの操作をするのか」。
  **操作は意図が名指した物だけを変える**
- **繋げるだけを自作に膨らませない** — 新しい型・trait を定義する瞬間に
  「これを既にやっている物は何か」を答える。答えられないなら探していない。
  **詰まったら `re_renderer` の中を読む** — 手元に在る:
  `~/.cargo/git/checkouts/rerun-bdb1f1ac6277bf7e/7cca401/crates/viewer/re_renderer`
- **Inspector に renderer のパラメータを出さない**(2026-08-28 利用者指示)。
  モデルが Lottie なので、利用者が触るのは**意味**(位置・不透明度・trim path・エフェクト)
  であって**機構**(MSAA・alpha channel usage・render config)ではない。
  判定は機械的: **Lottie で表現できない物は Inspector に出さない** — 出しても保存で消えるので
  利用者から見れば嘘になる。迷ったら `app/reference/lottie.schema.json` を引く
- **テストは9ファイル + Lottie 計器3本だけ**。見える物は窓で見る(裁定270)。
  足りなければホットリロードで足す
