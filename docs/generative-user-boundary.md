# ジェネラティブ表現とユーザー拡張の境界

作成日: 2026-07-15
状態: **設計決定**。既存の凍結済み公開契約は変更しない。未実装のLive runtime/APIは本書だけを根拠に実装せず、各仕様の解凍手続きとタスク化を先に行う。

## 1. 結論

Motoliiは白紙から素材世界を作るCreative Coding環境や3D DCCではなく、**既存の動画・画像・音声・SVG・3D素材を、時間と解析結果で関係づけて完成映像へ合成するコンポジットツール**である。

四角、楕円、線、星、パス、テキスト等の基本要素は、マスク、オーバーレイ、マット、配置、反復、解析結果の可視化に常用するためHost側に持つ。ただし高度なイラスト制作、モデリング、リギング、彫刻、映画級シミュレーションまで内製しない。複雑な素材はSVG、glTF、画像、動画、ベイク済みシーケンスとして持ち込む。

ユーザー拡張の根本原則は次の通り。

> **どんな表現も「編集時に実体化」「時刻から純関数評価」「宣言した時間参照」「Host所有状態をベイク」「外部で制作して取込」のどれかへ翻訳する。ユーザーに表現を開くが、キャッシュ・状態復元・Undo・書き出し再現性の例外処理は渡さない。**

禁止するのは状態そのものではなく、Hostへ宣言されない状態である。変態的な動きのために純関数契約を黙って破るのではなく、必要なコストと依存を明示した正規経路へ昇格する。

## 2. 製品の境界

### Motoliiが持つもの

- 合成に頻出する基本ShapeとPath、fill/stroke、transform、mask/matte
- 動画・画像・音声・SVG・glTF等の素材取込
- キーフレーム、DataTrack、ParamDriver、型付きLink、Instance/Elementによる時間・関係・反復
- Filter、Composite、LayerSourceによるGPUネイティブな画像処理と補助生成
- lookbehind、Feedback、Simulation等、時間依存をHostが管理する境界
- preview/export一致、Undo、保存、欠落plugin診断、依存・scope・無効化の可視化

### Motoliiが素材制作ソフトとして持たないもの

- Illustrator相当の網羅的な作画・入稿機能
- Blender相当の複数world/camera、collection、constraint、rig、sculpt、重い物理制作環境
- Processing/p5.js相当の暗黙canvasとイベントループを、そのまま製品の時間意味論にすること
- ユーザーコードへ任意のDocument変更、layer名検索、隠れcontroller、前フレームbuffer、OS依存APIを開くこと
- 「何でもできる」ことを理由に、頻出する合成操作をscriptやplugin作者へ放置すること

Blenderが「世界と素材を構築する」側なら、Motoliiは「素材間の関係と時間変化を高速に試し、MVとして完成させる」側に立つ。競争軸は機能総数ではなく、**任意時刻への移動、音・映像への反応、変奏の試行回数、手編集との往復、最終出力の再現性**である。

## 3. 基本ShapeとSVGの分界

見た目の語彙はSVGと重なるが、Documentの正本をSVG文字列やSVG DOMにはしない。正本はMotolii正準座標の**型付き`VectorRecipe`**とする。

**現状の凍結点**: 実装済み`StandardShape`は`Rect { width, height }`と`Ellipse { width, height }`だけである。次表は将来の目標語彙であり、本書を根拠に既存variantの解釈変更や未決field追加をしてはならない。Line/Path/Star/Polygon、corner、fill/stroke等は[backlog GAP-15](backlog.md)で意味論表、追加的schema、migration、golden、UI入口を先に確定する。中心位置は共通transformと重複保存しない方向を第一候補とするが、GAP-15決定前にfield配置を焼かない。

| 目標要素 | 保持したい意味 | 早期に汎用Pathへ潰さない理由 |
|---|---|---|
| Rectangle | size、corner radius | 幅・高さ・角丸を意図単位で編集・駆動するため |
| Ellipse/Circle | radius/diameter | 半径、縦横比、boundsを安定して扱うため |
| Star/Polygon | outer/inner radius、point count、local rotation | 「トゲ数」「尖り」をアニメ・DataTrack駆動するため |
| Line/Polyline/Path | points、handles、closed | mask、stroke、path animation、PathOpへ接続するため |
| Group | children、transform/styleの明示scope | 生成後も通常編集、scope、Undoを維持するため |
| Style | fill、stroke、opacity、blend等の型付き値 | 色変換・premultiplied alphaをHost契約へ揃えるため |

概念上の流れは次で統一する。

```text
基本Shape / ShapeScript / SVG import
                 ↓
       型付きVectorRecipe / Group
                 ↓
       時刻tで評価しGPU描画
```

SVGは次の役割に限定する。

1. **素材の交換形式**: 外部作画ソフトからロゴ・図形を持ち込む。
2. **materialize adapterの入力**: LLMやツールが生成した公開語彙を通常のGroup/VectorRecipeへ変換する。
3. **互換意味の境界**: viewport、左上原点、Y-down、単位、transformを入口で正準座標へ変換する。

SVG runtime、XML、外部参照、script、イベント、ネットワーク取得を保存後の評価意味に残さない。対応外要素は黙って似た絵へfallbackせず、採用/拒否表に基づく型付きエラーにする。import後はSVG engine無しでも保存・再読込・preview・exportが同じ結果になることを審判する。

## 4. 表現を受ける5つの正規経路

| 経路 | 適する表現 | ユーザーが定義するもの | Hostが引き受けるもの | 現在地 |
|---|---|---|---|---|
| **A. Materialize** | 有限個のShape、配置、反復、初期レイアウト | 明示seedと有限生成手順 | 全体preflight、D2 command batch、1 Undo、通常Group化 | v1.x SCR-1〜3 |
| **B. Pure Live `f(t)`** | LFO、wiggle、解析反応、決定論的particle、時刻で直接決まるoverlay | 型付き入力、`t`、seed、paramsから出力を求める関数 | 並列評価、cache、scrub、preview/export | 既存ParamDriver/LayerSource。汎用live scriptはv2判断 |
| **C. Temporal Window** | echo、frame blend、motion blur、非再帰lookbehind | 必要な前後offset/サンプル数の静的宣言 | TimeMap解決、入力texture、cache key、循環拒否 | 口の予約、実装はM4後 |
| **D. Feedback / Simulation Bake** | 非clear蓄積、衝突、流体、布、粒子間相互作用 | 初期条件、固定step、明示seed、step/合成規則 | 状態所有、checkpoint、無効化、再生、scrub、予算 | v1.x SCR-4 / SIM群 |
| **E. External Material** | 高度な作画、モデリング、rig、映画級simulation | 外部ツールで素材を確定 | import、時刻写像、合成、欠落診断 | コア方針 |

選択規則は「最も安い経路から」である。ただし安い経路へ見せかけるため意味を変えてはならない。

- 有限命令ならAへ畳む。
- 任意時刻を直接計算できるならBを使う。
- 他時刻を読むだけならCで依存を宣言する。
- 前出力や逐次状態が意味そのものならDへ送る。
- 合成との関係より素材制作の比重が大きいならEへ送る。

Hostは経路を黙って変更しない。Materialize、Live、Bakeは保存意味・編集可能性・コストが異なるため、自動診断はしても切替はユーザーが確認できる操作にする。

## 5. p5.js型表現を受けるときの翻訳

p5.jsは入口の親しみや公開コーパスに価値がある一方、暗黙canvas、左上原点、pixel単位、`draw()` loop、frame rate、入力event、前frame画素、グローバル乱数を持つ。これらを互換名のままMotoliiへ持ち込むと、正準座標、任意時刻scrub、フレーム並列、preview/export一致と衝突する。

したがってp5.js完全互換を正本にせず、将来p5風syntax sugarを追加する場合も、次の意味へ翻訳できる部分だけを受ける。

| p5.js型の要求 | 翻訳先 | 実装上の扱い |
|---|---|---|
| 1回の実行で有限個の`line/circle/shape`を描く | Materialize | 呼出しを記録し、VectorRecipe/D2 commandへ変換 |
| `t`とseedから毎時刻の配置を計算する | Pure Live | `t`はHostが渡す。frame counterやwall clockを使わない |
| clearせず前frameへ追描きする | MaterializeまたはFeedback | 有限shape履歴へ畳める時だけ実体化。画素履歴が必要なら明示漸化式+checkpoint |
| velocity、衝突、近傍相互作用を蓄積する | Simulation Bake | 固定step、Host所有StateTrack、部分再シム |
| `random()` | 全経路 | Document由来seed+stable identity+channel。OS entropy禁止 |
| mouse/keyboard/camera/network/file | 編集時入力またはAsset import | レンダ時の非決定入力にしない。許可能力を明示しsandboxする |
| `loadPixels()`とCPU pixel loop | Filter/WGSLまたは外部素材 | 製品経路のGPU→CPU同期とCPU frame処理へ落とさない |
| frame rate依存の`deltaTime` | 閉形式または固定step Bake | preview速度を意味入力にしない |

「未対応だから動かない」だけで終わらせず、診断は少なくとも次を区別する。

- `Materialize可能`: 有限命令として通常Shapeへ展開できる。
- `Live純関数が必要`: 時刻入力は必要だが履歴は不要。
- `Feedbackが必要`: 前出力そのものを参照している。
- `Simulationが必要`: 逐次状態や相互作用が意味に含まれる。
- `外部素材化が適切`: Motoliiの合成境界より素材制作側の処理である。
- `禁止能力`: 非決定入力、秘密情報、任意I/O、CPU pixel同期等を要求している。

## 6. ユーザーとHostの責任分担

### ユーザーへ開くもの

- 表現の式、Shape生成、WGSL、plugin実装
- 型付きparamと既定値、明示seed、必要な入力とscope
- Materialize / Live / Bakeの選択
- Bake範囲、品質、再計算の開始と取消
- plugin/version/依存素材を含むRecipeの共有

### Hostに残すもの

- Documentへの書込みと単一writer、Undo/Redo、journal
- 入力参照の解決、循環拒否、cache key、変更時の無効化
- 時刻、TimeMap、固定step、checkpoint、scrub時の復元
- GPU resource、VRAM予算、Draft降格、非同期実行
- 正準座標、色変換、premultiplied alpha、preview/export一致
- sandbox、能力許可、実行時間・命令数・メモリ・nest深度の上限
- plugin欠落・version不一致・unsupported機能の診断
- 失敗時の全体rollback。部分生成や半端なDocument変更を残さない

ユーザーコードへ`try/catch`を許すことと、製品の整合性をユーザーへ任せることは別である。ユーザー側のエラー処理は表現内の局所的な代替に限り、Document整合、GPU資源回収、checkpoint、書き出し再現性の責任移譲には使わない。

## 7. 制作体験

ジェネラティブユーザーを牽引するのはAPIの自由度だけではなく、試行ループの短さと共有可能性である。

```text
素材を置く → 反応/生成を加える → 即preview → seed/paramを変える
          → 音/DataTrackへ接続 → 通常編集と混ぜる → Recipeとして共有
```

将来の編集体験では次を優先する。

- Script/Generator実行前に、入力、seed、出力個数、Materialize/Live/Bake区分が見える。
- Materialize結果は通常Shapeとして選択・編集でき、再実行を強制しない。
- Live表現は値の由来、参照先、現在時刻の評価結果をAdvancedで検査できる。
- Bake対象はbadge、範囲、dirty区間、進捗、推定コスト、取消を見せる。
- 同じseedでvariationを固定し、seed変更で安全に別案を試せる。
- 公開paramをHost生成UIで必ず編集でき、custom UI欠落時も操作不能にしない。
- サンプルは単なるAPI断片でなく、短尺映像、素材、公開param、期待出力を含む再現可能なRecipeにする。
- docsと例題はrepository内Markdown、安定URL、検索可能な静的内容を正とし、独自APIの公開コーパスを意識して育てる。

Materializeの編集時ホットリロードと、Live runtimeの毎frame常駐は分ける。前者は再実行して通常Documentへ置換する開発体験として実現できるが、後者はcache、sandbox、Param Pipeline、保存意味を増やすためv2の別判断である。

## 8. 実装時の主な懸念と解決策

| 懸念 | 壊れるもの | 解決策 |
|---|---|---|
| SVG/Shapeを早期にPathへ平坦化 | 意図単位の編集、param駆動、stable bounds | 基本Shape variantを保持し、必要時だけ評価結果をPath化 |
| SVGをDocument runtimeに残す | 外部参照、単位差、engine差、移行 | import時に検証・正準化しVectorRecipeへmaterialize |
| Generatorへ`&mut Document`を渡す | 単一writer、Undo、部分失敗 | immutable snapshot→typed command batch→全体preflight→1 macro commit |
| 再実行中にDocumentが変わる | stale結果の上書き | 開始revision/snapshotを照合し、staleならcommit拒否 |
| 無限loop・shape爆発・巨大path | UI停止、RAM/VRAM枯渇 | worker分離、時間・command数・点数・nest・出力bytes上限、取消 |
| `Date/random/input/network` | 再現性、cache、export | 能力を既定拒否し、時刻・seed・Asset/DataTrackをHost入力に限定 |
| 前frameをplugin内部へ保持 | seek、並列、cache、preview/export | lookbehind/Feedback/SimulationをHost所有にする |
| Live scriptがCPUでpixel処理 | VRAM常駐、UI応答 | pixel表現はWGSL/texture pluginへ分離。readback APIを渡さない |
| plugin/version欠落 | projectが開かない、無言で違う画 | 読込は保持+診断、preview fallbackは明示、Final exportはstrict |
| DraftとFinalで異なるruntime | 書き出し事故 | 同一意味関数+`Quality`差だけ。近似箇所を明示 |
| custom UIだけに保存paramがある | plugin UI欠落で編集不能 | `NodeDesc`からのHost生成fallbackを必須にする |
| 自動的なLive/Bake昇格 | 意味・コストの不意な変更 | 診断と候補提示は自動、確定は明示操作、Advancedで経路を表示 |
| source/provenanceを早期に必須保存 | runtime固定、migration、再編集意味の未決 | v1 Materializeは生成結果を正本にし、sourceの保存寿命は別判断 |

## 9. 実装ゲートと再現可能な審判

新しいGenerator、script adapter、時間依存pluginを実装する前に、その経路へ応じて以下を完了条件へ割り当てる。

### 全経路共通

1. 同じDocument、時刻、入力、seed、plugin versionで同じ出力になる。
2. previewとexportが同一意味関数を通り、差は`Quality`だけである。
3. 空間値は正準座標、色変換はHostの1箇所、製品経路にCPU frame受渡しがない。
4. unsupported、resource limit、plugin欠落を型付きエラーで区別し、無言fallbackしない。
5. cache keyへ時刻、入力recipe、params、seed、plugin version、時間窓、品質等の全依存を算入する。

### Materialize

1. 1実行=1 Undoで、失敗・取消・stale・上限超過時はDocumentと履歴がbyte同一である。
2. engineを外したsave/reload/exportでも結果が同じである。
3. 同じscript+seedから同じcommand batchが得られる。
4. SVGの座標、transform、style、対応/拒否要素をgoldenで固定する。

### Pure Live / Temporal Window

1. 時刻を順番に再生した結果と、各時刻へランダムseekした結果が一致する。
2. 同じ`t`を複数回・並列順序違いで評価して一致する。
3. 宣言外の時間参照と循環を拒否する。
4. plugin instanceを作り直しても出力が変わらない。

### Feedback / Simulation

1. clip開始からの順再生、直近checkpointからのreplay、Final exportが一致する。
2. param・入力・seed変更で影響時刻以降だけがdirtyになる。
3. 固定stepで、preview fpsやworker数を変えても結果が同じである。
4. 状態予算超過、取消、plugin version不一致から安全に再Bakeできる。
5. checkpointと中間textureはHost所有で、plugin Dropや再生headに意味を依存させない。

## 10. 既存計画への割当

- 基本Shape/VectorRecipe、SVG読込、Vello描画: `concept.md`、M2-D1i-1/D1i-2、M5 PathOp計画、GAP-15
- one-shot Generator/ShapeScript/SVG adapter: M3-U9a〜U9c、backlog SCR-1〜3
- 非clear canvasの蓄積: `plugin-resources.md` F-11、SCR-4
- 純関数pluginと時間軸のはしご: `plugin-authoring.md`、`simulation-model.md`
- Simulation/StateTrack: backlog SIM群、M4 K1/K7後
- live JS/expression/WASM Param Pipeline: v2、PP-Gateと解凍手続きの対象
- plugin UI: GAP-13の採否決定までHost生成fallback以外を公開契約へ足さない

本書は新しい万能script APIの実装許可ではない。新しい表現を要求された時に、既存のどの境界へ置くか、境界が足りなければどの仕様を先に改訂するかを判断するための上位設計である。

## 11. レビュー時の短い判定

新機能・plugin・script提案には次を順に問う。

1. 既存素材との合成・反応・時間編集を強くするか、それとも素材制作ソフトを内蔵しようとしているか。
2. Materialize、Pure Live、Temporal Window、Bake、Externalのどれか。
3. 状態・入力・seed・時間範囲・scopeがHostから見えるか。
4. 任意時刻seek、並列評価、Undo、save/reload、Final exportで意味が変わらないか。
5. 同じrecipeがコミュニティで反復された時、検証済みpreset、first-party plugin、Host primitiveへ昇格できる意味の形か。

1つでも答えられなければ、ユーザーコードへ例外処理を足して進めず、意味文書または依存境界の改訂へ戻る。
