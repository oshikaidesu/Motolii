# R0 probe — rerun store は編集ソフトの書き込みに耐えるか(実測)

日付: 2026-08-20
状態: **観察**(実測。裁定は[リセット裁定](2026-08-20-reset-to-one-axis.md)側)

[リセット裁定 §6](2026-08-20-reset-to-one-axis.md#6-最初に落ちるテスト--r0-probe-store-が編集に耐えるか) が
「軸の唯一の致命的な未検証点」と名指しした点の実測。**6/6 通過**し、軸は立つ。
ただし設計に**1点の訂正**が要る(§2)。

器具: `next/probes/r0-store-edit`(新 workspace の最初の crate)。
再実行: `cd next && cargo test --release -- --nocapture`

## 1. 実測値

| 試験 | 内容 | 予算 | 実測 | 余裕 |
|---|---|---|---|---|
| **R0-1** 編集耐性 | 300打点の property を 1000回書き換え | query < 1,000µs / store < 64MB | **query 9µs**(最新)・**2µs**(最古)、**3.5MB**、chunks **10**、書き込み **16.1µs/編集** | 100倍以上 |
| **R0-3** keyframe 密度 | 10 property × 300打点 × 300フレームを cache 無しで全評価 | 16,600µs/フレーム(60fps) | **10µs/フレーム**(合計 3.19ms) | 1,600倍以上 |
| **R0-4** 保存・読込 | `to_messages` → `add_log_msg` 往復 | 全 query 一致 | **一致**(50世代すべて) | — |
| **R0-5** custom component | `motolii.KeyFrame` / `motolii.KeyValue` を rerun の木の外で定義 | 往復すること | **成立**(`re_types` を fork せず) | — |
| **R0-2** undo/redo | 1000編集を跨いだ時間移動 | drop も replay も無しで戻る | **成立**(latest-at の移動だけ) | — |
| **R0-A** 2次元 query | comp 軸に Document を置けるか | — | **置けない**(§2) | — |

補足の実測:

- **依存グラフに `egui` / `eframe` / `winit` / `iced` が 0件**(`cargo tree --edges normal`)。
  旧 workspace で名目化していた柵が、新 workspace では依存グラフの事実になっている
- ビルド: 初回 `cargo build --tests` **33.5s** / `cargo test --release` **1m48s** /
  **増分ビルド 1.68s**
- 1000編集 × 300打点の生データは 3.6MB で、store は 3.5MB。**保持オーバーヘッドは実質ゼロ**
  (1000行が 10 chunk へ自動圧縮されている)

## 2. 訂正 — Document は `comp` 軸に載らない(R0-A)

`LatestAtQuery` は **単一 timeline しか取らない**(`re_chunk/src/latest_at.rs:16-21`)。
したがって「**comp=0 の値を、edit=0 の時点で**」という2次元の問い合わせが原理的に書けない。

R0-A が機械で固定した帰結:

1. 2打点の property を `comp` 軸に打つ(comp=0→10.0、comp=10→20.0)
2. comp=0 の打点を編集する(edit=1 で 99.0)
3. `latest_at(comp, 0)` は **99.0** を返す。編集前の 10.0 を返させる query は**無い**
4. `latest_at(edit, 0)` は編集前へ戻れるが、それは「最後に書かれた行」であって
   comp 位置を選べない

つまり **Document を comp 軸に置くと、undo が query では成立しない**(drop + replay になる。
redo も失う)。[リセット裁定 §2](2026-08-20-reset-to-one-axis.md) の対応表のうち2行を訂正する:

| AE の概念 | 訂正前(裁定 §2) | **訂正後(R0-A 実測)** |
|---|---|---|
| keyframe | 疎な chunk(comp timeline 上)+ 補間 | **property track まるごと1行**(`edit` timeline 上) |
| 現在時刻の値 | latest-at / range query(comp timeline) | **Motolii の評価器**(track を latest-at で1回取り、補間する) |

他の行(layer = entity path、property = component、undo = edit timeline の時間旅行)は変わらない。

**この訂正は軸を弱めず、強める**:

- undo も redo も **query の移動だけ**で成立する(rerun blueprint の undo と完全に同じ機構)。
  drop も replay も要らない — R0-2 が 1000編集跨ぎで確認
- store が扱うのは「小さな値を append して latest-at で引く」だけになり、
  これは rerun の store が最も得意な形そのもの。R0-1 の余裕(100倍)はここから来ている
- **時間の意味が1箇所に集まる**: store は「いつ編集されたか」だけを持ち、
  「作品のいつの値か」は評価器だけが持つ。段差が増えるのではなく、責任が分かれる
- eased 補間・expression は元々 rerun の latest-at(step 補間)では表せないので、
  評価器は**どのみち Motolii 側に要った**。訂正後の形はそれを前提に置き直しただけ

## 3. 空席・観察点(通ったが目をつぶらない)

- **undo 履歴の GC 方針が要る**。全世代を保持するので、1000編集 = 3.5MB は
  10万編集 = 約360MB になる。rerun は `EntityDb::gc_with_target` を持ち、
  rerun 自身の blueprint undo は `MAX_UNDOS = 100` で切っている
  (`re_viewer_context/src/undo.rs`)。Motolii の方針は未決
- **依存グラフに gRPC/protobuf 系(`re_protos` / `tonic`)が入っている**。
  store だけが要るので feature を削れる余地がある(未着手・ビルドは既に十分速いので急がない)
- R0 は**単一 property**の track を測った。layer 数・entity path 数が増えた時の
  latest-at は未測
- 評価器は**線形補間のみ**を測った。AM式高度イージング(Bounce/Elastic/Steps=区間補間)・
  expression のコストは未測
- 素材ピクセル・音声波形を store に置くかは未決(R0 は数値 property だけを測った)
