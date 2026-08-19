# リセット — 軸1本(rerun store を Document に、iced を唯一の front に、拡張口を1つに)

日付: 2026-08-20
状態: **決定**(利用者裁定)

## 裁定(利用者の言葉から)

> 今のmotoliiはドリフトの修正でドリフトが入り、もうぐちゃぐちゃだ。ここで1度リセットをする必要がある。
> 軸は1本。rerunのフォークでae化、icedでrustの思想を持ちバックとフロントが一致する段差の無い設定、
> 尚且つ、最小コアのプラグインベース。

同日の追加裁定(2択):

- **fork 射程**: `crates/store/*` と `re_renderer` を **pin fork から引く**。AE の意味は
  `re_types_core` の custom component として **Motolii 側 crate に建てる**。fork は seam のためだけ
  (既存の[rerun fork seam 台帳](2026-08-18-rerun-fork-seam-ledger.md)方式をそのまま継続)
- **リセットの器**: **新 workspace へ切り、生存資産を移植**。旧 workspace は歴史証拠として残す

## 1. 何が「ぐちゃぐちゃ」だったのか(実測)

| 症状 | 実測値 | 出所 |
|---|---|---|
| shell が2つ並走 | `motolii-ui`(egui)54,053行 / `motolii-shell-iced` 21,240行。M-5(既定bin切替)未実施で **egui が今も既定** | [icedホスト移行裁定](2026-08-18-iced-host-migration-decision.md)、[CANON](../CANON.md) |
| 柵が構造でなく宣言 | `motolii-shell-iced/Cargo.toml` は「egui は入れない」と書きつつ `motolii-ui.workspace = true` を引く。`motolii-ui` は `egui`/`eframe`/`egui-wgpu`/`egui-winit`/`egui_tiles` を**全て非optional**で持つ。**iced 殻のビルドは毎回 egui 一式をコンパイルしている** | 両 `Cargo.toml` |
| 正本が面ごとに逆向き | Browser/Inspector は「HTML/CSS が正本・egui 実装は手本にするな」、Timeline は「egui 実装が正本」 | [CANON](../CANON.md) |
| 翻訳層の増殖 | `inspector_panel/read_model.rs` → `inspector_model.rs`、`timeline_editor/` → `timeline/semantics.rs`。同じ意味が2〜3回書かれている | — |
| 台帳の増殖 | `docs/**.md` 646本(reviews 543本)、14日で 595 commit | — |
| 残骸 | spike 21本、`ui/motolii-rn`(TS 9,159行)、`ui/motolii-web`(空) | — |

**診断**: 個々の修正は正しかった。壊れていたのは「Document の意味を誰が持つか」が1箇所に決まっていなかったこと。
そのため *Document → UI モデル* の翻訳層が面ごと・shell ごとに増殖し、翻訳層どうしのズレを直す修正が
また別の翻訳層を生んだ。柵(dep policy)は宣言であって依存グラフを止めていなかったので、分離は名目だけだった。

## 2. 軸が要求する構造 — Rerun は既に UI と backbone が切れている

`~/rust_ae/rerun-s2-seam-20260818`(現 pin `483b8559`)を実測した。

| 層 | 実体 | egui 依存 | リセット後の扱い |
|---|---|---|---|
| store | `re_chunk_store` 18,025 / `re_query` 12,786 / `re_log_types` 9,505 / `re_entity_db` 7,989 | **無し** | **引く**(Document の実体) |
| renderer | `re_renderer` 24,305 | **無し** | **引く**(合成器の実体) |
| viewer | `re_ui` / `re_time_panel` / `re_selection_panel` / `re_blueprint_tree` / `re_viewport` / `re_view_spatial` 26,438 | 有り | **引かない**(iced が置き換える層) |

そして決定打が `re_viewer_context/src/undo.rs` の冒頭にある:

> "We store the entire edit history of a blueprint in its store.
> When undoing, we move back time, and redoing move it forward.
> When editing, we first drop all data after the current time."

**Rerun の undo は時間旅行そのもの**であり、store は名前つき timeline を複数持てる。
つまり AE 化の対応が構造として付く:

| AE の概念 | rerun の機構 |
|---|---|
| composition | store(1 recording) |
| layer | entity path |
| property | component |
| keyframe | 疎な chunk(comp timeline 上)+ 補間 |
| 現在時刻の値 | latest-at / range query(comp timeline) |
| **undo / redo** | latest-at query(edit timeline)**— 既存機構** |
| camera layer | `SpatialStage::set_camera` seam(2026-08-18 に新設済み) |
| precomp | entity path の入れ子 |

> **訂正(2026-08-20・同日の R0 実測)**: 上表のうち **keyframe と「現在時刻の値」の2行は誤り**だった。
> `LatestAtQuery` は単一 timeline しか取らないので、「comp=F の値を edit=E 時点で」という
> 2次元の問い合わせが書けない。したがって Document は `comp` 軸に載らない。
> 正しくは **keyframe = property track まるごと1行(`edit` 軸上)**、
> **現在時刻の値 = Motolii の評価器**(track を latest-at で1回取って補間)。
> 他の行は変わらず、undo/redo は両方とも query の移動だけで成立する(むしろ強くなる)。
> 実測と理由は [R0 probe §2](2026-08-20-r0-store-edit-probe.md#2-訂正--document-は-comp-軸に載らないr0-a)。

### 「バックとフロントが一致する段差の無い」の意味

front は store への query の**投影**、write は chunk の **append**。それ以外の状態を front は持たない。
これで *Document → UI モデル* の翻訳層が**設計上存在しなくなる** — 段差はそこに溜まっていた。
iced の `Message` は「store へ何を append するか」であり、`view` は「今の (comp_time, edit_time) で
store をどう読むか」になる。iced の Elm 構造と store の temporal query が同じ形をしているので、
接着剤が要らない。これが「rust の思想を持ち」の実体。

### 「最小コアのプラグインベース」の意味

**コア = store + schema + 評価ループ**。それ以外は全部拡張。
現在の拡張口は4本(`FilterPlugin` / `LayerSourcePlugin` / `ParamDriverPlugin` / `CompositePlugin`、
`motolii-plugin` 3,979行)だが、store モデルではこの4本は**同じ形**に潰れる:

- `ParamDriverPlugin` = component を読んで component を書く
- `FilterPlugin` = component + texture を読んで texture を書く
- `LayerSourcePlugin` = component を読んで texture を書く(入力 texture 無し)
- `CompositePlugin` = 複数 texture を読んで texture を書く

いずれも「**(path, comp_time) で component を読み、値か画を書く**」1種類。宣言した入出力 component 集合が
違うだけ。拡張口は **trait 1本** に収束する。first-party も third-party も同じ口を通り、
出自による実行特権を持たない([小さなコアと探索可能な拡張](../extensible-core-model.md) §1.1 の分類は維持)。

## 3. 新 workspace の crate 地図

```text
motolii/
  core/
    motolii-store       Document = EntityDb。timeline 2本(comp = 作品時間 / edit = 編集履歴)。
                        読み = StoreView(不変)、書き = Intent → chunk append。
                        undo/redo = edit timeline 上の移動(自前機構を作らない)
    motolii-schema      AE component 定義(Layer / Transform / Opacity / Blend / TimeRemap /
                        EffectRef / Keyframe / Interp)を re_types_core の custom component として
    motolii-eval        (path, comp_time) → 値。keyframe 補間・式・effect 適用順
    motolii-ext         唯一の拡張口。trait 1本 + 登録
  engine/
    motolii-compositor  re_renderer 直叩き。layer = テクスチャ板、camera、順序、blend。
                        preview と export は同じ関数、窓の有無だけが違う
    motolii-media       decode / encode(移植)
    motolii-audio       (移植)
    motolii-export      mux(移植)
  shell/
    motolii-shell       iced のみ。pane は StoreView の投影
    motolii-input       toolkit-free(無傷で移植)
  ext/                  first-party 拡張(参照実装。Opacity / Sine / Radial Repeater を書き直す)
```

### 削れない背骨(構造で強制する。監督で守らない)

1. **shell は store の可変ハンドルを持てない** — shell が受け取るのは `StoreView`(不変)と
   `Intent` の送り口だけ。型で禁じる(現行の `intent_gateway_fence.rs` が文字列 grep で守っている物を、
   型の見えない所へ移す)
2. **評価経路は1本** — `Compositor::render(&StoreView, comp_time, camera) -> Texture`。
   preview も export もこれを呼ぶ。第二経路を作れる公開 API を置かない
3. **拡張口は trait 1本** — 4本目の口が要ると感じたら、それは component の設計が足りていない合図

## 4. 生存資産の移植表

`motolii-shell-iced` と `motolii-ui` 以外の**全 crate が toolkit-free** であることを実測で確認した
(`Cargo.toml` に egui/eframe/iced/blitz を持つのはこの2つだけ)。したがって engine 側は素直に移植できる。

| 資産 | 行数 | 扱い |
|---|---|---|
| `motolii-input` | 2,413 | **無傷で移植**(toolkit-free、移行裁定でも移行対象外と確認済み) |
| `motolii-render` / `motolii-media` / `motolii-audio` / `motolii-export` / `motolii-gpu` | 19,583 | **移植**。`motolii-render` は合成部分を `motolii-compositor` へ譲り、per-layer 評価に縮む |
| `motolii-doc` | 52,922 | **溶かす**。schema/persist は store へ。ただし `param_eval` / `pathgeom` / keyframe 補間 / D2 command の意味は**捨てる資産ではなく AE 化の中身**として `motolii-schema` / `motolii-eval` へ移す |
| `motolii-plugin` + first-party 3本 | 4,392 | **書き直す**(4口 → 1口)。`GpuCtx` / `TextureRef` の GPU 境界規律は継承 |
| `timeline_editor/`(egui) | 9,059 | **操作カタログの正本として意味関数だけ移す**。UI は移さない。[能力台帳](2026-08-19-egui-timeline-capability-ledger.md)が移植対象の一覧 |
| `motolii-shell-iced` の pane 構造 | 21,240 | **種として使うが正本にしない**。翻訳層(`inspector_model.rs` / `timeline/semantics.rs`)は store 投影に置き換わるので消える |
| `motolii-ui`(egui shell) | 54,053 | **落とす** |
| `ui/motolii-rn`(TS) / `ui/motolii-web` | 9,159 | **落とす** |
| `spikes/**` 21本 | — | 旧 workspace に歴史証拠として残す。移さない |
| `docs/**` 646本 | — | 移さない。新 workspace は本書 + CANON + decision-index の3枚から始める |

## 5. このリセットが撤回するもの(目をつぶらない)

| 既決 | 日付 | 処分 |
|---|---|---|
| 「Motolii は Rerun Spatial **Viewer** の creator-facing wrapper。**direct `re_renderer` scene・第二 runtime を禁止**」 | 2026-08-11 | **撤回**。viewer(egui)を引かない以上、direct `re_renderer` が唯一の道。ただし禁止の趣旨(第二 runtime を作らない)は背骨2で維持する。Motolii が必要なのは「テクスチャ板 × 順序 × カメラ × blend」で、`re_view_spatial` の 26,438行は全 rerun archetype + egui 対話の分だから、置き換えは同スケールにならない |
| iced 移行 M-0〜M-5(絞め殺し方式) | 2026-08-18 | **打ち切り**。M-4 まで到達したが M-5 に届かず、柵が名目化したまま2 shell が並走した。方式ごと新 workspace へ置き換える。移行裁定の**方向**(ホストは iced)は維持 |
| Timeline の正本 = egui 実装 | 2026-08-19 | **縮小**。「操作の正本」としては維持、「UI の正本」としては新 shell へ移る |
| Browser / Inspector の正本 = HTML/CSS モック | 2026-08-19 | **維持**。視覚の手本は変えない |
| Rerun を合成のメイン基盤とする | 2026-08-18 | **維持・強化**([合成基盤裁定](2026-08-18-rerun-as-composition-foundation.md))。E0 の3点(offscreen 決定性・カメラ注入・遮蔽)は[実測済み](2026-08-18-rerun-e0-composition-probe.md) |
| Timeline は AE 型の自由配置 | 2026-08-19 | **維持**([配置土台裁定](2026-08-19-timeline-packing-model-decision.md)) |

## 6. 最初に落ちるテスト — R0 probe(store が編集に耐えるか)

この構造の**唯一の致命的な未検証点**は、rerun の store が「append 主体の観測ログ」向けに作られており、
**編集ソフトの書き込みパターン(同じ component を何百回も上書きする)で成立するか実測されていない**こと。
ここが落ちるなら軸そのものが立たないので、他の何より先に単独レーンで実測する。

落ちるテストを先に書く([発注は落ちるテストで渡す](../decision-index.md)):

1. **編集耐性**: 1 layer の Position component を comp timeline 上の1点に対して 1,000回上書き
   (= scrub 中のドラッグ相当)。store のメモリ・latest-at query のレイテンシが線形に劣化しないこと
2. **undo 粒度**: 上記 1,000回が edit timeline 上で **1つの undo 単位**に畳めること
   (rerun の inflection point ヒューリスティックがドラッグを1単位に畳む挙動が、編集ソフトの
   期待と一致するか。一致しないなら自前の区切りを打てるか)
3. **keyframe 密度**: 300 frame に 300 keyframe を持つ property を 10本、latest-at + range query で
   1 frame 分の値を引く時間が実時間再生(60fps = 16.6ms)に収まること
4. **保存・読込**: 上記 store を保存 → 読込 → 全 query の結果が byte 一致すること
   (`.rrd` をそのまま document 形式にするか別 serialize かは、この実測の結果で決める)

> **結果(2026-08-20)**: **6/6 通過・軸は立つ**。1000編集 × 300打点で query 9µs・store 3.5MB、
> 10 property × 300打点 × 300フレームの全評価が 10µs/フレーム(60fps 予算の 1,600倍の余裕)。
> 依存グラフに egui/eframe/winit/iced は 0件。実測の全量は
> [R0 probe](2026-08-20-r0-store-edit-probe.md)。器具は `next/probes/r0-store-edit`。
> 上記4項目は R0-1/2/3/4 として常設試験になっており、加えて R0-5(custom component)と
> R0-A(2次元 query が書けないこと)を機械で固定した。

**不成立時の分岐**: (1)(3) が落ちたら store を「編集中は自前の疎な表現、確定時に store へ」の2段にする
(= 段差が1つ戻るので、その時点で軸を再裁定する)。(2) だけが落ちるなら fork seam で対処(既存の型)。

## 7. 空席(このリセットでは決めない)

- 音声波形・素材ピクセルを store に置くか外に置くか(R0-4 の結果待ち)
- `.rrd` を document 形式にするか(同上)
- plugin の process 分離 / sandbox(現行と同じく静的リンクから始める)
- 上流 rerun の追随頻度(seam 台帳方式は継続するが、store crates は viewer より変化が遅い想定 — 未実測)
- AccessKit(iced 上流未統合。移行裁定で立てた観察点をそのまま引き継ぐ)

## 8. 影響範囲

新 workspace の初期 crate 群、`motolii-doc` の解体、`motolii-plugin` の 4口→1口、
`motolii-ui` / `ui/motolii-rn` の落とし、旧 workspace のアーカイブ化。
