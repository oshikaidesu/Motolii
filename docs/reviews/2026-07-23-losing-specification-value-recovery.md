# 「負けた仕様」の価値回収 — 系譜・理由・再入場条件を失わない処分法（2026-07-23）

状態: **決定**（知識の処分方法と本書の個別判定。Plugin Set / Project Lockの製品仕様は比較中）

対象: 初期設計、後続で一般化・縮小・延期・棄却された案、現行正本から消えた配布構想。

監査範囲: 本書は全歴史corpusの完了宣言ではない。ユーザーが挙げたsingle camera、2.5D、旧Kitを起点に、初期設計2文書と旧plugin ecosystem文書から現行判断へ影響する主張を回収する第一単位である。後続の歴史監査は§2の同じ分類へ追加する。

関連正本: [コンセプト](../concept.md)、[統一カメラ設計](2026-07-14-unified-stage-camera-design.md)、[M2 CompCamera決定](2026-07-16-m2-comp-camera-decision.md)、[M5](../specs/M5-3d-and-post.md)、[Vism / Kitモデル](../vism-kit-model.md)、[Creator / Developer連続体](2026-07-22-creator-developer-continuum-decision.md)

## 1. 結論

「負けた仕様」を一括してarchiveへ送らない。後続で名称や実装案が消えていても、次のいずれかが現行設計を支えているなら、**生き残った主張だけ**を現在の正本から逆引ける形で残す。

1. **現行規範**: 形を変えて現在も守る不変条件。
2. **成立理由**: 現行規範がなぜ必要かを説明する系譜・反面。
3. **再入場候補**: 必要は残るが、公開契約や実装時期が未決の案。
4. **負例**: 再採用を止める具体的な失敗条件。
5. **archiveのみ**: 当時の技術名、推測、暫定スキーマ等で、現在の判断を拘束しないもの。

したがって「採用された文書／負けた文書」ではなく、**主張単位**で処分する。同じ旧文書の中に現行規範、負例、obsoleteな実装詳細が同居してよい。

今回の重要な判定は二つである。

- **single cameraと2.5Dは消えた仕様ではない。** single cameraは「全Compositionに単一`CompCamera`」、2.5Dは「2D/3Dを分けない単一XYZ世界と、Z遮蔽ポリシーの分離」へ一般化された。旧語を製品modeとして復活させず、現行設計の系譜として回収する。
- **旧plugin ecosystemのKitは現行Vism Kitと別概念であり、価値が未回収だった。** 旧名を戻さず、共有する導入環境を仮称`Plugin Set`、作品の厳密再現を`Project Lock`として分離して再入場候補へ戻す。

## 2. 分別規則

### 2.1 文書の勝敗ではなく、主張の変換を追う

典型的な変換は次の五つである。

| 旧案で起きたこと | 現在の扱い | 例 |
|---|---|---|
| 構文は負けたが不変条件は勝った | 現行正本への系譜を残す | `single camera` → 単一`CompCamera` |
| 製品modeは負けたが世界モデルは勝った | 一般化後の規範を優先する | `2.5D mode` → 単一XYZ世界 + 遮蔽ポリシー |
| 実装技術は負けたが責任分離は勝った | 技術名をarchive、境界理由を保持 | Tauri固定 → React/native責任境界 |
| 名前が別概念へ再利用された | 旧名を戻さず役割を改名する | 旧Kit → Plugin Set / Project Lock、現行KitはVism合成 |
| 必要は残るが前提が足りない | 再入場条件つき比較候補にする | hostless discovery / install / set共有 |

### 2.2 回収時に必須の情報

回収する各主張は次を持つ。

- 旧出典の固定Git objectまたはcommitと該当行。
- `現行規範 / 成立理由 / 再入場候補 / 負例 / archiveのみ`の分類。
- 現在の正本または、まだ正本を持たないこと。
- 復活させるものと復活させないもの。
- 再入場候補なら、実装へ進めるための条件とSTOP線。

旧文書を丸ごと現行へコピーしない。未決のfield、拡張子、runtime、UI、技術選定まで権威化してしまうためである。

### 2.3 削除してよいもの

次の全てを満たす主張だけをarchive参照に留め、入口へ戻さない。

- 現行の意味・非目標・テスト・停止線のどれも説明しない。
- 将来案の再入場判断にも使わない。
- 単に当時のlibrary version、推測、仮の型名、暫定UI配置を述べる。
- 消すことで同じ誤りを再発明する可能性が増えない。

## 3. 今回の個別判定

| 旧主張 | 分類 | 判定 | 現在の正本／回収先 |
|---|---|---|---|
| 単一カメラだけを持つ | **現行規範 + 成立理由** | 復活ではなく継続。camera layer、group camera、shot switch、preview cameraを作らない単一`CompCamera`へ昇格済み | [統一カメラ設計](2026-07-14-unified-stage-camera-design.md)、[M2 CompCamera決定](2026-07-16-m2-comp-camera-decision.md) |
| 2D平面と3D meshを同じsceneへ置く2.5D | **現行規範 + 成立理由** | `2.5D`という切替modeは戻さない。全objectが同じXYZ世界にあり、Zは投影へ常時効き、遮蔽だけを`Layer Order / Group Depth / AE-style Bins`で選ぶ形へ一般化済み | [3D深度境界](2026-07-14-3d-depth-scope-design.md)、[M5](../specs/M5-3d-and-post.md) |
| unlitまたは固定1灯、DoFはZ距離post blur | **現行規範 + 成立理由** | Blender型scene editorへ膨らませない縮小線として現行M5に残る。旧camera pose案や光学camera model全体は戻さない | [M5](../specs/M5-3d-and-post.md) |
| 最小Core + pluginで表現を増やす | **現行規範 + 成立理由** | 「本体を空にする」案ではなく、Hostが持続性と共通責任を持ち、作者が表現を増やす境界へ精密化済み | [小さなコア](../extensible-core-model.md)、[Creator / Developer連続体](2026-07-22-creator-developer-continuum-decision.md) |
| 内部は依存graph、UIはlayer/timeline | **現行規範 + 成立理由** | node graphを高度さの必須UIにしない理由として継続。専門generatorが独自recipeを持つ可能性とは分ける | [concept.md](../concept.md)、[小さなコア](../extensible-core-model.md) |
| 明示`FrameDesc`とGPU texture共有 | **現行規範** | width/height/format/color/alpha等を暗黙にしない境界として実装・規範化済み。初期の具体field一覧を新しい公開schemaとして戻さない | [concept.md](../concept.md)、[plugin authoring](../plugin-authoring.md) |
| 重い・再利用可能な結果だけを選択的にcacheする | **現行規範 + 成立理由** | 全frame RAM cache依存を避け、dependency/invalidation/resource lifecycleをHostが所有する理由として継続 | [concept.md](../concept.md)、[小さなコア](../extensible-core-model.md) |
| PreviewとExportを別の利用経路として扱う | **成立理由 + 負例** | realtime schedulingと確定書き出しの要求差は残るが、別の評価意味・別rendererへはしない。現在は同一関数で`Quality`だけを変える規範へ精密化済み | [concept.md](../concept.md)、[AGENTS.md](../../AGENTS.md) |
| pixel/GPUを扱うnative拡張と、構造化dataだけを扱うWASM | **現行規範 + 再入場候補** | 責任分離は現行。WASM/dylibの配布・loader・sandbox実装はv2の別審判であり、初期文言から直実装しない | [concept.md](../concept.md)、[plugin authoring](../plugin-authoring.md) |
| Web UI + native render core | **成立理由** | UIと映像処理を分ける理由は継続。Tauri固定、別surface重ね合わせ、IPC texture転送という当時の具体案はarchiveのみ。現行surface topologyを上書きしない | [UI runtime責任境界](../ui-runtime-architecture.md) |
| Tracery型の解析→生成 | **現行規範 + 再入場候補** | Motoliiを作る理由として残るが、core初期完成条件ではない。DataTrack / ParamDriver / Vismへ分解し、解析producerは後段へ置く | [concept.md](../concept.md) |
| p5.js型の世界生成 | **成立理由 + 再入場候補** | p5.js runtime互換は戻さない。表現を純関数、materialize、時間窓、Host所有simulation/feedback、外部素材へ分類する入力コーパスとして残る | [p5.js表現処分](2026-07-15-p5-generative-pattern-disposition.md) |
| AviUtl2を土台として魔改造する | **負例** | UIやC ABIの先例は参照できるが、解析→生成、3D、cache、API変動をMotolii側で制御できないためHost依存案は戻さない | 本書の歴史出典。現行のAviUtl系観察とは別 |
| vector-first / paper.jsを合成coreにする | **負例 + 成立理由** | vectorは重要な素材・上流表現だが、動画と3Dを合成する最終coreを置き換えない。特定JS library採用はarchiveのみ | [concept.md](../concept.md) |
| GitHub等を作者正本とするhostlessな発見地図 | **再入場候補** | 価値を回収する。Motoliiが中央配布・決済・人気集計を所有しない原則は候補として維持するが、tap schemaやinstall方式は未決 | §4 |
| 作品のplugin再現lockと、人へ渡すplugin一式 | **再入場候補** | 旧Kit名は復活させない。`Project Lock`と`Plugin Set`へ役割を分け、現行Vism Kitとの衝突を解消する | §4 |
| text authority + verify / repair / doctor | **再入場候補 + 成立理由** | cacheや内部DBを唯一の正本にしない回復可能性は、creator/developer連続体を支える価値として回収する。CLI名とhash算法は未決 | §4 |

## 4. 回収する配布構想 — Vism Kitと混ぜない

### 4.1 二つのKitは責任が違う

| 概念 | 何を組むか | いつ効くか | 現在の扱い |
|---|---|---|---|
| **Vism Kit** | Vism、型付き接続、provider、初期値、素材要求 | Projectへ表現をmaterializeする時 | 現行設計原則。名称維持 |
| **Plugin Set**（仮称） | ある制作環境で使うplugin/packageの選択集合と紹介metadata | 他者の環境を「同じ入口」へ揃える時 | 今回再回収した比較候補 |
| **Project Lock**（仮称、旧Plugin Lock） | Projectが実際に要求するidentity、version、source/contentの固定 | Projectを再現・診断する時 | 今回再回収した比較候補 |

Vism Kitは作品内の表現構成、Plugin Set / Project LockはHost環境の導入・再現である。一つのfileやschemaへ統合する決定ではない。

### 4.2 生き残らせる価値

旧`plugin-ecosystem.md`から、次の価値を再入場候補へ戻す。

1. **作者の場所が正本**: openな成果は作者GitHub等、商用成果は外部店とlocal packageが正本。Motoliiは索引、取得、検査、導入を担う。
2. **中央サーバを前提にしない**: 複数の分散indexを購読・マージできる。公開名称`tap`、URL慣例、署名方式、manifestはまだ決めない。
3. **人気を正本にしない**: download数、trend、公式ランキングをHostが集計しない。地図はidentity、kind、tag、更新情報、sourceを示し、推薦の時間軸は外部記事や個人indexへ置ける。
4. **伝播単位を個別packageだけにしない**: creatorが使う一式をPlugin Setとして渡し、受け手は不足分と外部購入が必要なものを確認して揃えられる。
5. **作品再現と推薦を分ける**: Project Lockは再現のため、Plugin Setは人に入口を渡すために使う。
6. **導入状態を観測・修復できる**: source/lock/manifest等のtext authorityから導入状態を再計算でき、cacheは破棄可能にする。失敗を隠さずverify、repair、doctor相当の診断面を持つ。
7. **動的loaderと発見地図を同一タスクにしない**: loaderが未完成でも、first-party・静的登録・source一覧の発見と作者導線は独立して価値を持ち得る。

これは「serverless marketplaceを今すぐ実装する」決定ではない。hostlessは運用費をゼロにする魔法ではなく、availability、source消失、改竄、trust、互換、build失敗を利用者側へ押し付ける危険もある。Hostが中央配布を持たなくても、検証、診断、permission、欠落保持、再現可能性はHost責任として残る。

### 4.3 再入場ゲート

Plugin Set / Project Lock、分散index、install UIの仕様化は、次を順に満たした後に行う。

1. Vism/package identity、version、Host capability、provenanceの語彙が決まる。
2. source、artifact、build、install、runtime loadを別状態として定義する。
3. Lockが何を固定するか（source revision、content hash、artifact、target、commercial local package）をfixtureで比較する。
4. 欠落、改竄、非互換、untrusted code、permission、撤去、作者URL消失のfailure matrixを作る。
5. Project openがnetwork、install、build、任意code実行を暗黙に起こさない[VSM-A0Dの既決境界](2026-07-17-vism-a0d-contract-migration-ownership-decision.md)を維持する。
6. Vism KitとPlugin Set / Project Lockが同じ名称、拡張子、parser、Document fieldを奪い合わない。
7. 中央serverなしでの更新・失効・署名・mirror・商用導線を実fixtureで反証する。

いずれかが未決なら、公開manifest、lock schema、拡張子、tap schema、install APIを実装しない。

## 5. 後続への運用

今後、歴史監査で価値ある旧主張を見つけた時は、次の順で処理する。

```text
旧主張を固定Git objectで特定
  → 現行正本に同じ意味があるか照合
    → 同じなら系譜だけ接続
    → 一般化済みなら旧語を戻さず変換を記録
    → 必要だけ残るなら再入場条件とSTOP線を記録
    → 再発防止だけ残るなら負例へ送る
    → 何も残らなければarchive参照のみ
```

回収文書が新しい製品仕様を発明してはならない。公開API、plugin契約、Document、永続形式、UIの所有を変える場合は、対象正本の独立した仕様改訂へ分ける。

## 6. 固定歴史出典

今回の判定で用いた初期資料は次で固定する。

| Git object / command | 回収した範囲 |
|---|---|
| `git cat-file -p ac3cda40eb8dd92523c25a5b3c7b1926f1334c3c` | `discussion-log-2026-07-06.md` blob。22–26行=内部graphと簡易UI、28–40行=AviUtl2依存棄却、42–74行=最小Core・native/WASM・FrameDesc・Web UI/native core、76–80行=2.5D・single camera |
| `git cat-file -p 4b8e1e6ccf6025e586da6eeffdd5d8faec025c73` | `design-memo.md` blob。27–49行=動画/3Dゆえのpixel合成と解析→生成→post、73–94行=Web UI/native core、98–111行=選択的cache、127–134行=初期工程 |
| `git show 2cbfc813d0db5f258d31bb4a83eb3ac759d60285:docs/plugin-ecosystem.md` | 1–35行=hostless原則、37–173行=look/primitive・set伝播・分散推薦、394–517行=D&D・lock・旧Kit、543–604行=text authority・verify/repair/doctor・dynamic loadingと発見地図の分離、610–622行=未決事項 |

これらは歴史証拠であり現行正本ではない。本書の処分とリンク先の現行文書を経ず、旧field、拡張子、技術名、工程表を実装根拠に使わない。
