# vism契約の回収 — 旧世界で決まっていた外枠と、next/の現在地

日付: 2026-08-23 / 状態: **回収完了（速度優先・簡略版）**。利用者指示によりスコープを縮小し、骨4本＋歴史回収1本＋レビュー題名一覧に限定した。169本の精読はしていない。

前提: `git merge main` 実行済み（差分0、already up to date）。本ファイル以外は変更していない。

## 0. 読んだ範囲

- 精読: `docs/vism-plugin-catalog.md`、`docs/vism-kit-model.md`、`docs/vism-package-concept.md`、`docs/vism-known-implementation-adoption-map.md`、`docs/reviews/2026-07-23-historical-vism-foundation-contract-lineage-recovery.md`（計5本）
- 部分参照（該当箇所のみ）: `docs/reviews/2026-08-17-vism-param-list-type-decision.md`、`2026-08-17-vism-identity-known-implementation-survey.md`、`2026-07-23-vism-kit-rack-unification-decision.md`、`2026-07-27-vism-authoring-journey-decision.md`、`2026-08-01-vism-authoring-language-boundary-decision.md`（各文書冒頭の状態行のみ）
- 題名だけ一覧（開いていない）: `docs/reviews/`配下 `*vism*` 約90件。目についた物のみ§4に記す
- 新世界側: `next/DECISIONS.md`全212裁定、`next/GOALS.md`、`next/README.md`、`next/reference/CANON.md`、`next/core/motolii-store/src/effect.rs`、`next/engine/motolii-compositor/src/effects/{mod.rs,glow.rs}`

## 1. 決まっていた契約（旧世界）

### 1.1 語彙の階層（vism-kit-model.md L9-23）

```
Core     = 文法（時間・型付き入出力・接続・identity・保存・Undo・資源・失敗）
Vism     = 語彙（一つの小さな映像表現/provider）
Kit      = 接続済みの文章／用途セット
Project  = 実際の作品
```

Vism同士は直接IDを参照しない。**型を宣言し、Kitが具体providerを選ぶ**（同L23-33）。

### 1.2 境界の形 — 3層分離（vism-package-concept.md §4, L75-113）

| 層 | 内容 |
|---|---|
| Expression contract (L75-86) | 安定identity/version、型付き入出力/parameter/default、時刻・seed・Quality宣言、Preview/Export共有評価意味。**Motolii固有のUI型・座標・API面をここへ出さない** |
| Package (L88-100) | 作者/由来/license/version/互換範囲、capability宣言、実装source/WGSL/WASM、fixture、署名。**field/containerは未決** |
| Host integration (L102-113) | 発見/install/依存診断、NodeDesc標準UI生成、Undo/single writer/保存、GPU resource/cache/StateTrack/Preview-Export、error/Cancel/accessibility |

Vismが持てるもの/Hostに残すものの一覧はL133-151。

### 1.3 パラメータの型

`docs/reviews/2026-08-17-vism-param-list-type-decision.md` L5: 状態「**決定**（型と規則）＋型・規則は実装済み（2026-08-17、旧egui世界）／GPU受け渡しとUIウィジェットは未実装」。同種の並びを持つparameterの型・規則を固定した文書。

### 1.4 同一性

`docs/reviews/2026-08-17-vism-identity-known-implementation-survey.md` L5: 状態「**比較中**（候補の提示であり、採否は未決）」— identity未決6問への既知実装対照。**結論は出ていない**。

`vism-kit-model.md` §10（L288-300）は5層のidentityを混ぜるなと定める: Vism package identity（配布系）／Vism entry identity（表現契約）／Kit identity（構成作者・配布系）／Project instance identity（Document）／Artifact・署名identity（build/trust系）。

### 1.5 kit / rack / package の指し示すもの

`docs/reviews/2026-07-23-vism-kit-rack-unification-decision.md` L1,3: 「Kit / Plugin Set統合決定 — Rack型の作者成果へ一本化する」。状態「**決定**（意味と用語を統合。公開schema、container、install、製品UIは未決）」。旧仮称`Plugin Set`を廃止しKitへ統合、Ableton Rackに似た「複数能力を接続済みの一単位へまとめる作者成果物」という構造類比を採った（vism-kit-model.md §2.1, L68-80）。

v1のKitは**materialize型**（常駐runtimeにしない）: Kitを選ぶ→preflight→展開案→**全体成功時だけ1 macro commit**→通常編集（§5, L160-182）。1回のKit追加=1 Undo、失敗時変更ゼロが不変条件。

### 1.6 作者の動線・記述言語の境界

`docs/reviews/2026-07-27-vism-authoring-journey-decision.md` L5: 状態「**比較中**」。作者journeyとshader依存closureの整理途上。

`docs/reviews/2026-08-01-vism-authoring-language-boundary-decision.md` L3: 状態「**決定**。一般creator-authorがprogramを書く段の公式作者言語をTypeScriptとする」。

### 1.7 candidate registry（技術選定のgovernance、vism-known-implementation-adoption-map.md）

状態: 「**決定／調査結果固定**」（L3）。K-WGPU/K-RERUN-SPATIAL/K-VECTOR/K-TEXT/K-LYON/K-ARROW/K-GLTF/K-PHYSICS/K-TS/K-JS/K-WASM/K-GRAPH/K-INTEROP/K-SUPPLY/K-INDUSTRYの各候補を`REUSE/WRAP/ADOPT/PATTERN/REJECT`に固定し（§2 L22-40）、§7（L130-152）で`VSM-A4I → A8G0 → P0I → LANG-TS-F0 → A5→B0→B1→B2 → …(7a-7h) → C0-C4`という**probe発注順序**を定める。`VSM-A9`が対象laneの非干渉を証明するまでは複数Vism runtime実装の並列発注表と読まない、と明記（§7末尾）。

### 1.8 第三者生態系（catalog §14.1、package-concept §12）

「第三者作者は同じrepository、release周期、優先順位を共有しない」ことを前提に、名前空間・capability依存・依存の公開前検査・conformance・provenance分離・非同期更新・追加的昇格の8条件を定める（catalog L228-245、package-concept L310-319）。「中央serviceや単一marketplaceを逆算しない」が原則であり、**禁止ではなく設計原則として並列実装可能性を作ること**が目的（catalog L201-222）。

### 1.9 歴史的基盤契約（lineage recovery、2026-07-23）

3層構造: `Document raw recipe（保存正本）→ immutable Contract Catalog（宣言検証・migration）→ prepared recipe（runtime-only）→ Executor Registry（実行可否）`。不等号「**保存できる ≠ 意味を検証できる ≠ このHostで実行できる**」を固定（同文書§1）。Project openはinstall/network/build/plugin code実行を起こさない。

## 2. 新世界の譲れない意見との衝突

next/DECISIONS.mdは2026-08-20に**軸を1本へリセット**（裁定1）しており、旧世界のvism決定はこのリセットの外側にある。CANON.md自身が「実装の正本はこの workspace だが、理想・概念（**vism候補**・空間モデル・UI品質バー・拡張の憲法）は旧`docs/`に正本が残っている」（next/README.md L60）と明言し、旧vism文書を"読むべき理想"として保持しつつ実装はしていない、という二層構造を自覚的に採っている。

| 旧世界の契約 | 新世界の裁定 | 衝突の形 |
|---|---|---|
| Vismは型付きinput/outputを宣言し、Kitが具体providerを型で接続する（kit-model §3, L82-96） | **裁定70**（effect.rs L11-13, 30-40）: Documentは`plugin_id`文字列＋順序＋param＋enabledのみを持つ。paramは既存`KeyframeTrack`+`PropertyId`の平坦trackへ**そのまま**乗る。**裁定72**「新機構ゼロ」 | 型付きport・provider/consumer接続・Kit解決という設計そのものが無い。plugin識別は文字列1本、paramは名前つき値の列で、型システム上の「input port」概念は存在しない |
| Kitはtyped connection＋preflight＋1 macro commitで用途をmaterializeする（kit-model §5, L160-182） | **裁定118**でようやく`apply_all`の部分コミット事故を修正（batch内Errで`drop_time_range`により巻き戻す）。Kit相当のatomic batch機構、preflight、typed connection解決は存在しない | Kitのmaterialize契約が要求する「全体成功時だけ1 commit」はDocument writer側の基礎機構としてまだ発展途上で、Kit概念そのものが未着手 |
| 拡張の口はいずれtraitとして公開契約になる（package-concept §4.1 Expression contract, L75-86） | **裁定6**「拡張の口はtrait 1本」だが**裁定13**「まだ作らない、2つ目の利用者（compositor）が現れるまで待つ」。GOALS.md D6行（next/GOALS.md L54）: 「意図的に未着手（DECISIONS #13）」 | 拡張契約(trait)は方向としては継承されているが、故意に凍結されている。現状の`EffectPass`（effects/mod.rs L1-6）は**compositorローカルのclosed enum**（Identity/Glowの2バリアントのみ）で、外部作者が実装できる公開traitではない |
| Vismはfilter/generator/simulation等をtyped texture in/outで受け、Host所有GPU資源だけを使う（adoption-map §3, L54） | **裁定14/26/44/45**: 合成は`re_renderer`直叩き、Stageは最終的にCPU経路（裁定44で裁定26のGPU共有埋め込みを撤回）。ゼロコピー合成という当初方針は「iced deviceへの埋め込み」の形で一度採用されたが撤回され、CPU readbackを伴う経路へ後退した | adoption-mapが前提する「Vismはtyped texture in/outだけを持ち、CPU readback 0」という単一pass Filterの理想契約と、Stage表示系がCPU経路へ後退した現状は同一平面の話ではないが、**GPU資源をHostがVismへどう受け渡すかという設計そのものが、next/にはまだ一つも実装されていない**（Glowはcompositor内部で完結し、外部Vismへの受け渡し経路が無いため検証不能） |
| Preview/Export共有評価意味・決定的seed（package-concept L83, kit-model L44「Coreは時刻・型・接続・評価順・循環拒否を持つ」） | **裁定15/18**: `Compositor::render` 1本、`Engine::render_frame(&StoreView, t, comp)` 1本を常設試験でbyte一致させる。**背骨2**に相当 | 方向は一致している。Vism実行がこの単一経路に乗る保証はまだ無い（Vism自体が無いため） |
| Vismへの書き口はKitのmaterializeやVism instance payloadなど専用形（package-concept §6, L195-215） | **裁定212**「背骨1＝`Intent`（24枝の閉じた列挙）がDocumentへの唯一の書き口」 | 将来Vismが実装されても、書き込みは既存Intent列挙を拡張する形になる可能性が高く、旧世界が想定した「Vism instance payloadを持つ専用構造」とは別の実装形になりうる。**まだどちらの形にするか裁定が無い** |
| 第三者生態系は本体と同じrepo/release周期を共有しない前提で並列実装可能に設計する（catalog §14.1, package-concept §12） | **next/GOALS.md「要らないもの」**（L73）: 「動的配布marketplace / 第三者SDK / **独自plugin UI** / VST互換」を明示的に除外 | 旧世界は「第三者生態系を後付けにしない」ことを設計原則としたが、next/はこれを一旦除外リストへ入れている。除外が「無期限凍結」なのか「設計原則自体の撤回」なのかを裁定した記録が無い |
| candidate registry・probe発注順序というgovernance構造そのもの（adoption-map §6-7） | next/にはVSM-A/B/Cという段階gate、probe/cutover/retirement規律に対応する仕組みが**存在しない**。wgpu・rerunはこの文書のgateを経ずに裁定3/14/17/26で直接採用された | この文書はまだ「決定／調査結果固定」ステータスのままだが、next/の実際の技術選定は既にこの統治構造の外で行われている |

## 3. いま生きているか

CANON.md（next/reference/CANON.md）は既にvism系文書のうち3本について生死判定を明記している（L15-16, 58, 62）。これは本回収作業の一部を既に済ませている貴重な先行資料である。

### 生きている

- **first-party特権禁止**（package-concept §7, L217-229）→ CANON.md L16「裁定72の源流」と明記。裁定70でも「int registryだとfirst/third-partyが同じ口にならない」として明示的に継承（effect.rs L34-37コメント）
- **候補名一覧そのもの**（Glow/Bloom/Displace/Particle Field/Text animators/Transitions等、catalog §4-11）→ CANON.md L15「候補一覧は生存」。Glowは実際に「内蔵vism第1号」（裁定153、compositor effects/mod.rs L2-8のコメント）として実装された
- **画面ではプラグインを選ぶ、という利用者語彙原則**（package-concept §2.1, L46-55）→ 裁定193「Lumetri等はvism＝プラグインで実現する圏」という語彙で生きている
- **`.vism`という名称そのもの**（意味であって実装ではない扱い）→ CANON.md L66「`.vism`名称」は decision-index.md 経由で生存扱い。ただし本文中に実装コードは一切ない（`next/`全体をgrepして0件、CANON.mdの言及1件のみ）

### 死んでいる

- **型付きport連鎖のKitモデル**（provider→Kit解決→consumer、kit-model全体）→ CANON.md L58「裁定72と構造矛盾 — **復元禁止**」と明記
- **lane分類（SINGLE/PORTS/MULTIPASS/BAKE/TEXT/TEMPORAL/SIM/KIT）**（catalog §2, L33-47）→ CANON.md L15「旧trait前提で失効」
- **4trait体系の旧plugin機構**（`plugin-authoring.md`等）→ CANON.md L62「裁定72で置換」、B節の失効リストに明記

### 宙に浮いている（決まったのにnextに無い）

1. **5層identity分離**（package identity/entry identity/Kit identity/Project instance identity/artifact identity、kit-model §10, L288-300）— next/には`plugin_id: String`が1つあるだけで、この5分類に対応する型は皆無。CANON.mdもこの点に言及していない。
2. **Host integration層の一式**（発見/install/update/依存診断/NodeDesc標準UI生成/GPU resource/cache/StateTrack/error/Cancel/accessibility、package-concept §4.3, L102-113）— `EffectPass`という閉じたenumだけがあり、この層の器そのものがまだ存在しない。
3. **`.vism`という配布物理形式**（拡張子/container/manifest/署名、package-concept §9,§11）— コード上は一切の痕跡なし（grep 0件）。CANON.mdは名前だけ生存扱いにしているが、器の設計判断（container形式か単一binaryか等）はどこにも引き継がれていない。
4. **candidate registry・probe発注順序というgovernance構造**（adoption-map全体）— CANON.mdに一言も登場せず、next/の技術選定はこの文書の統治を経由せずに行われている。この文書自体は「決定／調査結果固定」のまま放置されており、失効の裁定も継承の裁定もない。
5. **Vism identity未決6問**（identity-survey）— 「比較中」のまま。CANON.mdにも触れられておらず、next/にも対応する検討の痕跡なし。
6. **BeatEvents等の型付きdata provider→consumer概念**（kit-model §6, L184-220の BPM例）— 裁定70/72で「param全部が既存KeyframeTrackに乗る」設計を採ったため、providerからconsumerへの**型付きカスタムport**という発想自体の居場所がnext/に無い。BPM Rhythm Vismという具体候補（catalog §9, L134）も跡形なし。
7. **第三者生態系の並列実装原則**（catalog §14.1、package-concept §12）— GOALS.mdの除外リストに「動的配布marketplace／第三者SDK／独自plugin UI」と書かれているだけで、これが旧世界の設計原則の**撤回**なのか、単なる**優先度の後回し**なのかを区別する裁定が無い。CANON.mdもここは扱っていない。
8. **vism-param-list-type-decision（2026-08-17）の型・規則**— 旧（egui）世界で「型・規則は実装済み」とされているが、next/の裁定72は独立に別設計（named map方式）へ到達しており、旧決定がnext/へ引き継がれたのか、それとも並行に同じ結論へ別ルートで着地したのかの裁定記録が無い。

## 4. レビュー題名一覧（開いていない・目についた物のみ）

`docs/reviews/*vism*`は約90件（169本の内訳の大半はreviews外の`docs/`本体ファイル群と推定、本回収では未検証）。題名だけ見て気になった物:

- `2026-07-17-vism-a0-plugin-boundary-inventory.md` / `2026-07-17-vism-a0d-contract-migration-ownership-decision.md` / `2026-07-17-vism-a0s-contract-catalog-spec.md` — Contract Catalog系（§1.9の基盤契約の一次資料と思われる、本回収では未読）
- `2026-07-17-vism-ready-counter-review-disposition.md` — 「Vism-ready」と早期宣言する案を止めた記録（lineage recoveryが既に要約済み、§1.9参照）
- `2026-08-02-vism-entrance-parallelization-root-map.md` — adoption-mapが従属する「入口・並列解禁の根本マップ」（adoption-map冒頭L5で参照されている一次文書、本回収では未読）
- `2026-08-21-effect-seam-survey.md` — 裁定153の一次資料（next/側でGlow実装の意思決定根拠として既に参照されている、生きている）
- `2026-08-22-core-vs-vism-classification.md` — next/側の"vism"という語の**現在の使われ方**を示す最重要文書。ここでの"vism"は旧世界の壮大な契約体系ではなく、「普通の動画編集ソフトに不要な専門機能一式（色補正スイート・オーディオスイート・個別エフェクト等）を、後で継ぎ目（プラグイン機構）で載せられる状態として台帳に残す」という**verdictラベル**にまで縮小されている。裁定175/177/193の適用先

## 5. 総括（新案なし・回収漏れの指摘のみ）

- 旧世界のvism契約は**理想としては** CANON.md経由で公式に「まだ拘束する」文書として維持されている（README L60-61）。これは意図的な二層構造であり、「引き継ぎ漏れ」ではなく「実装を意図的に遅延させている」状態である（裁定13が明示的にそう宣言している）。
- ただし、CANON.mdの生死判定は`vism-plugin-catalog.md`・`vism-package-concept.md`・`vism-kit-model.md`の3本にしか及んでおらず、**`vism-known-implementation-adoption-map.md`（本発注の骨の1本）と、identity-survey・param-list-type-decision等の細部決定文書は、CANON.mdの回収作業から漏れている**。これが本発注で見つかった最大の空白であり、§3「宙に浮いている」4・5・8番の根本原因である。
- 第三者生態系原則（§3の7番）がGOALS.mdの除外リストと矛盾するかどうかは、単なる読み落としではなく**裁定が必要な論点**として残っている。

## 6. 逸脱

- 利用者（コーディネーター）の途中指示によりスコープを縮小: 169本精読→骨4本＋歴史回収1本、§2/§3も網羅より速度優先。
- `docs/reviews/`配下の`*vism*`約90件は題名一覧のみで開いていない。深掘りが必要な物として§4に5件だけ名前を挙げた。
- `git merge main`は差分0（すでにmain相当）だったため、実質的なマージ作業は発生していない。
