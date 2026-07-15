# AE反復再発明プラグイン標準化監査(2026-07-14)

状態: **設計レビュー。仕様変更は未承認**。Relative Move、Effect Link、Duplicator/seedの採否は[2026-07-15決定](2026-07-15-relative-scope-duplicator-decision.md)を優先する。

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
| Null制御のクローン/エフェクター | 強 | 欠けた基本操作+AE回避策 | **Cloner/Effectorを一級化候補**。万能Nullは移植しない |
| アニメーション軌跡の相対移動 | 強 | 隠れた基本操作 | **標準Canvas操作候補**。Nullを作らず全PositionへΔを加える |
| Effect/Adjustment Layer | 強 | layer比喩の過積載 | **明示Effect Scope候補**。非描画Effectをvisual layerにしない |
| プロジェクト自動整理 | 弱〜中 | 作業嗜好 | **保留**。不可逆な自動整理をコア化しない |

### 3.1 「AEではプラグイン」の転記先

本監査はAEプラグイン市場を需要の観測点に使ったが、Motoliiでの実装先をプラグインに限定しない。AEでプラグインとして売られていることは、AE本体の拡張可能面へ押し込まれた結果でもある。

責務の判定規則:

- **Host根本**: 他objectの評価順、合成順、depth、所有、参照解決、Undo、永続意味を変える
- **Host UI/Command**: Documentの既存値を安全に一括編集するが、render能力は増やさない
- **First-party plugin**: 公開契約の実地検証を兼ねる標準表現。局所的な`f(input, params, t)`として交換可能
- **Third-party plugin**: 同じ契約上の追加layout、effect、generator、creative variation

プラグインが他layerを勝手に列挙し、隠れlayerを生成し、framebufferを任意位置でcaptureし、合成順を変更する設計にはしない。必要能力はHostが型付き境界として所有し、プラグインは宣言して利用する。

### 3.2 今回の責務配置

| 機能 | Hostが必ず持つ根本 | 標準plugin/UIへ置けるもの |
|---|---|---|
| **3D** | 共通world、Transform、CompCamera、projection、depth policy、参加scope、object/world/camera表現、GPU pass | mesh generator、material、3D effect、importer |
| **Depth Rail** | Position Z、Edit Space、D2 command、depth policyとの整合 | なし。Host Canvas tool |
| **Relative Move** | Transform/Keyframe意味、座標変換、Undo | なし。Host UI command |
| **Timing Rail** | RationalTime、layer/key編集、Undo、selection snapshot | なし。Host UI command |
| **Group** | 所有、再帰、合成境界、transform、scope | Group向けpresetや自動配置tool |
| **Effect Scope** | target routing、処理stage、flatten boundary、interval、循環拒否 | 個々のBlur/Color/Distort等 |
| **Backdrop Surface** | 直前compositeの安全なGPU入力、capture stage、dependency、padding | `Copy Background`相当のfirst-party Effectと後続Effect群 |
| **Cloner** | instance評価/描画契約、stable order、source参照、depth参加、循環拒否 | Linear/Grid/Radial layout、Noise/Step等のgenerator/effector |
| **Effector/Field** | 型付き接続、space、target可視化、評価順 | field shape、falloff、creative modifier |
| **Text** | shaping、font fallback、cluster mapping、glyph描画 | Lyrics/Text generator、文字range animator |
| **Easing** | Interpの評価意味と決定論 | curve preset、UI library |
| **Align/Search** | selection、bounds、workspace state、D2 command | なし。Host UI |

特に3Dはplugin領域へ閉じられない。RGBA化後のEffectとして3Dを実装すると、layer間交差、depth、camera、shadow、Backdropとの処理段を正しく共有できない。M5の共有world/depth参加境界がHost根本で、pluginはその境界へobject/material/generatorを供給する側である。

ClonerとBackdropも同様に二層へ分ける。製品として見える`Grid Cloner`や`Copy Background`はfirst-party pluginにできるが、instanceを通常objectと同じdepthへ参加させる能力や、直前compositeをGPU上で安全に読む能力はHost契約でなければならない。

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

## 12. R9: Cloner / EffectorとNull制御

### 12.1 反復している需要

- [Cloners + Effectors](https://aescripts.com/cloners-plus-effectors/): 複数レイヤーをlinear、radial、grid、path等へ配置し、falloff付きeffectorでまとめて変形する。2016年から継続している
- [React 2](https://aescripts.com/react/): repeaterとeffectorをtoolbar化し、grid、radial、linear配置と複数modifierを提供する
- [Easy Clones 2](https://aescripts.com/easy-clones/): Clone Control Layerからposition、scale、rotation、opacity、delay、wiggle等を制御する
- [xCloner](https://aescripts.com/xcloner/): effect plugin内で複製し、expression/実レイヤー複製より高速な経路を提供する
- [Dupli-Kit](https://aescripts.com/dupli-kit/): patternに沿ったレイヤー複製を簡略化する
- [Moglyph FX](https://aescripts.com/moglyph-fx/): glyph/textにclonerとeffectorの操作体系を持ち込む
- [Power Null](https://aescripts.com/power-null/): viewport上のクリック、snap、parenting、property linkによってNull作成自体を直接操作化する
- [Create Pivotal Null](https://aescripts.com/create-pivotal-null/): layer bounds上の位置を指定してNull作成とparentingを自動化する
- [Pinna](https://aescripts.com/pinna/): path pointとNull controller/followerの対応付けを可視的に作る

この反復は2つの不足が重なっている。

1. **複製集合が一級オブジェクトでない**ため、大量レイヤー、expression、effect内の閉じた画像のいずれかへ逃げる
2. **Nullが直接操作できる意味を持たない**ため、位置合わせ、親子付け、制御対象との接続を別ツールで補う

兄弟領域では、Cinema 4DがCloner、Effector、Fieldを別概念として持ち、Unreal Engine Motion Designも[Cloner ActorとEffector Actor](https://dev.epicgames.com/documentation/en-us/unreal-engine/motion-design-cloners-and-effectors-in-unreal-engine)を一級化している。UnrealのEffectorはviewport境界、影響範囲、falloff、複数effector接続を明示する。したがってNull制御は唯一の自然なモデルではなく、AEのlayer/expression境界に合わせた回避形である。

### 12.2 Nullが直感的でない理由

AEのNullは単純な空オブジェクトに見えるが、実際には次を兼務する。

| Nullへ載せられる役割 | ユーザーが本当に操作したいもの | 問題 |
|---|---|---|
| 空のtransform | 複数要素の共通座標系 | 不可視レイヤーを選ばないと操作できない |
| parent | 所有/追従関係 | Timeline上の親子と意味上のGroupが分離する |
| Effect Controlsのslider | 名前付き共有parameter | 空間オブジェクトがparameter panelを兼務する |
| expressionの参照先 | 依存関係 | 接続、scope、破損がCanvasから見えない |
| falloff中心 | 影響範囲を持つEffector | 範囲と強度がNull自身の見た目に現れない |
| 選択proxy | rigの操作ハンドル | 表示、描画、選択、書き出しの区別が曖昧になる |

問題は「Nullという名前」ではなく、**異なる意味を不可視レイヤー1種へ畳んだこと**である。Null作成を1クリックにしても、接続後の意味と影響範囲が読めなければ根本解決にはならない。

### 12.3 Motoliiでの役割分離案

| 意図 | 第一選択 | Timeline/Canvasでの見え方 |
|---|---|---|
| 複数要素を同じ座標系で動かす | **Group transform** | 既存Groupを選択し、通常のtransform gizmoを表示 |
| 規則的に複製する | **Cloner** | 1行の生成ノード+全instanceの直接プレビュー |
| 空間範囲で値を変える | **Effector** | 境界、中心、falloffをCanvas gizmoで常時識別可能 |
| rigを直接掴む | **Control handle**候補 | editor-onlyの名前付きhandle。映像レイヤーにしない |
| 共有値を公開する | **Group/pluginのnamed parameter** | 所有者のInspectorへ置き、空オブジェクトへsliderを載せない |

Control handleは永続的なrig参照を持ち得るため、現時点ではスキーマへ追加しない。必要性、所有、削除時の参照整合、プラグインからの宣言方法を別レビューする。少なくとも「何でもNullで解く」を標準作法にはしない。

### 12.4 Clonerの最小意味

Clonerは素材を複製したTimelineレイヤーの束ではなく、次の入力を持つ純関数的な生成ノード候補とする。

- source: 1個以上のvisual objectまたはGroup
- layout: linear / grid / radial。pathは次段
- countとlayout parameters
- source選択順とinstance indexの決定規則
- per-step transform、opacity、color、time offset
- seed付きvariation
- ordered effector list

評価の概念形は次である。

```text
instance(i, t) = evaluate_source(select_source(i), t + time_offset(i))
                 |> layout_transform(i)
                 |> ordered_effectors(i, t)
```

状態を隠した逐次シミュレーションにしない。delay、noise、spring風の動きはまず`f(i, t, seed)`で表し、本物の履歴依存が必要な場合だけsimulation-modelのベイク境界へ送る。

### 12.5 3つの出力形を混ぜない

| 形 | 特徴 | 用途 |
|---|---|---|
| **Procedural instances** | 1つのClonerとして高速評価。個別編集なし | 大量反復、背景、モーショングラフィックス |
| **Linked clones** | 個別overrideを保持しつつsource共有 | 将来候補。identity設計が必要 |
| **Materialized layers** | 通常の独立オブジェクトへ確定 | 破壊的な個別編集、手仕上げ |

初期候補はProcedural instancesと、明示的なMaterializeだけに絞る。bake/unbake/refreshの往復契約は作らない。Linked clonesは、countや並べ替え後もoverrideのidentityを保てる仕様が立つまで延期する。

### 12.6 Effectorの最小意味

Effectorは「Nullの位置をexpressionで読む」仕掛けではなく、影響関数を持つ明示オブジェクト候補とする。

- shape: unbound / sphere-circle / box / linear plane
- space: world / cloner-localを明示
- inner/outer boundaryとfalloff curve
- effect: position / rotation / scale / opacity。colorとtimeは次段でもよい
- combine orderとblend operationをInspectorのlistで明示
- 1つのEffectorを複数Clonerへ接続可能。ただし接続先をCanvas/Object treeから辿れる
- gizmoは色だけに依存せず、境界線、形、labelで種類と選択状態を識別できる

Advancedではcustom numeric propertyへの接続を選べる余地を残すが、property pathを文字列expressionとして保存しない。型付きparameter IDと互換性診断が必要である。

### 12.7 2D/3Dと深度の整合

Clonerは2D専用の画像effectに閉じない。各instanceは共通world内でX/Y/Zを持ち、[3D深度スコープ設計](2026-07-14-3d-depth-scope-design.md)の選択中policyへ通常オブジェクトと同じように参加する。

- `Layer Order`: instanceのZは投影/パララックスへ効くが、外部レイヤーとの遮蔽はレイヤー順
- `Group Depth`: scope内のinstanceが共有depthへ参加する
- `AE-style Bins`: 明示選択時だけbin規則へ参加する

Cloner内部を先にRGBAへflattenすると「3DCG背景の中に2Dキャラや複製物を入れる」目的を再び壊すため、object/world/camera表現を共有する。大量instanceは可能な範囲でGPU instancingとsource評価共有を使うが、公開契約に特定GPU APIを露出しない。

### 12.8 UIの初期導線

1. CanvasまたはObject treeでsourceを選ぶ
2. `Cloner`を押すと、その場で同じ見た目のClonerへ置換プレビューする
3. Canvas上の終端handleまたはInspectorでcount/spacingを編集する
4. `Add Effector`で選択中Clonerに接続済みEffectorを作る
5. Canvas上の境界を直接動かし、影響中instanceをライブ表示する
6. 必要な場合だけ`Materialize`し、通常オブジェクトへ確定する

Cloner作成時に隠れGroup、Null、controller layer、expressionを生成しない。元sourceを所有下へ移すか参照するかはDocumentのownership規則に関わるため、仕様改訂前に決める。

### 12.9 不変条件と審判案

- 同じsource、parameter、seed、時刻から同じinstance列を得る
- instanceの同値順は安定し、preview/exportで一致する
- countを増減しても既存indexの結果が不用意に変わらない操作を定義する
- Effectorの追加、並べ替え、無効化がD2 commandで完全にUndoできる
- 2D/3D、Depth policyを切り替えてもClonerを作り直さない
- source更新がClonerへ反映され、Materialize後は明示的に独立する
- Cloner未使用Documentは既存ゴールデンとpixel-identicalである
- 1,000 instance fixtureでTimeline行を1,000本生成せず、UIスレッドを同期readbackで待たせない
- Effector未選択時にも、影響先と境界の識別表示を設定で維持できる
- source削除、循環参照、非対応parameter接続を型付き診断で拒否する

### 12.10 暫定判定

**強い採用候補**。ただしCloner/EffectorをM3だけの便利ツールにはしない。Document所有、評価グラフ、GPU instance、Depth参加、plugin parameter IDにまたがるため、M2/M5を含む独立仕様改訂が必要である。

製品群が示している需要は「Nullをもっと簡単に作りたい」より大きい。**複製集合と影響範囲を、映像上で見えるまま直接操作したい**が中心要求である。

## 13. R10: Relative Move / アニメーション軌跡の相対移動

### 13.1 問題の正確な位置

Positionにキーフレームがある状態でCanvas上のオブジェクトを動かす時、ユーザー意図には少なくとも2種類ある。

1. **現在時刻の姿勢を直す**: 現在キーを変更する、またはAuto Keyでキーを作る
2. **動き全体の置き場所を変える**: 全時刻へ同じ位置差分を足し、軌跡の形とタイミングを保つ

AEでも全Positionキーを先に選択してComposition上でドラッグすれば相対移動できる。[Adobe公式のMotion paths](https://helpx.adobe.com/after-effects/using/assorted-animation-tools.html)は「property名をクリックして全キーを選択してからdrag」と説明している。また[複数キー編集の公式説明](https://helpx.adobe.com/after-effects/using/editing-moving-copying-keyframes.html)では、数値入力は全選択キーを同じ絶対値にし、下線値またはCanvas上のdragは同じ差分を加えるとしている。

したがって能力が完全に無いわけではない。しかし最も頻繁な「動き全体を少し右へ」のためにTimelineを開き、propertyを展開し、全キーを選ぶ必要がある。ユーザーがNullを追加するのは、この相対オフセットを後付けする常設段が通常の直接操作に無いためである。

Blenderは[Delta Transforms](https://docs.blender.org/manual/en/latest/scene_layout/object/properties/transforms.html)をprimary transformの上へ適用し、既存animationを保ったまま配置を変える用途を明示する。Mayaも[Additive Animation Layer](https://help.autodesk.com/cloudhelp/2025/ENU/Maya-Animation/files/GUID-BBCA0BC3-7608-4E86-8E9F-B4099C316156.htm)を持つ。ただしMotoliiの単純な軌跡移動に、汎用animation layer stack全体を導入する必要はない。

### 13.2 初期UI案

- 通常drag: 現在時刻のPosition編集。Auto Key規則に従う
- **Primary modifier+drag**: Relative Move。macOSでは`Command+drag`候補。Windows/Linuxの割当はshortcut表で確定する
- ~~ToolbarまたはCommand Paletteにも`Move Animation / 軌跡全体を移動`を置く~~ — 後続決定で撤回。専用Tool/panelを増やさず、keymap可能なmodifier+dragとHUD/ghostだけに統一する
- 操作中は現在オブジェクトだけでなくmotion path全体をghost表示し、全キーが同じΔで動くことを示す
- HUDに`ΔX / ΔY`、3Dでは操作toolに応じて`ΔZ`を表示する。絶対Positionと混同させない
- `Escape`で取消、pointer-upで確定する

Primary modifierは設計上の操作名であり、キー割当そのものをDocumentへ保存しない。OS慣習やユーザー設定との衝突を確認してから確定する。

### 13.3 初期実装の意味

Position sourceごとに処理を分ける。

| Positionの供給源 | Relative Moveの初期動作 |
|---|---|
| `Const(Vec2/Vec3)` | 値へEdit-SpaceのΔを加える |
| `Keyframes` | 全Positionキー値へ同じΔを加える。時刻、補間、接線は不変 |
| `Vec2Axes`のConst/Keyframes | 対応軸の全値へΔ成分を加える |
| `DataTrack` / `Follow` / 手続き駆動 | **勝手にベイクしない**。加算offset契約が無い間は型付き診断で未対応を示す |

これにより、通常のキーアニメーションでは新しいDocument field、Null、parent、controller、expressionを作らずに解決できる。D2コマンドは対象sourceの全値変更を1 macroとして保持する。

DataTrack等にもRelative Moveを可能にするには、将来`Offset { source, delta }`のような型付き加算sourceか、primary animationとlayout transformを分ける追加フィールドが必要になる。既存`Transform2D.position`の解釈変更はせず、必要性と合成順を仕様改訂で先に定義する。

### 13.4 座標空間

Relative MoveのΔは**Edit Space**で定義する。root objectではWorld、同じGroup/parentを持つ子では共通parent空間とし、Depth Railの規則と揃える。

- 2D dragは選択対象のEdit-Space XY平面へviewport差分を逆投影する
- Depth Move tool中のPrimary+dragはPosition Zの全キーだけへΔZを加える
- Scale tool中に同じmodifierを押してもPositionへ密輸しない。scale全体の相対編集は別設計とする
- 異なるparentを持つ複数選択では、Canvas上で同じ見かけの移動になるworld差分を各parent空間へ変換する
- parent行列が特異で逆変換できない対象は、全体を部分適用せずgesture開始時に型付き診断で拒否する

### 13.5 選択との優先順位

- Canvas上でobject本体をPrimary+drag: そのobjectのPosition軌跡全体
- motion path上で特定keyを選択してdrag: 選択keyだけ
- 複数objectをPrimary+drag: 各objectの軌跡全体へ同じ見かけ差分
- GroupをPrimary+drag: Group自身の軌跡全体。子のキー値は書き換えない
- ClonerをPrimary+drag: Cloner node自身の配置軌跡。instanceごとのlayout値は書き換えない

選択keyが残っていても、object本体から開始したPrimary+dragは「軌跡全体」を優先する。開始地点とHUD labelで対象scopeを明示し、選択状態だけから暗黙に意味を変えない。

### 13.6 Auto Keyと履歴

Relative Moveは**Auto KeyがONでも新しいキーを作らない**。全時間配置を変える操作だからである。

- pointer-down時に対象sourceと全元値をsnapshotする
- drag中はtransient previewだけを更新する
- pointer-upでD2 macro 1件を発行する
- `Escape`またはcapture loss時は元値へ戻し、Document revisionとUndoを増やさない
- 値の一部だけ変更できない対象が混ざる場合はgesture開始前に拒否し、部分成功にしない

### 13.7 不変条件と審判案

- 全Positionキーの時刻、補間、接線、キー数が操作前後で一致する
- 任意の2時刻`a, b`について、操作前後の`position(b)-position(a)`が一致する
- 操作後の全評価時刻で`new_position(t)=old_position(t)+Δ`となる
- Relative Move中はAuto Key ON/OFFで結果が同一になる
- Group操作で子のPosition sourceがbit-for-bit不変である
- 異なるparent下の複数objectがCanvas上で同じscreen/world差分だけ移動する
- Depth MoveではZ以外、通常Moveでは意図しないZ/Scaleが不変である
- 1 gesture=1 Undo、Cancel=0 commandである
- DataTrack/Followを暗黙にkeyframeへ変換しない
- Relative Move未使用Documentは既存ゴールデンとpixel-identicalである

### 13.8 暫定判定

**強い採用候補**。Cloner/Effectorより基礎的で、Nullを使う理由を1つ直接消せる。初期版はConst/Keyframesへの非破壊な一括ΔコマンドとしてM3へ追加でき、スキーマ変更を要しない。

ただし「後からoffset値だけを再編集したい」要求は別である。その場合は通常値へ焼いた一括移動では足りず、型付きadditive transform段が必要になる。まず一回の相対配置変更を標準操作にし、常設offset段は反対側レビューと実利用で判断する。

## 14. R11: Effect Scope / Adjustment Layer

### 14.1 違和感の正体

Cavalryは公式に、LayerをShapes、Behaviours、Utilities、Effectsの4カテゴリへ分ける。[Cavalry Layers](https://cavalry.studio/docs/getting-started/key-concepts/layers/)と[Scene Tree](https://cavalry.studio/docs/user-interface/menus/window-menu/scene-window/scene-tree/)では、描画可能なShape/Null/Falloffにeye、非描画のBehaviour/Utility/Effectにenable checkを表示する。つまり同じScene Treeの「Layer」には、画像を作るものと計算だけを行うものが混在する。

このモデルはnode graphとしては理解できるが、visual stackingを読むTimeline/Scene Treeでは次の違和感が出る。

- 行があるのに、その行自身の画素、境界、Z、合成面が無い
- 上下移動が描画順なのか依存順なのか処理順なのかをiconで判別する必要がある
- 非描画Effectを選んでもCanvas上で直接掴む実体が無い
- 「Effectがどの対象へ、どの段階で作用するか」が階層や接続を読むまで分からない

これはEffectが悪いのではなく、**visual layerの比喩を処理ノードへ拡張しすぎたこと**による。

### 14.2 AE Adjustment Layerの実際

[Adobe公式](https://helpx.adobe.com/jp/after-effects/using/creating-layers.html)によれば、Adjustment Layerは単に下の各レイヤーへ同じEffectを配るのではない。正確には、**stack上で下にある全レイヤーから作られたcompositeへEffectを1回適用する**。maskを使えば画面領域は制限できるが、対象レイヤーのscopeは「下全部」のままである。

この方式には利点もある。

- 複数レイヤーへ1回だけ処理できる
- Timeline上のin/outで有効時間を切れる
- maskで画面領域を制限できる
- stack位置が処理順を兼ねる

一方、次の意味を暗黙の上下関係へ押し込む。

- どのvisual objectが対象か
- どの時点までを先にcompositeするか
- full frameかmask領域か
- 2D/3Dのどの境界でflattenするか

[Adobeのprecompose/collapse説明](https://helpx.adobe.com/after-effects/using/precomposing-nesting-pre-rendering.html)でも、nested composition内のAdjustment Layerはflattening/croppingを強制し、外側3D layerとの交差、shadow、camera/light参加を失わせる場合がある。Adjustment Layerは便利な共有Effectであると同時に、**見えにくい合成境界**でもある。

### 14.3 Motoliiの基本方針

Effectをvisual layerの同格行として作らず、**所有者のEffect Stackに属する処理項目**とする。Timelineでは所有者を展開したproperty/effect行として時刻を編集できるが、top-levelの描画順には数えない。

Effectの意味は、次の4軸を別々に表示する。

| 軸 | 意味 | UI |
|---|---|---|
| **Targets** | 誰に作用するか | owner名、target chips、target数 |
| **Stage** | 各object合成前 / Group合成後 / Output後 | stage icon+label |
| **Region** | 全域 / mask / shape / spatial field | Canvas outline+region chip |
| **Time** | いつ有効か | owner下のeffect intervalまたはkeyframed enable |

Effect選択時は、対象objectをCanvasとTimelineで強調し、処理段の直前/直後をInspectorで示す。文字や色だけに依存せず、target badge、stage形状、region outlineを併用する。

### 14.4 Scopeの初期候補

| Scope | 処理 | 用途 | Flatten |
|---|---|---|---|
| **Self** | 1 objectの出力へ適用 | blur、color、distort等 | object内部だけ |
| **Each Target** | 明示targetごとに同じEffectを個別適用 | 複数キャラの同一補正 | target間をまとめない |
| **Group Composite** | Groupの子を合成後、結果へ1回適用 | 一まとまりのgrade/glow | **Group境界で明示** |
| **Backdrop Surface** | shape領域内で背後のcompositeを取り込む | 局所blur、glass、部分補正 | layer位置の背後だけ |
| **Composition Output** | 最終出力へ適用 | 全体grade、vignette、grain | Output段で明示 |
| **Stack Range** | 指定した連続範囲のcompositeへ適用 | AE互換の高度用途 | **範囲境界で明示** |

既定はSelf。複数選択からEffectを追加する場合は`Apply to Each`を既定とし、同じparameter sourceを共有しても処理は各object内で行う。

複数objectを先に合成して1回だけEffectを掛ける場合は、既存Groupの`Group Composite`を使う。離れた非連続objectだけを一時合成すると、その間にあるlayerとのblend/depth意味が変わるため、暗黙のtarget set compositeは作らない。Groupが無ければ作成確認を出すが、自動生成しない。

### 14.5 Stack RangeはAdvancedだけにする

ユーザー選択肢としてAE式のAdjustment Layer相当を残す場合も、「この行より下全部」を無表示で採らない。

- `Stack Range`をAdvanced scopeとして明示選択する
- Timeline上に開始/終了boundaryを描き、影響中の連続行を括弧で囲む
- layer並べ替え中に対象変化をlive previewし、pointer-up前にtarget数を表示する
- range外へ移動したobjectの意味が変わることをUndo 1件に含める
- 3D/depthをflattenする位置へ警告badgeを表示する
- 既定の終端はcomposition bottomではなく、ユーザーが選択した連続範囲とする

これによりAE互換の自由度は保てるが、安全な既定にはしない。Stack Rangeを使わないDocumentでは、visual layer順がEffect targetを遠隔変更しない。

### 14.6 Alight Motion型のBackdrop Surface

[Alight Motion公式Effect Guide](https://guide.alightmotion.com/effects/copy-background)の`Copy Background`は、背後にある全レイヤーのcompositeを現在layerへコピーする。元layerの不透明部分がcoverageになり、`Copy Background`より後ろのEffectが取り込んだ背景画素へ作用する。公式自身が「実質的にlayerをAdjustment Layerへ変える」と説明している。

この方式はAE Adjustment Layerより直感的な用途がある。

- 通常のshape/vector/freehand layerがあるため、Canvas上で作用領域を直接掴める
- shapeのtransform、path、mask animationがそのまま領域編集になる
- full frame rectangleを作らない限り、画面全体へ作用しない
- layerのin/outがそのまま有効時間になる
- 取り込んだ背景へ通常のEffect Stackを使える
- mosaic、局所blur、部分色補正、glass、glow等を同じ作法で作れる

したがってMotoliiでも、通常visual objectへ追加できる`Backdrop Capture`またはユーザー向け名称`Copy Background`相当をfirst-party Effect候補とする。このobjectは「実体のないEffect Layer」ではない。**shapeのcoverageという実体を持ち、appearance inputだけを背後のcompositeへ差し替えるvisual object**である。

### 14.7 Effect Stack内の境界

Alight MotionではEffect順が重要で、`Copy Background`より前のEffectはmask形状へ、後のEffectは取り込んだ背景へ作用する。Motoliiではこれを単なる順序知識にせず、Effect Stack内に明示dividerとして表示する。

```text
Shape / Coverage
  Mask・Path・Coverage Effects
  ── Capture Backdrop ──
  Pixel Effects: Blur / Mosaic / Color / Distort
  Blend / Opacity
```

- dividerより前はcoverage生成段、後はcaptured RGBA段とlabelする
- captured RGBAへ影響しないEffectを前へ置いた場合は、無効結果として構造化診断する
- dividerの上下移動は可能でも、処理段が変わるpreviewと警告を出す
- 元shapeのfillを残す量は`Original Fill Mix`として0〜1で明示する。0はcaptured backdropのみ、1は元fillのみ、中間は両者のmix
- Effectを外すと通常shapeへ戻り、隠れた背景コピー素材を残さない

### 14.8 Group Compositeとの使い分け

| ユーザー意図 | 選ぶもの | 対象決定 |
|---|---|---|
| 特定のobject群をまとめて補正 | **Group Composite Effect** | Group membership |
| object群へ同じEffectを個別適用 | **Each Target** | 明示target IDs |
| 円や任意shapeの内側だけ背後を加工 | **Backdrop Surface** | Canvas coverage+stack位置 |
| 作品全体のgrade/post | **Composition Output** | final output |
| AE互換の連続stack範囲 | **Advanced Stack Range** | 明示range boundaries |

Backdrop SurfaceはGroupを増やさず局所Effectを作れる。一方、対象は「そのshapeより背後に見える画素」であり、特定objectのidentityは追わない。対象objectをstack内で前へ移せば取り込みから外れる。identityを保って補正したい場合はGroupまたはEach Targetを使う。

### 14.9 Captureの意味と制約

- capture sourceの既定は、**同じcompositing scope内でこのobject直前までに確定した背景color**
- 背景をCPU画像へcopyせず、GPU上の蓄積textureまたは必要領域を入力として読む
- 同じtextureへのread/write hazardはping-pongまたは別attachmentで解消し、公開契約にwgpu内部型を出さない
- capture結果はDocumentやcache payloadへ保存せず、時刻`t`の評価から再生成する
- layer順に対する後方参照だけを許し、自分自身や前景への循環参照を作らない
- capture bounds外をEffectが参照するblur/distortでは、必要paddingをEffectが宣言し、切れを防ぐ

### 14.10 3D/depth上の扱い

Backdrop Surfaceは画面上では自然だが、真の3D refractionとは異なる。

- `Layer Order`では、そのobject直前の蓄積colorを通常どおりcaptureできる
- `Group Composite`内では、そのGroup内部で直前までに合成されたcolorだけをcaptureする
- `Composition Output`ではscreen-space region Effectとして使える
- 共有depth pass内のworld-space透明面で使う場合、opaque scene color、透明順、屈折、depth samplingの別設計が必要

v1ではBackdrop Surfaceを**screen/composite-space Effect**として明示し、Group Depth内の真の屈折材質を名乗らない。3D Canvasに置いた場合も、どのcomposite段をsampleするかをbadgeで表示する。将来のworld-space refractionはM5の別spikeとする。

### 14.11 Regionは対象Scopeと分離する

AEのmask付きAdjustment Layerは「誰」と「画面のどこ」を1つの透明layerへ載せる。MotoliiではRegionをEffect入力として分離する。

- `Full Target Bounds`: Self/Each Targetの既定
- `Full Frame`: Composition Outputでのみ既定
- `Mask/Shape Reference`: 明示shapeのcoverage
- `Spatial Field`: falloff付き範囲。Cloner Effectorと同じfield語彙を再利用可能

RegionにはCanvas上の実体があるため、outline/handleを直接操作できる。Effect自身には画素が無くても、作用領域は見える。mask/shape参照は型付きIDで保存し、文字列expressionや隠れcontrol layerを作らない。

### 14.12 Effectの時間表示

Effectをtop-level visual layerにしなくても、時間範囲は表現できる。

- owner行を展開するとEffect Stackと各Effectのintervalを表示する
- interval外ではEffect評価をskipする
- fade/strengthは通常parameterとしてkeyframe可能
- Group Composite EffectのintervalはGroup行の内側に表示する
- Composition Output Effectは専用Post Stack行へ表示し、visual layerの上下へ混ぜない

時間を持つこととvisual stackingへ参加することを分離する。

### 14.13 3D/depthとの整合

- Self/Each Targetのobject-spaceまたはsurface-compatible Effectは共有depth参加前に処理できる
- RGBA入力を必要とするSelf Effectは、そのobject内部のraster境界だけを持つ
- Group CompositeはGroupの子をRGBAへまとめるため、その位置がflatten boundaryであることを表示する
- Composition Outputはdepth合成完了後のscreen-space postである
- Stack Rangeは範囲内compositeを作るため、Group Compositeと同等以上に強いflatten警告を出す
- Backdrop Surfaceは直前のcomposite colorを読むscreen/composite-space段として識別する

Effect API側も必要入力を`Object/Surface/RGBA/Output`等の型で宣言し、ホストが処理段を推測しない。現行公開契約へ未決variantを先行追加せず、M5 spikeと仕様改訂を先に行う。

### 14.14 不変条件と審判案

- Self Effectの追加で兄弟objectの画素と評価グラフが不変である
- Each Targetは対象ごとの個別適用と一致し、target間のblend/depth順を変えない
- Group Compositeの有無で変わるflatten boundaryをgraph fixtureで検出できる
- Composition Output以外のEffectが無指定でfull frameへ作用しない
- Region変更がTargetsを変えず、Targets変更がRegion geometryを変えない
- visual layer並べ替えがSelf/Each Target/Group Compositeのtarget集合を変えない
- Stack Rangeだけは並べ替えによるtarget変化を明示previewし、Undoで完全に戻る
- Effect interval外では同Effect未接続とpixel-identicalである
- Effect行はtop-level visual layer数、Z sort、hit-test候補へ数えない
- scope/region参照の削除、循環、非対応stageを型付き診断で拒否する
- Backdrop Surfaceのcoverage外が同Effect未使用時とpixel-identicalである
- Backdrop Surfaceより前景のobjectを変更してもcapture結果が変わらない
- Backdrop Surfaceより背後のobject変更は同じ時刻のcaptureへ即時反映される
- `Original Fill Mix=0`では元fill色が結果へ混入せず、`=1`の合成規則が仕様どおり再現される
- capture padding fixtureでblur/distortの端が不意に切れない
- depth modeでscreen-space captureをworld-space refractionとして誤評価しない

### 14.15 暫定判定

**Effect Layerは標準概念として棄却候補、Effect Scopeは採用候補**。EffectにTimeline上の時間表示は与えるが、visual objectのふりはさせない。

Adjustment Layer相当は、`Group Composite`、`Backdrop Surface`、`Composition Output`で通常用途を覆い、AE式の「連続stackへ掛ける」はAdvanced `Stack Range`としてのみ残す。画面全体への作用はComposition Outputを明示選択した時だけ既定にする。

特に局所的な加工ではBackdrop Surfaceを第一選択にする。作用対象が「背後の画素」でよい限り、Groupを増やさず、maskを別管理せず、Effectの範囲をCanvas上の通常shapeとして説明できる。

## 15. 仕様へ進める順序

| 優先 | 変更候補 | 先に必要なもの | 現時点の扱い |
|---:|---|---|---|
| 1 | Relative Move | D2 macro/merge、Canvas transform契約 | **one-shot版をM3-U2fへ正式割当**。常設offsetだけPP-Gate待ち |
| 2 | Timing Rail | D2 macro/merge、Timeline編集契約、反対側レビュー | 詳細案まで記録 |
| 3 | Cloner/Effector | M2所有、M5評価/Depth、instance spike | 製品採用前にM5-P0I境界spike |
| 4 | Effect Scope | M2 Effect所有、M5処理段/flatten | 三分類を決定。ExplicitSet/Backdrop schemaは独立仕様改訂待ち |
| 5 | Text addressable spans | M5 P6 cluster mappingの審判 | first-party plugin要件候補 |
| 6 | Anchor/Align/Distribute | M3 selection/bounds契約 | UI要件候補 |
| 7 | Search/Filter/Selection | Workspace-session所有決定 | UI要件候補 |
| 8 | Import/animated bounds | importとderived cacheの所有決定 | M4-K0 RoD/RoI契約へ前倒し |
| 9 | Path batch operations | shape/path選択モデル | 監査待ち |

ここに挙げた`M3-U3e`等はレビュー内の仮称であり、仕様書の確定タスクIDではない。未決の所有境界を、それらしいデフォルトで埋めない。

## 16. 非目標

- aescripts製品を網羅した市場一覧を作ること
- 製品数を根拠に機能を無条件採用すること
- AE互換のexpression、null、controller、precomp補修を再現すること
- Rail自体をDocumentオブジェクトとして永続化すること
- 全操作を1つのAdvancedモードへ詰め込むこと
- 高価なalpha boundsやpixel readbackをライブUIの必須経路にすること
- specialized creative effectを標準機能へ吸収すること
- Nullを万能制御レイヤーとして標準作法にすること
- Cloner作成時に大量レイヤー、expression、controllerを裏で生成すること
- Linked cloneのidentity未決のまま個別overrideを永続化すること
- Relative MoveのためだけにNull、parent、animation layerを生成すること
- DataTrack/Followを暗黙にkeyframeへベイクすること
- 非描画Effectをtop-level visual layerとして扱うこと
- Effectのtargetを無表示のlayer隣接関係だけで決めること
- Group/Stack Rangeのflatten boundaryを隠すこと
- Backdrop Surfaceを真のworld-space屈折として扱うこと

## 17. 反対側レビューで潰す問い

1. Timing Railは既存Timeline複数選択と数値入力だけで十分ではないか
2. 空間順の時間差展開は、camera移動中にどの時刻の評価位置を使うべきか
3. Text rangeのアドレスは再組版後も安定して参照できるか
4. visual boundsを持たないことで、整列品質が実用上不足しないか
5. 検索/絞り込みを標準化すると、大規模Documentでindexの恒久契約が必要にならないか
6. 同じ操作文法をDepth/Timingで共有することが、軸固有の違いを隠さないか
7. Clonerはfirst-party pluginで足り、Documentの一級概念にする必要はないのではないか
8. Group transformとControl handleを分けることで、かえって選択対象を増やさないか
9. Procedural instanceだけでMVユーザーの個別修正要求を満たせるか
10. Effectorの複数接続は、Null/expressionと同じ見えない依存関係を再生産しないか
11. 全キー値を書き換える方式は、常設additive offsetより編集意図を失いやすくないか
12. Primary+dragはOSや他のCanvas操作と衝突しないか
13. 異なるparent下で「同じ見かけ差分」を優先すると、正準値がユーザー予想から外れないか
14. Effectをowner下へ格納すると、長いEffect Stackの時間編集が発見しにくくならないか
15. Each Targetの共有parameterと、Effect instance自体の共有を区別する必要があるか
16. Advanced Stack Rangeを残すことが、隣接による遠隔作用を再導入しないか
17. Group Compositeを要求すると、EffectのためだけのGroup増加を招かないか
18. Backdrop Capture dividerの前後という二段Effect Stackは、Alight Motionの単純な順序より分かりやすいか
19. shape boundsとEffect paddingから必要capture領域を安全に決定できるか
20. Backdrop Surfaceのstack依存は、Advanced Stack Rangeと同じ遠隔作用を局所的に再導入しないか

これらを通過した項目だけを、採用/縮小/延期/棄却の判定語付きで仕様へ移す。
