# 操作単純化モデル — 複雑さをユーザーへ転嫁しないための横断仕様

日付: 2026-07-14

状態: **現行コンセプトのM0〜M5への割当。凍結済み公開契約の変更は含まない**

正本: [concept.md「操作設計の根本原則」](concept.md#操作設計の根本原則-複雑さをユーザーへ転嫁しない)

UI側の既知外殻、視覚動線、共通component契約、漏れ実装の拒否は[UI操作言語](ui-interaction-language.md)を正本とする。

利用者と開発者の学習曲線、探索しても壊れない条件、編集系pluginの責任寿命とcapability境界は[小さなコアと探索可能な拡張](extensible-core-model.md)を正本とする。

先例調査: [反復再発明の標準化監査](reviews/2026-07-14-repeated-wheel-standardization-audit.md)、[4ツールの称賛・日曜大工・根本ギャップ監査](reviews/2026-07-14-motion-tools-praise-diy-gap-audit.md)

## 1. 目的

本書は「シンプルにする」を見た目や初心者モードの話で終わらせず、フェーズごとの実装責務と審判へ変換する。

Motoliiが削る対象は機能ではなく、次の**制作と無関係な負荷**である。

- 同じ目的のために隠れNull、補助layer、precomp、expressionを組む。
- 操作順を間違えると意味が変わる「作法」を暗記する。
- 値の由来、作用scope、依存先、再計算理由を別画面で探す。
- 簡易操作で作った状態を高度編集時に作り直す。
- plugin作者ごとに導入、命名、controller、更新手順が異なる。

シンプル化は機能を隠すことではない。**通常操作を短くし、その結果をAdvancedで完全に検査・制御できること**を指す。

## 2. 全フェーズ共通の不変条件

### S-1. 1つの意味、3つの入口

```text
Direct operation ─┐
Named Tool        ├─→ Domain Intent → D2 Command → Document meaning
Advanced editor  ─┘
```

- `Direct`: Canvas drag、shortcut、target click等の最短操作。
- `Tool`: 奥行き展開、Stagger等、目的の名前を持つ操作。Relative Moveは専用Toolでなくmodifier+drag gestureである。
- `Advanced`: 評価列、明示scope、policy、数値を検査・編集する入口。

3入口は同じDomain Intentまたは同じDocument意味へ正規化する。簡易UI専用field、Advanced専用コピー、隠れhelper objectを作らない。

### S-2. Simple表示中も意味を隠さない

Advanced controlsを畳んでも、出力へ影響する状態は要約表示する。

- 通常値と異なるdepth policyが有効ならpolicy名または識別可能なbadgeを示す。
- 他object/DataTrack/pluginに駆動されている値は由来を示す。
- effectの対象scopeを「下全部」のような暗黙位置関係だけにしない。
- plugin欠落、非対応alpha、fallback、近似を警告なしで隠さない。

「Advancedを開かなければ現在の意味を判別できない」は不合格とする。

### S-3. 操作短縮の審判

クリック数だけを固定KPIにしない。代表操作ごとに次を記録する。

| 項目 | 合格条件 |
|---|---|
| Domain Intent | ユーザー目的の名前があり、UIイベント列を永続化しない |
| 永続物 | 生成・変更されるDocument要素を列挙できる |
| 隠れ物 | Null/controller/helper layer/expressionを無表示で生成しない |
| Undo | 1 gestureまたは1 tool確定が1履歴。Cancelは変更ゼロ |
| Round-trip | Direct/Toolで作った状態をAdvancedで検査・編集し、閉じても意味不変 |
| 依存 | target、scope、plugin、DataTrackを型付きIDで追跡できる |
| 失敗 | typed errorまたは診断。別の意味へ無言fallbackしない |
| 性能 | 操作中にUI thread待機、GPU同期readback、全Document再構築をしない |

### S-3a. 接続操作はカーソル自身が意味を説明する

LookAt / Follow / Parent / DataTrack / Effect Use等の型付き参照は、渦巻きiconやsocketだけを置いて意味を推測させない。接続開始から確定またはCancelまで、カーソル近傍へ少なくとも次の三要素を含む短文を常時表示する。

- **何を変えるか**: 例「このグループの移動」
- **何へ繋ぐか**: 例「円形パス」または未選択時の期待型「パス」
- **どうなるか**: 例「パスに沿って移動します」

実装は汎用node editorではなく、`Idle → Picking → HoverValid / HoverInvalid → Commit / Cancel`のTransientな状態機械とする。`Picking`中は期待型を明示し、接続可能targetを形+outlineで強調、接続不能targetをdimし理由を文言で示す。hover中は仮線または同等のfrom/to手掛かりを表示し、確定後はInspector等へ`移動経路 → 円形パス`相当のsemantic badgeを残す。平常時まで追従文を常時出さず、接続controlのhoverでは開始方法をtooltip、接続mode中はカーソル追従文を常時出す。

ウィップdrag、`接続`button、Canvas/Timeline clickは同じConnection Intentへ正規化する。Documentへ保存するのは既存の型付きIDと決定済み値だけで、pointer軌跡、hover、入口種別、説明文、仮線を保存しない。iconだけのウィップ、layer名/property path文字列、接続のための隠れhelper生成を正規入口にしない。

#### Advanced例外の許容境界

今後の論点は接続UIを重くすることではなく、通常入口の外にどこまで高度な接続意味を追加できるかである。Advancedは説明や検査を省く裏口ではなく、同じDocument意味の由来、対象、評価順、所有/共有、失敗理由を詳しく検査・編集する入口とする。

高度用途の例外は、次をすべて満たす場合だけ独立仕様で追加できる。

1. target型、作用scope、評価順、循環/欠落時の失敗が宣言できる。
2. Simple表示を閉じても接続の存在と由来がsemantic badgeから分かる。
3. 接続、解除、所有化/共有化等が明示D2 commandで可逆になり、複製・移動時の参照規則をfixture化できる。
4. preview/export、cache invalidation、rename、削除後の意味を自動審判できる。
5. 文字列expression、名前検索、隠れcontroller、型検査や循環拒否を外す`force connect`を要求しない。

この条件を満たしても、cross-group参照、共有path、複数target、座標space変換、接続後Modifier列等の具体形はここで一括採用しない。最小の通常接続で実需を観測し、例外を1契約境界ずつ追加する。Advanced表示の存在を理由に未決のDocument fieldや汎用Constraint Graphを先焼きしない。

### S-3b. 探索を罰しない

安全性を「想定値だけを許すこと」と定義しない。極端な値、逆転、発散、画面外、奇妙な接続候補は表現の自由であり、意味と回復経路を保てる限り許可する。

- 操作中はTransient previewで結果を見せ、Cancelは変更ゼロにする。
- 拒否するのは型不一致、宣言されない循環、復元不能なmutation等、Documentの因果と回復可能性を壊す操作に限る。
- clampやfallbackで別の意味へ黙って補正せず、必要なら警告と明示確定を使う。
- 画面外へ移動した対象、極端に小さい対象、欠落pluginを選択・Fit・Reset・Undoから回収できる。
- 一部の失敗で無関係なDocument領域を操作不能にしない。

合格の基準は「初心者が値を適当に動かしても常に整った絵になる」ではなく、**試した結果を理解し、元へ戻り、別の組合せを続けて試せること**である。

### S-4. Expressionとpluginの位置

解決順は次とする。

1. Hostの直接操作または型付きprimitiveで解けるか。
2. 複数primitiveを目的単位のfirst-party Tool/presetへ畳めるか。
3. 未知・専門用途ならuser pluginの型付き境界で試せるか。
4. それでも表せない時だけ、v2のWASM parameter pluginを脱出口として使う。

文字列expressionを標準プロジェクト作法にしない。pluginも自由なDocument mutation、layer名検索、隠れ状態、隠れcontrollerを持たない。

### S-5. pluginからHostへの昇格

```text
user plugin / recipe
        ↓ 反復需要を観測
validated preset / first-party plugin
        ↓ 意味と審判が安定
Host primitive / Direct Tool
```

昇格判定は販売数や要望数だけで行わない。次をすべて確認する。

- 複数作者が同じ目的を独立に再実装している。
- 作品固有の見た目ではなく、Undo・scope・依存・選択等の編集基礎である。
- Hostでなければ隠れhelperや不安定なlayer列挙が必要になる。
- 最小の型付き意味と、自動判定可能な審判を定義できる。
- 既存Documentへ追加的に導入できるか、migrationを明示できる。

## 3. 代表操作コーパス

M3のUI実装前に、最低限次の操作を同じ書式で台帳化する。

| Intent | Direct | Tool | Advanced | Hostが保存する意味 | 禁止する補修 |
|---|---|---|---|---|---|
| 相対移動 | keymap modifier+Canvas drag | — | drag中HUD+motion path ghostのみ | **v1正式機能**: D2 macroによるConst/選択source全key差分。常設UI/Modifierなし | Null、expression、隠れoffset channel、専用Tool |
| 追従 | targetをCanvasでclick | Follow | target ID、offset、評価順 | `DocParam::Follow`等の型付き参照 | layer名文字列、pick-whip式文字列 |
| 反復 | sourceからdrag/create | Clone/Stagger | index、distribution、seed | Hostのinstance/context境界。具体表現はplugin可 | 大量layer、式コピー |
| 局所effect | Effect out→Layer stack inへdrag | Timeline Effect Link | definition/use、from/in、stack位置 | Owned=合成後1回、Explicit=共有recipeを各layerへ個別適用、Backdrop=Host入力+plugin処理 | 「下全部」の無表示推論、隣接依存、二重描画 |
| 奥行き配置 | Z rail drag | 奥行き展開 | depth policy、participant、数値 | 通常transform + 明示policy | controller、auto group、Bake必須 |
| key easing | 区間選択→preset | Easing popup | 補間型とparams | 区間`Interp` | valueAtTime式、暗黙近傍curve変更 |
| plugin parameter | 自動panel | plugin preset | source/version/type/dependency | NodeDesc準拠params | custom UIでしか編集できない保存値 |

この表の「常設Modifier」「汎用Element Domain」「永続Constraint Graph」は未決である。既存variantへ推測で焼かず、それぞれ独立レビューを通す。一方、Relative Moveのone-shot版、Bounds/ROI最小契約、Scope三分類、Instance/Element spikeは[既知技術による処分決定](reviews/2026-07-14-motion-foundation-known-tech-disposition.md)に従い、未決事項と一括保留しない。

## 4. Param Pipeline Gate（PP-Gate）

Autographの`Generator → Modifier[] → Result`は有力だが、現行`ParamSource`は値の出所を選ぶ凍結済み契約である。次を満たすまでModifier列をDocumentへ追加しない。

M2終了時の扱いと発火条件は[判定記録](reviews/2026-07-14-m2-exit-param-pipeline-disposition.md)を正本とする。M2 blockerにはせず、M3で常設補正・汎用Modifier・高度property評価列のいずれかへ着手する前に解凍する。

| Gate | 必要な証拠 |
|---|---|
| PP-1 意味 | `Base / Link-or-Driver / Modifier[] / Result`の型、順序、循環拒否、errorを宣言 |
| PP-2 小さい代替 | `Transform.offset`等の少数field、D2 key差分、presetだけで足りないか比較 |
| PP-3 可逆操作 | Canvas dragをどの段へ逆写像するか。逆変換不能時のUIを宣言 |
| PP-4 永続化 | 追加的schema、旧project migration、未知Modifier保持を宣言 |
| PP-5 評価 | preview/export一致、依存順、cache invalidationの意味論golden |
| PP-6 反対側レビュー | 過剰なnode graph化、順序例外、UI認知負荷を独立再判定 |

PP-Gate前のRelative Moveは**選択keyへ同じ差分を適用する1回のD2 macro**に限定する。永続的な後段offsetと偽らない。

### 4.1 v1.x候補: One-Knob Macro Control

AbletonのMacro Controlのように、**一つのcontrolから複数parameterを同時に動かす**入口は、複雑な設定を演奏可能な少数ノブへ畳めるためMotoliiとも相性がよい。ただしM3の基礎UIには入れず、v1.xの追加候補とする。

これはD2の「複数commandを1 Undoにまとめるmacro」やshortcut macroとは別物である。保存される一対多のparameter driverになるため、実装前にPP-Gateを通す。

```text
Macro Control M
  ├─ typed target A + mapping
  ├─ typed target B + mapping
  └─ typed target C + mapping
```

最低限、次を仕様改訂で決める。

- Macroのscopeとstable identity。Group、Layer、Effect Definition、projectのどこに所有させるか。
- targetを名前やproperty path文字列でなく、安定IDと期待ValueTypeで参照する方法。
- Macro入力域と、targetごとのmin/max、反転、clamp、将来curveの写像。
- Macro自体をkeyframe/DataTrackで動かす場合の評価順とcache invalidation。
- target側の通常値、Link/Driver、将来Modifierとの合成順と、Canvasからの逆編集可否。
- 自己参照/相互参照の循環拒否、削除済みtarget、型変更、plugin欠落の表示。
- Group/Definition複製時に内部targetを複製先へ張り替え、外部targetを明示的に維持する規則。
- 1 knob drag=1 Undo、Cancel変更ゼロ、preview/export同一。

Simple表示はMacro knobと接続target数/異常のsemantic badgeを残し、Advancedで各target、範囲、反転、評価順を検査する。隠れcontroller layer、文字列expression、UIだけに存在するmapping、custom UIでしか編集できないtargetを作らない。

#### UI配置

初期配置は、独立windowではなく**右Inspector内のEffect編集領域**を第一候補として固定する。選択中のLayer / Group / Effect Definitionに対応するeffect-stackまたはparameter panelの上部へ、横一列のMacro stripとして置く。

```text
Inspector / Effects
┌─────────────────────────────┐
│ Macro strip  [M1] [M2] [M3]│  ← knob + target数/異常badge
├─────────────────────────────┤
│ Effect stack / Parameters   │
│ ...                         │
└─────────────────────────────┘
```

- 平常時はノブ、名称、target数、欠落/循環等の異常だけを表示する。
- `Map`またはAdvancedで同じInspector領域を展開し、target/range/invert/orderを編集する。別の浮動windowを唯一の編集口にしない。
- StageとTimelineにはMacro本体を重複配置せず、接続/自動化の存在をsemantic badgeで示し、選択するとEffect編集領域へ戻す。
- 画面上の配置を先に決めても、Document上の所有scopeは決めたことにしない。Group/Layer/Effect Definitionのどれが正本かはMC-0で複製意味と同時に決める。
- 初期版は同じEffect編集context内のtargetへ範囲を限定する案を小さい代替として比較し、project横断mappingを既定にしない。

## 5. M0〜M5への割当

### M0 — 成立性を先に測る（完了済み・遡及変更なし）

役割は「短い操作を作る」ことではなく、その前提を実測することだった。

- S1: render負荷中もdrag/IMEが動き、UIがGPU readbackを要求しないこと。
- S2: seekが操作モデルを人質に取らないこと。
- S3: frame、音声sample、BPMを丸め誤差の作法へしないこと。

完了済みM0へ新しい製品機能を追加しない。今後UI案の成立性が疑わしい場合も、本体へ試作を混ぜず独立spikeへ戻す、という手順だけを継承する。

### M1 — 同じ意味を通す縦スライス（完了済み・契約基線）

M1が保証した土台:

- `render_frame(t, Quality)`の単一路線。
- 型付き`Value / ParamSource / DataTrack`。
- plugin純関数とNodeDesc。
- 正準座標、GPUテクスチャ、preview/export共通評価。

M1へDirect/Tool/Advanced UIを遡及追加しない。以後の簡易操作は、この評価基線を迂回する別経路を作ってはならない。PP-GateがM1公開契約の変更を要求する場合は、凍結解除と既存golden維持を先に行う。

### M2 — 意味、Undo、可搬性をHostが所有する

M2は単純化の**意味側**を担当する。UI都合のfieldは追加しない。

| 割当 | 既存タスク | 追加する完了観点 |
|---|---|---|
| Intent→Command | D2 | 代表操作が決定済みdomain値を記録し、Direct/Tool差では履歴形式を分けない |
| 1 gesture=1 history | D2 | macro/merge/Cancelを代表操作コーパスでproperty test |
| 型付き依存 | D1a/D1h/D3 | Follow/LookAt/Data参照をIDと期待型で検査。文字列property path禁止 |
| 欠落plugin可読性 | D1f | 未知pluginを保持し、警告、使用箇所、再保存不変を保証 |
| 欠落plugin書出し | D6接続 | pass-through可能性を判定し、再現不能ならtyped error |
| 評価順 | D3/D1i-3 | UI入口に依存せず同じDocumentが同じrender graphになるgolden |

M2で新規に決めてよいのは既存意味へのCommandだけである。PP-Gate、汎用Element Domain、Constraint Graphは別の仕様改訂前に着手しない。

### M3 — 最短操作と検査可能性を同時に作る

M3は単純化の**入口側**を担当する。外観だけを簡単にして意味を隠してはならない。

#### M3-GS: 操作単純化ゲート

G0の入力として次を確定する。

1. 代表操作コーパスの各行にDomain Intent、永続物、Undo、失敗を記入。
2. Direct/Tool/Advancedのうち存在する入口と、未実装入口を明示。
3. Simple表示で残すsemantic badge一覧を決定。
4. 同じfixtureを各入口から操作し、最終DocumentまたはD2 command列が意味同値になる審判を決定。
5. AE等との比較操作列は証拠として保存するが、競合よりNクリック少ないことを恒久契約にはしない。

#### M3タスクへの割当

| 責務 | 接続先 | 完了条件 |
|---|---|---|
| Domain Intent | U0b/U0c | UI eventやegui/eframe/winit型を含まず、同じIntentをshortcut/button/Canvasから発行可能 |
| Command正規化 | U2a/U2b | 入口違いで同じDocument意味、Undo 1回、Cancel変更ゼロ |
| Conformance harness | U2c | 代表操作を複数入口で実行し、hidden itemなし・serialize意味同値を検査 |
| semantic badge | U0e/U4a | key/Data/Link/plugin/scope/policyを文字だけ・色だけに頼らず識別 |
| Advanced round-trip | U4c | 畳む前後でserialize不変。Directで作った既存意味を検査・編集可能 |
| plugin fallback | U4a | custom UIなしで全保存paramを編集可能 |
| 操作性能 | U1c/U3a | action sequenceのp50/p95、UI非blocking、readbackなし |

`G0-7 / U2c / U4c`は[2026-07-14全層反映監査](reviews/2026-07-14-recent-concept-propagation-audit.md)を経てM3仕様の正式IDへ昇格した。個別候補機能の採用を意味せず、まず既存意味を同一入口へ正規化・検査するHost審判を実装する。

### M4 — 短い操作が再計算地獄を生まない

M4は単純化の**予測可能性**を担当する。

- Direct/Tool/Advancedの入口差をcache keyへ入れない。評価結果を変えるDocument意味だけを入れる。
- plugin ID/version/content hash、DataTrack、target参照、scope、policy、将来Modifier順をキーへ含める。
- 1パラメータ変更で影響ノードだけを無効化し、理由を開発HUDから追跡できるようにする。
- proxy、Bake、解析jobを操作の前提にしない。未完了なら低品質または未解析表示で編集を続ける。
- Purge、Refresh、Bakeし直しを通常の修復手順にしない。

K1のキー網羅性変異テストとK2の無効化伝播へ、代表操作コーパスのDocument変異を入力する。将来PP-Gateを通過した場合はModifierの追加・削除・並べ替え・plugin version変更を同じ変異集合へ追加する。

### M5 — 2D/3Dの複雑さを直接操作へ畳む

M5は原則の最初の大規模な実地審判になる。

| Intent | 通常入口 | Advanced | Document意味 | 審判 |
|---|---|---|---|---|
| 2D素材を奥へ置く | Depth Move | Z数値、camera | 共通world position.z | ScaleとZの変更channelを混同しない |
| 複数layerを展開 | 奥行き展開 | rail範囲、分布 | 通常transformのD2 macro | helper/group/expressionを生成しない |
| 遮蔽を有効化 | Z Occlusion ON/OFF | policy/participant/bin | 明示depth policy | 座標、子順、Undoを変えない |
| 構図を維持してZ変更 | Preserve Appearance | 補正channel表示 | position.z + 通常transform補正 | 補正を隠さずCancel可能 |
| 3D背景へ2Dキャラ | 同じCanvasへ配置 | material/depth診断 | 同一world/camera | 別3D sceneや中間export不要 |

P2U/P2RはDirect/Tool、P2DはSimple/Advanced同一意味の審判である。P5の実素材検証では「完成した画」だけでなく、3D背景へ2Dキャラを配置し、前後関係を調整し、元へ戻す操作記録を残す。

## 6. フェーズ横断の出荷審判

M5完了時点で、代表操作コーパスについて次を満たす。

1. 作品を開いて、値の由来・scope・depth policy・欠落pluginをAdvancedへ入らず概略識別できる。
2. Direct/Toolで行った変更をAdvancedで検査でき、開閉だけではDocumentが変化しない。
3. 操作取消が1回のUndoで戻り、隠れhelperが残らない。
4. previewとexportが同じ評価意味を使う。
5. pluginが欠けても未知データを保持してProjectを開け、再現不能なexportだけを拒否する。
6. cache削除、Refresh、Bake、再起動を通常手順に含めない。
7. 未対応機能は無言の近似ではなく、何が不足しているか診断する。

この審判は「初心者が使えた」という主観だけでは完了しない。自動fixture、操作ログ、serialize差分、意味論golden、基準機測定、人間の認知確認を分けて証跡化する。

## 7. 非目標

- 全機能を1クリック化すること。
- Advanced機能を隠して存在しないように見せること。
- 競合と同じショートカット、画面、名称をそのまま複製すること。
- plugin市場の需要を無条件でHostへ焼くこと。
- クリック数を減らすために確認、診断、Undo、可逆性を削ること。
- v1で汎用node editor、文字列expression、永続Constraint Graphを導入すること。
