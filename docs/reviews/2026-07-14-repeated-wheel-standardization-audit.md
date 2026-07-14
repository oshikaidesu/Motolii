# AE反復再発明プラグイン標準化監査(2026-07-14)

状態: **設計レビュー。仕様変更は未承認**

併読:

- [レビュー文書の規律](README.md)
- [3D深度スコープ設計](2026-07-14-3d-depth-scope-design.md)
- [M3 UI統合仕様](../specs/M3-ui-integration.md)
- [M5 3D・ポスト仕様](../specs/M5-3d-and-post.md)

## 1. 目的

After Effectsでは、同じ不便を埋めるプラグインが世代をまたいで何度も作られている。本監査は、その反復を次の4種に分け、Motoliiが標準装備すべき**最小プリミティブ**を抽出する。

1. **欠けた基本操作**: コアまたは標準UIの候補
2. **AEの構造に対する回避策**: 需要は認めるが、回避策の形は移植しない
3. **創作効果**: 標準化せずプラグイン生態系へ残す
4. **作業嗜好**: Documentへ焼かずUser settingsまたはWorkspace-sessionへ置く

プラグイン数は需要の証拠にはなるが、設計の正しさの証拠ではない。巨大な万能パネルを模倣せず、反復している最小の意味を採る。

## 2. 調査方法と証拠強度

製品の公開ページに、解く対象と操作が明記されたものだけを数えた。レビュー、売上、作者の主張は採否の根拠にしない。

- **強**: 独立製品3本以上が同じ基本操作を解く。または兄弟Adobe製品の標準機能と複数のAE製品が一致する
- **中**: 独立製品2本が同じ不足を明示する、または長期間にわたり同じ回避策が再登場する
- **弱**: 多機能製品の隣接機能だけで、独立した反復とまでは言えない

この文書は先例調査であり、反例探索と反対側レビューを通すまで仕様根拠に昇格させない。

## 3. 結論一覧

| 反復クラスタ | 強度 | 分類 | Motoliiでの判定 |
|---|---:|---|---|
| レイヤー/キーの時間差展開 | 強 | 欠けた基本操作 | **標準UI候補**。Timing Railを詳細化 |
| グループ/プリコンポ代替 | 強 | AE構造の回避策 | **既存Groupで吸収**。AE式の補修機構は移植しない |
| 文字の文字/単語/行分解 | 強 | 基本操作+AE回避策 | **アドレス可能範囲を標準化**。大量レイヤー化は任意 |
| イージング編集/複製 | 強 | 欠けた基本操作 | **既存方針を維持**。式ではなく区間イージング |
| アンカー/整列/均等配置 | 強 | 欠けた基本操作 | **M3標準UI候補**。参照枠を明示 |
| プリコンポのクロップ/境界 | 強 | AE構造の回避策を含む | **境界意味論を監査**。固定キャンバス補修は移植しない |
| 検索/絞り込み/選択 | 中〜強 | 欠けた基本操作+作業嗜好 | **M3標準UI候補**。状態はWorkspace-session |
| パス一括処理/モーフ補助 | 中 | 基本操作+創作効果 | **基本的な一括操作だけ監査**。専用モーフはプラグイン |
| プロジェクト自動整理 | 弱〜中 | 作業嗜好 | **保留**。不可逆な自動整理をコア化しない |

## 4. R1: Timing Rail / 時間差展開

### 4.1 反復している需要

- [Rift](https://aescripts.com/rift/): レイヤーの開始/終了、キー、マーカーをshift、sequence、stagger、randomizeする
- [Staircase](https://aescripts.com/staircase/): レイヤーブロックを順序付け、時間差配置し、空間位置やラベルも順序に使う
- [pt_ShiftLayers](https://aescripts.com/pt_shiftlayers/): 選択順に固定フレーム/秒でレイヤーをずらす
- [Stratify](https://aescripts.com/stratify/): 昇順、降順、ランダム、選択順、位置パターン等で時間差配置する
- [Keystone 3](https://aescripts.com/keystone/): レイヤー/キーの整列、時間差、伸縮、Bezier分布を扱う
- [Expression Kit](https://aescripts.com/expression-kit/): 式ベースのstaggerを提供する

製品ごとの差は大きいが、「複数要素の時刻差を可視化し、順序と間隔をまとめて編集する」という核は一致する。これはDepth Railの奥行き差と同型である。

### 4.2 一語に混ぜてはいけない3操作

| 操作 | 変える値 | 保つもの | 典型用途 |
|---|---|---|---|
| **レイヤー区間移動** | in/outまたは配置開始時刻 | 長さ、内部キー間隔、素材再生速度 | カットや文字を順番に出す |
| **選択キー移動** | 選択キーの時刻 | 未選択キー、レイヤー区間、値 | 動きの開始だけ順番にずらす |
| **素材リタイム** | source time写像 | レイヤーの配置枠 | 速度変更、時間伸縮 |

初期実装は上2つを**別モード**で提供し、素材リタイムを含めない。「Shift」「Stagger」だけの曖昧なコマンドは作らない。

### 4.3 UI案

Timing Railは選択中要素の**評価済み時間オフセット**を一時的なレールとしてCanvas/Timeline上に表示する。DocumentにRailオブジェクトは保存しない。

- 各ノードは対象レイヤーまたは選択キーを表す
- 横位置は実時刻または基準からのオフセットを表す
- 両端ハンドルは全体レンジを拡大/圧縮する
- ノードのドラッグは個別オフセットを変更する
- 中央ドラッグは全体を平行移動する
- Reverseは順番を反転し、時刻レンジを保つ
- Flattenは同時刻へ集約する
- Distributeは両端を固定し、中間を等間隔にする
- Step入力は隣接差を固定する
- Randomizeはseed付きで結果をプレビューし、確定時に通常の時刻値へ焼く

Depth Railと共有するのは「ノード、範囲、分布、反転、ライブプレビュー、確定/取消」の操作文法だけである。DepthとTimingを同じ永続型や同じ軸値に抽象化しない。

### 4.4 順序モード

初期候補:

1. タイムライン順
2. 選択順
3. 現在時刻順
4. 空間X / Y順
5. 距離または放射順
6. seed付きランダム

親子鎖、ラベル、名前自然順は需要を再検証してAdvancedへ置く。既定はタイムライン順とし、選択順はUIイベント列をDocumentへ保存せず、操作開始時の対象列としてだけ使う。

### 4.5 不変条件

- レイヤー区間モードでは各レイヤーのdurationと内部キーの相対差を保つ
- キーモードではレイヤーin/outと未選択キーを変えない
- 同値の順序は安定で、タイブレークはDocument内の正準順序とする
- 全時刻は既存の`RationalTime`で表し、浮動小数の累積ずれを入れない
- Randomizeは同じ入力とseedで同じ結果になる
- ドラッグ中は一時プレビュー、pointer-upでD2 macro 1件、取消なら履歴0件
- 再生中もviewportを跳ねさせず、playheadは派生表示に留める
- controller、expression、hidden null、複製レイヤーを生成しない
- 1要素だけの操作、ゼロ幅レンジ、同時刻、負時刻を型付き結果として定義する

### 4.6 審判案

1. 3レイヤーを`0, 2, 5`フレームへ配置し、Distribute後が`0, 2.5, 5`相当の正確な有理時刻になる
2. Reverseを2回適用すると元の時刻列へ戻る
3. レイヤー区間移動前後でdurationと全内部キー差が一致する
4. キーモード前後でレイヤー区間と未選択キーがbit-for-bit一致する
5. 同一seedのRandomizeがUndo/Redo、preview/export、再起動後に一致する
6. 1回のドラッグがUndo 1回で完全に戻り、取消はDocument revisionを増やさない
7. 3D不使用コンポとRail未使用コンポは既存ゴールデンとpixel-identicalである

### 4.7 暫定判定

**採用候補**。ただしM3仕様へ今すぐ恒久APIを足さない。D2 macro/merge、選択順の一時状態、Timeline編集契約が揃った後に、仮称`M3-U3e`として仕様改訂する。これは確定チケットIDではない。

## 5. R2: Group / Precomp代替

### 5.1 先例

- [Free Compose](https://aescripts.com/free-compose/)
- [Layer Groups 2](https://aescripts.com/after-effects/automation/layer-groups/)
- [Squirrel](https://aescripts.com/squirrel/)
- [Groups and Toggles](https://aescripts.com/groups-and-toggles/)
- [Layer Slayer](https://aescripts.com/layer-slayer/)
- [Control LMFX](https://aescripts.com/control-lmfx/)

AEでは、折り畳み、まとめて選択、表示切替、プリコンポ内外の往来、制御レイヤーの生成が別々の製品として再発明されている。

### 5.2 Motolii判定

Motoliiの再帰的Groupは既に、所有、折り畳み、合成境界、Depth scopeの明示先を1つの構造で表せる。したがって採るのはGroupの操作性であり、次は採らない。

- プリコンポのbake/unbake/refreshを往復する補修機構
- null/controller/expressionを自動生成する擬似グループ
- ツール実行だけで暗黙Groupを増やす動作
- Group外の隣接レイヤーへ意味が波及する規則

Depth Rail、Timing Rail、文字分解のいずれも、Group作成を必須の副作用にしない。Groupはユーザーが意味上の所有や合成境界を必要とした時だけ作る。

## 6. R3: Text addressability / 文字範囲の操作

### 6.1 先例

- [TextExploder](https://aescripts.com/textexploder/)
- [DecomposeText](https://aescripts.com/decomposetext/)
- [Type](https://aescripts.com/type)
- [Dojo Text Generator](https://aescripts.com/after-effects/automation/text/dojo-text-generator/)

文字、単語、行ごとのアニメーション需要に対し、「テキストを多数のレイヤーへ分解する」が繰り返されている。これはアドレス可能な文字範囲が弱いことへの回避策でもある。

### 6.2 Motolii判定

第一選択は大量レイヤー化ではなく、標準Text/Lyricsプラグイン内で次を選択可能にすること。

- grapheme cluster
- shaped cluster
- word
- line
- 明示range

「character」をUnicode scalarや単一glyphと同一視しない。合字、結合文字、縦書き、ルビを壊さず、M5 P6のcluster mappingを正本にする。

レイヤーへのmaterializeは将来の明示コマンドとして残せるが、次を満たす必要がある。

- 元の組版位置とtransformを保つ
- 隠れた全文コピーやexpression依存を作らない
- 作成物を自動Group化するかは別選択とし、既定でGroupを増やさない
- 1コマンド=1履歴で完全に戻せる

## 7. R4: Easing / Keyframe UX

### 7.1 先例

- [Flow](https://aescripts.com/flow/)
- [Ease and Wizz](https://aescripts.com/ease-and-wizz/)
- [AccelCurve](https://aescripts.com/accelcurve/)
- [animationPATTERNSpro](https://aescripts.com/animationpatternspro/)
- [Keysmith](https://aescripts.com/keysmith/)

### 7.2 Motolii判定

既存の区間イージングpopoverと、Bounce/Elastic/Stepsをパラメトリックな補間として持つ方針は維持する。AE式を標準データモデルへ持ち込まない。

追加監査が必要なのは次の小操作である。

- 値や時刻を上書きせず、easingだけをcopy/pasteする
- 選択区間へ一括適用する
- curve presetをUser settingsとして保存/共有する
- 選択区間間で正規化した曲線を再利用する

値グラフの追加はしない。必要性が別途立証されるまで、速度/補間のUIと値編集を混ぜない。

## 8. R5: Anchor / Align / Distribute

### 8.1 先例

- [Anchor SNIPER](https://aescripts.com/anchor-sniper/)
- [Precomp Anchor Repo](https://aescripts.com/precomp-anchor-repo/)
- [Align Pro](https://aescripts.com/align-pro/)
- [Align3D](https://aescripts.com/after-effects/3d/align-3d/)
- [DistributeLayers](https://aescripts.com/distributelayers/)
- [Match Position](https://aescripts.com/match-position/)
- [Power Null](https://aescripts.com/power-null/)

### 8.2 必要な意味の分離

- anchorを3x3または任意点へ移す
- オブジェクトを別の参照枠へalignする
- 中心間またはedge gapをdistributeする
- 位置だけを一致させるか、回転/scaleも含めるか

参照先はselection bounds、key object、frame/comp、parentから明示選択する。「見た目の境界」と「幾何/宣言境界」も区別する。

anchor変更時はworld appearanceと既存animationを保つD2 macroにする。精密なalpha boundsをライブUIのGPU readbackで求めない。宣言境界で不正確な場合は、構造化された診断を出す。

暫定的に`M3-U6a`候補とするが、確定IDではない。

## 9. R6: Bounds / Crop

### 9.1 先例

- [Auto Crop 3](https://aescripts.com/auto-crop/)
- [pt_CropPrecomps](https://aescripts.com/pt_cropprecomps/)
- [Cut'n'Pack](https://aescripts.com/cut-n-pack/)
- [Precomp Anchor Repo](https://aescripts.com/precomp-anchor-repo/)
- [TweiNa](https://aescripts.com/tweina/)

AEの固定サイズprecompとcollapse transformの複雑さが、このクラスタを増幅している。MotoliiはGroupを固定キャンバスへ再サンプルする前提を置かないため、同じ補修UIは不要である。

一方で、次の境界意味論は未監査である。

- imported assetのlogical boundsとvisible bounds
- canonical unitでのpadding
- animated duration全体のunion boundsか、現在フレームboundsか
- alpha boundsが必要な場合の非同期derived cache

alpha走査結果をDocumentへ焼かず、UIスレッドで同期readbackしない。所有チケットを確定してから仕様改訂する。

## 10. R7: Search / Filter / Selection

### 10.1 先例

- [Smart Selector](https://aescripts.com/smart-selector/)
- [Shy Bar](https://aescripts.com/shy-bar/)
- [Squirrel](https://aescripts.com/squirrel/)
- [Command Frame](https://aescripts.com/command-frame/)
- [Control LMFX](https://aescripts.com/control-lmfx/)

### 10.2 Motolii判定

標準UI候補:

- 透明部分や重なりを含むCanvas hit-testの候補cycling
- name、type、effect、keyframe有無、error、selectionによるTimeline filter
- command、layer、compositionのfuzzy search
- 検索開始/終了で展開行、scroll、selectionを不用意に失わない

query、filter、expanded rows、直前の検索履歴はWorkspace-sessionまたはUser settingsであり、Documentへ保存しない。永続的な隠れグループDBを作らない。仮称`M3-U3f`候補とするが、確定IDではない。

## 11. R8: Path / Shape batch operations

### 11.1 先例

- [PathPrep](https://aescripts.com/pathprep/)
- [PastePath](https://aescripts.com/pastepath/)
- [Vertex Tool](https://aescripts.com/after-effects/automation/vertex-tool/)
- [Set Path Keyframes](https://aescripts.com/set-path-keyframes/)
- [Hooker](https://aescripts.com/hooker/)
- [Super Morphings](https://aescripts.com/super-morphings/)
- [Tweaks](https://aescripts.com/tweaks/)

このクラスタは同一操作の反復というより、AEのpath API/選択UIの弱さへ多方向から対処しているため証拠は中とする。

監査候補は、複数pathへのkey追加、transformとtimingを保つcopy/paste、parametric shapeのBezier化、複数vertexの基本一括編集である。特殊なmorph対応やリギングは標準化せず、既存のpath operator/プラグイン境界に残す。

## 12. 仕様へ進める順序

| 優先 | 変更候補 | 先に必要なもの | 現時点の扱い |
|---:|---|---|---|
| 1 | Timing Rail | D2 macro/merge、Timeline編集契約、反対側レビュー | 詳細案まで記録 |
| 2 | Text addressable spans | M5 P6 cluster mappingの審判 | first-party plugin要件候補 |
| 3 | Anchor/Align/Distribute | M3 selection/bounds契約 | UI要件候補 |
| 4 | Search/Filter/Selection | Workspace-session所有決定 | UI要件候補 |
| 5 | Import/animated bounds | importとderived cacheの所有決定 | 監査待ち |
| 6 | Path batch operations | shape/path選択モデル | 監査待ち |

ここに挙げた`M3-U3e`等はレビュー内の仮称であり、仕様書の確定タスクIDではない。未決の所有境界を、それらしいデフォルトで埋めない。

## 13. 非目標

- aescripts製品を網羅した市場一覧を作ること
- 製品数を根拠に機能を無条件採用すること
- AE互換のexpression、null、controller、precomp補修を再現すること
- Rail自体をDocumentオブジェクトとして永続化すること
- 全操作を1つのAdvancedモードへ詰め込むこと
- 高価なalpha boundsやpixel readbackをライブUIの必須経路にすること
- specialized creative effectを標準機能へ吸収すること

## 14. 反対側レビューで潰す問い

1. Timing Railは既存Timeline複数選択と数値入力だけで十分ではないか
2. 空間順の時間差展開は、camera移動中にどの時刻の評価位置を使うべきか
3. Text rangeのアドレスは再組版後も安定して参照できるか
4. visual boundsを持たないことで、整列品質が実用上不足しないか
5. 検索/絞り込みを標準化すると、大規模Documentでindexの恒久契約が必要にならないか
6. 同じ操作文法をDepth/Timingで共有することが、軸固有の違いを隠さないか

これらを通過した項目だけを、採用/縮小/延期/棄却の判定語付きで仕様へ移す。
