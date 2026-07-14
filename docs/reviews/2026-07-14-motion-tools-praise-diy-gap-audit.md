# モーショングラフィック4ツール 称賛・日曜大工・根本ギャップ監査

日付: 2026-07-14

状態: **先例調査・設計仮説。仕様変更ではない**

対象: After Effects / AviUtl・拡張編集 / Cavalry / Autograph

併読: [レビュー文書の規律](README.md)、[反復再発明の標準化監査](2026-07-14-repeated-wheel-standardization-audit.md)

## 0. 結論

3D・深度・調整スコープ以外にも、各ツールが別々の名前で何度も解いている根本問題がある。最も重要なのは次の2点である。

1. **パラメータの評価を一列に積めない**: キーフレーム、他パラメータへのリンク、手続き生成、相対補正が排他的だと、ユーザーはNull、式、追加レイヤー、スクリプトで「後段」を自作する。
2. **1レイヤーより細かい要素を共通に指せない**: 文字、クローン、シェイプ断片を別々の専用機能で扱うと、文字分解・連番・範囲選択・ランダム化が製品ごとに再発明される。

Autographは1を、Cavalryは2を比較的きれいに解いている。AEは両方の強力な先例を持つが、式・Null・Text Animator・プリコンポという別々の島に分かれている。旧AviUtlは小さなLuaスクリプトで不足を埋めやすい一方、ホストが持つべき導線まで利用者側へ移した。AviUtl2は内部設計と配布方式が変わっているため、旧版の負債をそのまま帰属させない。

Motoliiへの含意は「全部コアへ入れる」ではない。**意味・評価順・可視化・可逆性をコアが持ち、表現の種類はプリセットまたはプラグインで増やす**のが境界である。

## 1. 判定方法と限界

- 「褒められる」は人気投票ではなく、公式契約に一貫した設計上の強みがあり、実制作の反復作業を短くできるものを指す。
- 「日曜大工」は、公式回避策、機能要望、公開スクリプト、アセット市場で同型の補修が繰り返されているものを指す。個人の単発不満だけでは根本問題と判定しない。
- 市場規模が大きいAEは拡張の絶対数も多い。プラグイン数だけで欠陥の大きさを比較しない。
- Autographは2025年のLeft Angle終了後、Maxonから再提供された製品である。若い生態系で日曜大工が少ないことを、完成度の証明には使わない。
- 本文は反例探索前の調査仮説である。凍結済みスキーマへ採用するには、独立した反対側レビューと解凍手続きが要る。

## 2. 4ツールの短評

| ツール | 褒めるべき核 | 利用者が日曜大工している領域 | 根本の教訓 |
|---|---|---|---|
| After Effects | レイヤーと時間の分かりやすい入口、Text Animator、マスク・合成、式、巨大な教材・交換資産 | Nullリグ、相対オフセット、イージング、クローン、文字分解、レイヤー整理、局所調整、3D外注 | 強いプリミティブが式・特殊レイヤー・暗黙スコープへ散り、直接操作から見えない |
| 旧AviUtl + 拡張編集 | 軽さ、無料、小さなオブジェクト＋フィルタ、トラックバーとLuaで素早く拡張 | 導入一式、`patch.aul`、入出力、波形、PSD立ち絵、文字選択、移動・イージング | 拡張可能性とホスト責務の外注は別。必須級補修はコアの欠落 |
| AviUtl2 | 64-bit化、新しい内部設計、D&Dパッケージ、プラグイン・スクリプト導線 | 旧資産の移植・互換確認は継続中 | 旧ABIを永続契約と誤認せず、プロジェクト可読性と拡張の移行を分ける |
| Cavalry | Duplicator、Index/Context、Behaviour、データ駆動、レイヤーUIの裏の型付き接続 | リグ・シーン資産、イージング、SVG橋渡し、学習レシピ、真の3Dは別ツール | 要素ドメインは強い。一方「1属性1入力」でキーと接続が競合する |
| Autograph | 統合2D/3D、GPU、Generator＋Modifier、無限ソース、レスポンシブComp | レイヤー参照の手作業、Illustrator分割書出し、操作導線・学習資産 | 評価列と相対補正は強い。若さと供給主体の変更は別軸のリスク |

## 3. After Effects

### 3.1 褒めるべき部分

AEの最大の資産は、映像を「画面上の物」と「時間上の帯」の対応として入れるレイヤーUI、幅広い合成機能、そして教材・テンプレート・他ツール連携の厚さである。特にText Animatorは、文字をレイヤーへ破壊的に分解せず、文字・単語・行へ複数のAnimatorとSelectorを作用させる。これは後述するElement Domainの成熟した先例である（[Adobe: Animating text](https://helpx.adobe.com/after-effects/using/animating-text.html)）。

式も、全アニメータブルプロパティに「式適用前」と「適用後」の値を持つ。つまりAE内部には既に `base -> expression -> result` がある（[Adobe: Expression basics](https://helpx.adobe.com/after-effects/desktop/work-with-expressions/expression-basics/expression-basics.html)）。問題は能力ではなく、この後段補正をJavaScript文字列として露出したことである。

### 3.2 日曜大工が集中する部分

- **Nullと式による後付け補正**: キー済みの軌跡全体を後から相対移動するだけでも、親Nullか式を追加する。AE自身も位置・パス用Null生成を専用機能として案内している（[Adobe: Create Nulls for Positional Properties and Paths](https://helpx.adobe.com/after-effects/using/create-nulls-for-positional-properties.html)）。
- **グループ不在をプリコンポで代用**: 整理、変形、エフェクト範囲、キャッシュ境界が同じ「プリコンポ」に集中する。レイヤーフォルダ、リンクされたキー群、複数レイヤーだけへのEffectを求める要望が継続する（[Adobe Community: Group in After Effects](https://community.adobe.com/t5/after-effects-ideas/feature-request-group-in-after-effects/idi-p/1214497)、[Linked keyframe groups and layer folders](https://community.adobe.com/t5/after-effects-ideas/feature-request-linked-keyframe-groups-and-layer-folders/idi-p/14257066)）。
- **調整レイヤーの範囲制御**: 「下にある全レイヤー」が暗黙ターゲットなので、除外にはプリコンポ等が要る（[Adobe Community: Exclude a layer from adjustment layer](https://community.adobe.com/t5/after-effects-discussions/exclude-an-layer-from-the-adjustment-layer/td-p/69331)）。
- **クローン・イージング・文字分解・整理コマンド**: 同型のスクリプトが市場で反復される。これは既存の[反復再発明の標準化監査](2026-07-14-repeated-wheel-standardization-audit.md)で個別に整理済みである。

### 3.3 判定

AEの根本問題は「機能がない」より、**同じ操作意味がレイヤー、特殊レイヤー、親子、式、スクリプトへ分裂している**ことにある。MotoliiはAEの入口の分かりやすさとText Animatorの非破壊性は借りるが、式と暗黙隣接スコープは借りない。

## 4. AviUtl / 拡張編集とAviUtl2

### 4.1 旧版で褒めるべき部分

旧AviUtl＋拡張編集は、低い導入コスト、軽い実行、小さなオブジェクトへフィルタを積む理解しやすさ、トラックバーの移動方法を選ぶ即時性が強い。Luaからオブジェクトや効果を操作でき、利用者が小さな表現を自作・共有できた。PSDToolKitのように、立ち絵運用を一つの制作文化へ育てた拡張もある（[PSDToolKit](https://oov.github.io/aviutl_psdtoolkit/)）。

### 4.2 日曜大工が集中する部分

- **環境そのものの組立て**: 本体、拡張編集、入出力、コーデック、補修を利用者が揃える構成になりやすい。`patch.aul`が「必須級」と説明される状況は、表現拡張ではなくホスト品質の外注である（[AviUtl Installer Script: 導入されるもの](https://aviutl-installer.github.io/AviUtl-Installer-Script%E3%81%AB%E3%82%88%E3%81%A3%E3%81%A6%E3%82%A4%E3%83%B3%E3%82%B9%E3%83%88%E3%83%BC%E3%83%AB%E3%81%95%E3%82%8C%E3%82%8B%E3%82%82%E3%81%AE/)）。
- **編集基礎の補修**: 波形表示、RAMプレビュー、入出力、プロジェクト補助がプラグインへ分散する（例: [ShowWaveform](https://github.com/hebiiro/AviUtl-Plugin-ShowWaveform)）。
- **文字の細粒度制御**: 「文字毎に個別オブジェクト」を入口にするため、範囲・奇偶・ランダム選択や全体変形との両立をスクリプトで補う。これはText専用のElement Domainが弱い徴候である。
- **カメラ範囲とオフスクリーン描画**: タイムライン上の範囲や平坦化効果が、スコープとレンダ境界を兼ねる。見えるコンテナではないため、学習した作法に依存する。

### 4.3 AviUtl2を分けて評価する

AviUtl2は旧内部設計を変更しており、旧32-bitプラグインの大半はそのまま互換ではない。プラグイン、スクリプト、編集ウィンドウ、オブジェクト、エフェクトを追加でき、パッケージはD&Dで導入・削除できる（[AviUtl2 Modern Docs: 簡易説明](https://docs.aviutl2.jp/usage)、[Lua API](https://docs.aviutl2.jp/lua/)）。したがって旧版のインストール地獄をAviUtl2の確定的欠陥とはしない。

ただし教訓は残る。**表現拡張を開くこと**と、**編集基礎・互換・診断まで利用者へ渡すこと**は別である。Motoliiでは配布・依存解決・欠落プラグイン表示・プロジェクト読取りをホスト責務に残す。

## 5. Cavalry

### 5.1 褒めるべき部分

Cavalryの核はDuplicatorの個数ではなく、生成要素へIndexを与え、そのContextを位置、回転、色、辺数、ノイズ、時刻など任意属性へ流す設計である（[Cavalry: Context](https://cavalry.studio/docs/getting-started/key-concepts/context/)）。これにより文字・クローン・図形へ同じBehaviourを再利用でき、レイヤー爆発を避けられる。

UIはレイヤー式だが内部はノード接続で、互換型だけを接続できる。AEの文字列式より直接的で、型不一致をUIで拒否できる（[Cavalry: Connections](https://cavalry.studio/docs/getting-started/key-concepts/connections/)）。Duplicator、Stagger、Auto Animate、データ入力の組合せは「1個ずつキーを打たない」モーショングラフィックに強い。

### 5.2 日曜大工と限界

- Sceneryにはシーン、スクリプト、テンプレート、プラグイン、イージング、再利用レイヤーが集積する。これは健全な表現拡張である一方、アセット管理、定番リグ、入出力橋渡しを利用者が標準化している徴候でもある（[Scenery](https://scenery.io/)）。
- 真の3Dは主戦場ではなく、必要な利用者はBlender/C4D等へ渡す。Motoliiの統合2D/3Dとは目標が異なる。
- 最重要の構造制約は、**1属性につき入力は1個で、アニメーションカーブも入力として数える**こと。別入力で上書きするとキーが失われる（[Cavalry: Connections — Inputs and Outputs](https://cavalry.studio/docs/getting-started/key-concepts/connections/#inputs-and-outputs)）。型付き接続は優れていても、評価列が合成可能とは限らない。
- パラメトリック形状を編集可能形状へ変換すると、リンク・階層・アニメーションを保った非破壊更新が難しいという機能要望がある（[Cavalry issue #164](https://gitlab.com/scenegroup-public/cavalry/-/issues/164)）。これは後述するMaterialize契約の問題である。

### 5.3 判定

Cavalryから借りるべきは汎用Index/Contextと直接接続である。借りるべきでないのは「キーもリンクも同じ単入力」という排他性である。MotoliiのText Selector、Clone Index、将来のShape要素は、別々の専用番号ではなく共通の要素文脈へ寄せる価値がある。

## 6. Autograph

### 6.1 褒めるべき部分

Autographは2Dと3Dを同じCompositionへ置き、GPU上で扱うことを製品の中心にしている。OpenUSD、Filament、OpenEXR、OCIO/ACES、Compositionのテクスチャ利用を同じ系に置く（[Maxon: Autograph](https://www.maxon.net/en/autograph)）。3D背景へ2Dキャラクターを置くMotoliiの目標に最も近い比較対象である。

さらに重要なのが全パラメータ共通のGenerator/Modifierモデルである。

```text
manual / keyframes / generator / link
                  ↓
          modifier 1 → modifier 2 → ... → result
```

画像だけでなく数値、テキスト、3DオブジェクトにもGenerator 1個とModifier複数を持てる。公式例では、キー済みPositionへ`Math(Add)`を後段追加し、キーを変更せず軌跡全体を移動する。多くの数値Modifierは逆変換でき、Modifier有効中も元のMotion Pathを編集できる（[Maxon Help: Generators and Modifiers](https://help.maxon.net/ag/en-us/Content/html/Generators_modifiers.html)）。これはAEのNullやMotoliiで検討したCommand+Dragを、恒久的かつ型付きの評価列として一般化した先例である。

Generatorが無限サイズを持てること、Sub-compositionをローカルOverrideできること、Compositionをレスポンシブに扱うことも、固定キャンバス・破壊的複製を避ける強みである（[Maxon Help: Sub-compositions](https://help.maxon.net/ag/en-us/Content/html/Sub_compositions.html)）。

### 6.2 日曜大工と限界

- Layer Imageのような下位レイヤー参照は強いが、参照生成やInspector導線を手作業に感じる余地がある。機能の存在と直接操作性は別である（[Maxon Help: Layer Image](https://help.maxon.net/ag/en-us/Content/html/Generator_layer_image.html)）。
- Illustrator等のレイヤー資産を個別SVGへ分けて運ぶ運用、Graph Editorと空間Motion Pathの理解差、比較的小さい教材・資産圏が残る。若い製品なので反復市場の量はAEと直接比較しない。
- 2025年にLeft Angleが事業を終了し、チームと製品がMaxonへ移った。現在はMaxon版が提供されているが、制作プロジェクトをベンダーの認証・存続へ依存させる危険を示した（[Maxon: Autograph acquisition](https://www.maxon.net/en/article/autograph-acquisition)）。

### 6.3 判定

AutographのGenerator/Modifier列は、今回の調査で最も大きい反例である。ただしTransformだけ評価順が逆になる等、一般列にも例外はある。Motoliiは名称やUIを模倣するのではなく、`Base -> Driver/Link -> Modifier[] -> Result`の意味論、型、評価順、逆変換可能性を先に検証する。

## 7. 横断して見えた根本問題

### G1. Param Pipeline — 後付け補正を第一級にする

現行`ParamSource`は`Const / Keyframes / Data / Vec2Axes`の選択肢であり、基本的に「値の出所」を1つ選ぶ。今後Linkをvariant追加するだけでは、キーとリンク、キーと相対オフセット、DataTrackと手補正が競合する。

必要かもしれない意味は次である。

```text
Base: Const | Keyframes | DataTrack | TypedLink | Generator
  → Modifier[]: Add | Multiply | Clamp | Remap | Noise | ...
  → Result
```

ここで重要なのはModifierの品数ではなく、次の契約である。

- 同じ型だけを接続する。
- 評価順を保存し、UIにも順序を見せる。
- 元値と結果を同時に検査できる。
- Canvas操作が可能なModifierは逆変換を宣言し、できないものは明示的に編集不可とする。
- 一時的なCommand+Dragと、保存される相対オフセットを別コマンドにする。
- 循環参照を拒否し、依存グラフとキャッシュ無効化へ接続する。

これはM1評価器とM2スキーマの凍結面に触る。**現時点では実装しない**。独立先例調査、反対側レビュー、既存プロジェクトの追加的migration、意味論ゴールデンが揃うまでGAPとして扱う。

### G2. Element Domain — 文字・クローン・形状を共通に選ぶ

Text Animator、Cavalry Context、AviUtlの「個別オブジェクト」スクリプトは、すべて「1つのソース内のN要素へ重みを配る」問題である。

```text
Element { stable_id, index, count, group(word/line/path/clone), local_time }
Selector(element, t) -> weight [0, 1]
PropertyDelta × weight -> element result
```

Motoliiの`text-model.md`にはクラスタ・単語・行Selectorがある。これを直ちに汎用化するのではなく、CloneとShapeでも同型が現れるかをスパイクで確認する。安定ID、並べ替え、要素数変化、乱数seed、選択範囲の意味が一致しなければ、無理に1型へ畳まない。

### G3. Scope/Ownership — 何に効くかを構造で見せる

AE調整レイヤー、AviUtlの範囲、Cavalry Behaviour、Autograph Layer Imageはいずれも「入力集合は何か」を扱う。Motoliiでは既にGroup、Effect Scope、Backdrop Surface、明示3D Groupを候補化した。原則は変えない。

- タイムラインの隣接から対象を推論しない。
- 入力集合、処理、出力の3点をInspectorで示す。
- Groupは所有、Scopeは参照、Backdropはその地点の合成済み背景、と意味を分ける。

### G4. Materialize — 手続き生成を編集可能物へ変える契約

Cloneを実体化、文字をレイヤー化、手続き形状をPath化、動作をキーへBakeする操作には共通契約が要る。

- 明示コマンドでのみ実行し、1 Undoにする。
- 見た目、階層、時間範囲、名前をどこまで保持するか宣言する。
- 元とのリンクが切れるか、参照コピーとして残るかを事前表示する。
- 自動Bake/Unbakeで同期し続ける半可逆モデルを作らない。

### G5. Responsive Constraints — 一回の整列と持続する関係を分ける

後から画角、尺、文言、個数が変わるMVでは、Alignコマンドだけでなく「余白を維持」「中央を保つ」「順番に時間配分」の持続制約が欲しくなる。AutographのレスポンシブComposition、CavalryのLayout/Scheduling、AEの式・MOGRTはこの領域を扱う。

ただし持続制約は依存グラフになる。v1では一回の編集コマンドを優先し、保存されるConstraint GraphはParam Pipelineと依存評価の設計後へ延期する。

### G6. Evaluation Visibility — 結果だけでなく由来を見せる

手続き機能が増えるほど「なぜこの値か」が見えなくなる。Inspectorは最低限、`Base → Link/Driver → Modifiers → Result`、有効/無効、入力元、対象Scopeを1箇所で辿れる必要がある。Autographの元値/結果の二重表示、Cavalryの接続Popoverが先例になる。

### G7. Bounds / ROI — 無限表現と効果の広がりを契約にする

Blur、Glow、Shadow、手続き背景、3D投影は元レイヤーの矩形外へ出る。境界が曖昧だと、AEのGrow Boundsやプリコンポ相当の補修が再発する。

- Sourceの論理Bounds、現フレームのVisual Bounds、FilterのPadding宣言を分ける。
- 不明または無限Boundsを表現できる型を持つ。
- DraftのROI最適化がFinalの画を切らない審判を置く。
- Bounds計算のためのGPU同期readbackを導入しない。

### G8. Direct Manipulation — 高度なグラフにも必ず画面上の入口を置く

AEは入口が分かりやすいが高度操作を式へ追い出し、Cavalry/Autographはモデルが強い分だけ配線やInspector探索が増える。Motoliiは同じ意味に3段の入口を持たせる。

1. 直接操作: Command+Drag、Depth Rail、Canvas上のターゲット指定。
2. 標準ツール: 奥行き展開、Clone、Stagger、Range Selector。
3. Advanced: 評価列、明示Scope、AE互換的な深度動作等。

3段は別機能ではなく同じDocument意味の投影でなければならない。

### G9. Extension Supply Chain — プラグインが無くても読める

AEの受渡し時の欠落プラグイン、旧AviUtlの必須級補修、AviUtl2のABI移行、Autographの供給主体変更は、機能正当性と別の問題である。

- ProjectにPlugin ID、契約version、必要capabilityを固定する。
- 欠落時は未知データを保持し、Layer名・Bounds・依存関係を読めるplaceholderにする。
- 使用箇所と代替/平坦化可能性を一覧化する。
- オンライン認証不能でもProject構造の読取りと既存Bakeの再生を妨げない。
- 配布パッケージの署名、ハッシュ、依存診断はホスト責務にする。

### G10. Performance Predictability — 速さを意味論から外さない

AEのRAM Preview、旧AviUtlの補修、長尺・大量要素での各ツールの挙動から、平均速度より「何が再計算されるか分かる」ことが重要である。Motoliiの純関数、VRAM常駐、世代破棄、同一preview/export関数は正しい方向にある。今後のParam Pipeline、Element Domain、Constraintも依存宣言とキャッシュキーを持てないなら採用しない。

## 8. Motoliiでの優先順位

| 優先 | 項目 | 現在の扱い |
|---|---|---|
| Gate | G1 Param Pipeline | `ParamSource`拡張前に独立調査＋反対側レビュー。凍結面なので本文書だけで変更しない |
| Gate | G9 Supply Chain | M2のPlugin参照・未知データ保持・project validationへ審判を割り当てる候補 |
| M3設計入力 | G6 Evaluation Visibility / G8 Direct Manipulation | UIだけの状態を作らず、Document評価の投影としてモック検証 |
| M5設計入力 | G3 Scope / 統合2D・3D / G7 Bounds | 既存の3D・Effect Scopeレビューと統合して反対側レビューへ送る |
| Spike | G2 Element Domain | Text/Clone/Shapeで安定IDとSelector意味が共有できるか最小fixtureで確認 |
| 後段 | G4 Materialize / G5 Constraints | v1は明示Bakeと一回コマンドを優先。自動同期やConstraint Graphは延期 |

## 9. 反対側レビューで潰す質問

1. Autograph型Modifier列を作らず、`Transform.local_offset`等の少数フィールドだけで実用上足りないか。
2. Modifier列が任意グラフの別名になり、評価順・循環・UIを過剰化しないか。
3. Text/Clone/Shapeの要素IDは本当に同じ寿命と並べ替え規則を持つか。
4. Bounds宣言の誤りを、保守的な全画面評価より安全かつ速く検出できるか。
5. Direct / Tool / Advancedの3入口が同じ意味へ正規化されることを、round-trip fixtureで審判できるか。
6. 欠落プラグインplaceholderが「壊れず読める」と「同じ画を再現できる」を混同していないか。

## 10. 最終判定

各ツールの優位性は次のように分解できる。

- **AE**: 入口、合成の広さ、文字、蓄積。
- **AviUtl**: 軽さ、小さな拡張、共有文化。
- **Cavalry**: 要素文脈とプロシージャル反復。
- **Autograph**: 統合2D/3Dと、型付きの生成・後段補正列。

そして日曜大工が増える境界も共通している。

> 「値の由来」「効く範囲」「内部要素」「実体化」「依存資産」のいずれかが、画面から見えず、単一の型付き契約になっていない時、ユーザーはNull、式、追加レイヤー、スクリプト、テンプレートで契約を自作する。

Motoliiの優位性候補は、プラグインの数ではなく、この5つを可視かつ追加的な意味論として持ち、簡易操作とAdvanced操作を同じ世界へ正規化できることである。
