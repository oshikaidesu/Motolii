# UI操作言語 — 既知の外殻、可視の因果、裏切らない共通部品

日付: 2026-07-16

ステータス: **設計決定**。M3の操作互換性と共通component契約の正本。具体tokenは[UI視覚言語](ui-visual-language.md)、Document意味とDirect / Tool / Advanced正規化は[操作単純化モデル](interaction-simplicity-model.md)、実装タスクと審判割当は[M3仕様](specs/M3-ui-integration.md)を正本とする。成熟ソフトの更新から抽出した反証と受入観点は[UIアップデート考古学](reviews/2026-07-16-ui-update-forensics.md)を参照する。

## 1. 製品命題

MotoliiのUIは、**見た瞬間は知っている制作ソフトに見え、触ると従来より因果関係が分かる**ものにする。

- 外殻、基本配置、選択、drag、transport、Undo等は、制作ソフト間で既に学習された共通語彙へ乗る。
- 独自性は新しいpanel配置や専用gestureではなく、従来隠れていたtarget、scope、評価順、所有/共有、失敗理由を見えるようにすることへ使う。
- 一度学んだ操作規則を、別機能、Advanced、pluginが裏切らない。共通componentから漏れた実装は外観不良ではなく操作互換性の不具合として扱う。
- UIはDocumentの投影であり、別の制作データ正本、隠れhelper、入口別意味を持たない。
- 親しみやすさを、ポップな配色、派手なmotion、マスコット的装飾、ケレン味のある専用演出で代替しない。装飾を外しても、対象・結果・戻し方が分かることをユーザーファーストの基準にする。

```text
既知の制作ソフト外殻
        ↓
少数の共通操作文法
        ↓
型付きDomain Intent
        ↓
D2 Command / Document意味
        ↓
結果・由来・失敗を同じ画面へ投影
```

## 2. 参照の役割

### 2.1 主参照はプロ用ソフトの操作トポロジー

Ableton Liveから借りるのはDAW機能やArrangement Viewの画面構成ではなく、次の操作トポロジーである。

- 大半の作業を一つのmain screenで行い、各Viewの役割と出現位置を安定させる。
- 選択対象の詳細を決まった領域へ投影する。
- device chainの左→右のように、評価順と画面上の並びを一致させる。
- Browserからのdrag/drop、事前preview、並べ替え、折畳みを同じ文法で再利用する。
- Info Viewのように、hoverしたcontrol自身が名前と機能を説明する。
- global zoomと主要splitの調整を持ち、情報密度を固定pxの我慢へしない。

Abletonから得るもう一つの判断は、**機能数や演出量を多く見せることと、ユーザーファーストは別である**ということだ。表面の選択肢を抑えても、主要workflow、配置、評価順、feedbackが一貫していれば、道具は貧しくならない。Motoliiも「高機能に見せるためのcontrol」や「楽しそうに見せるための演出」を足さず、作品を作るために必要な意味だけを、予測可能な場所と操作で出す。

ただし、削ること自体を目的にしない。作品意味やadvanced用途を消して単純に見せるのではなく、同じ意味へ至る重複入口、説明不能なmode、局所UIを削る。必要な高度機能はAdvancedへ隔離するだけでなく、Simpleから存在と結果を確認できるようにする。

公式根拠: [Main Live Screen / Info View](https://www.ableton.com/en/live-manual/11/first-steps/)、[Browserのpreviewとdrag/drop](https://www.ableton.com/en/live-manual/12/working-with-the-browser/)、[Device Chainの左→右評価](https://www.ableton.com/en/live-manual/11/working-with-instruments-and-effects/)。

### 2.2 ゲームUIは操作中フィードバックの補助参照

ゲーム風の外観や大きなHUDは採らない。次の力学だけを補助参照にする。

| 参照 | 借りるもの | 借りないもの |
|---|---|---|
| Nintendo | 必要な状況だけ説明し、説明過多で操作を止めない | 娯楽向け演出、巨大prompt |
| CAPCOM / RE ENGINE | 多用途でも共通UIとworkflowを一貫させ、入口差を深さの削除にしない | title固有HUD、世界観固有装飾 |
| Xbox Accessibility Guidelines | 読めるtext、scale、contrast、focus、contextを測定可能にする | console前提の固定寸法の無条件移植 |
| Valve Steam Input | 物理入力よりAction / Action Setを正本にする | controller固有modeをDocument意味にすること |
| Naughty Dog / Riot | 操作上重要な対象を環境から分離し、重要情報を品質縮退で消さない | high-contrast演出の常時適用 |

公式根拠: [Nintendo UI/UX](https://www.nintendo.co.jp/jobs/keyword/112.html)、[CAPCOM RE:2019](https://www.capcom.co.jp/RE2019/)、[Xbox Accessibility Guidelines](https://learn.microsoft.com/en-us/xbox/accessibility/guidelines)、[Steam Input Actions](https://partner.steamgames.com/doc/features/steam_controller/iga_file?language=english)、[The Last of Us Part I accessibility](https://blog.playstation.com/2022/08/26/the-last-of-us-part-i-full-list-of-accessibility-features/)、[VALORANT gameplay clarity](https://www.riotgames.com/en/news/valorant-shaders-and-gameplay-clarity)。

## 3. 既知の外殻を守る

v1の既定配置は、既存の制作ソフトで学習済みの役割へ合わせる。

```text
上: transport / tool / project-level action
左: Asset / Plugin Browser
中央: Stage / Output Frame / direct manipulation
右: 選択対象のInspector
下: Timeline / time / order
```

固定分割で開始し、可変dockingを革新点にしない。panelの開閉や幅変更は許しても、同じ役割が機能ごとに別の場所へ移動しない。

次は原則として既存語彙を維持する。

- 選択→Inspector、drag/drop→配置または接続、drag→並べ替え、Space→再生/停止。
- Delete、複製、copy/paste、Undo/Redo、show/hide、lock、solo、snap。
- Timelineは左→右へ時間、stackは表示順または評価順を明示し、逆順を無表示で混ぜない。
- 無効なdrop先は反応を消すだけでなく、操作中なら拒否理由を返す。

既存製品が共有する配置やgestureから外れる場合は、単なる好みではなく、既存語彙ではDomain Intentを表せない証拠を要求する。

既知のshortcutは**既定値**であって契約ではない。Space、Delete、Undo/Redo、tool切替、snap、modifier+drag、接続開始等、製品内の全shortcutをユーザーが追加・置換・無効化できるようにする。機能は物理keyではなく`CommandId`/gesture intentを受け取る。設定画面の完成を待たず、version付きJSONを全機能へ届く正規fallbackとして提供する。OS/IMEが捕捉する組合せは別の固定操作へ黙って置換せず、利用不能理由を示す。

## 4. 少数の共通操作文法

全操作は、存在する範囲で次の流れへ揃える。

```text
Discover → Target → Preview → Commit / Cancel → Inspect → Undo
```

| 段階 | UI契約 |
|---|---|
| Discover | hover / label / Infoで名前と開始方法が分かる |
| Target | 現在の対象、期待型、scopeを識別できる |
| Preview | 確定前の値、配置、接続、画をTransientに示す |
| Commit / Cancel | 確定はD2 command、1 gesture=1 Undo、Escape/capture loss=変更ゼロ |
| Inspect | 結果、由来、接続、近似、errorを閉じたpanelでも要約表示する |
| Undo | Direct / Tool / Advancedの入口差にかかわらず同じ意味を戻す |

機能ごとに別の「追加」「接続」「選択」「確定」を発明しない。特殊なのが作品意味なのか、実装者が局所的な近道を選んだだけなのかをレビューで分ける。

## 5. 視覚動線と情報密度

### 5.1 場所と因果を一致させる

- 選択したobjectの詳細は常にInspectorへ出す。専用windowを唯一の編集口にしない。
- 順序が結果へ影響するものは、評価順と同じ方向へ並べるか、矢印とlabelで差を明示する。
- 参照元、参照先、共有definition、DataTrack等は、renameやtimeline順に依存しない接続として投影する。
- 値がどこから来たかを別画面で探させず、parameter近傍のbadgeからAdvanced詳細へ辿れるようにする。

### 5.2 現在操作中の情報へ面積を譲る

全機能を常時小さく並べない。情報量を時間方向に配分する。

| 状態 | 表示密度 |
|---|---|
| 平常時 | 小さなsemantic badgeで意味と異常を要約 |
| hover / focus | 名前、機能、開始方法をInfo表示 |
| drag / connection / direct edit中 | Stageまたはカーソル近傍へ説明、候補、仮線、ghost、数値を十分な大きさで昇格 |
| Advanced | 由来、scope、評価順、所有/共有、数値を十分な幅で検査・編集 |

小さなInspector文だけを唯一の操作説明にしない。global UI scale、主要panel幅、Timeline density、Stage overlayの可読性は分けて審判する。具体寸法はG0-2/G0-6で基準機、DPI、視距離を測って固定し、測定前の数値を本書では焼かない。

## 6. 説明付き接続

LookAt / Follow / Parent / DataTrack / Effect Use等は共通Connection Target Pickerを使う。接続mode中はカーソル近傍へ次を常時表示する。

- 何を変えるか。
- 何へ繋ぐか、または期待target型。
- 確定するとどうなるか。

`Idle → Picking → HoverValid / HoverInvalid → Commit / Cancel`をTransientな共通状態機械とし、valid target、invalid理由、仮線、確定後badgeを同じcomponentが持つ。button、whip、Canvas/Timeline clickは同じConnection Intentへ正規化する。詳細な境界は[操作単純化モデル S-3a](interaction-simplicity-model.md#s-3a-接続操作はカーソル自身が意味を説明する)に従う。

## 7. Simple / Advancedは同じ意味

Simpleは機能制限版、Advancedは規則を外す裏口ではない。

```text
Direct / Tool ─┐
               ├→ 同じDomain Intent → 同じDocument意味
Advanced ──────┘
```

- Directで作った結果をAdvancedで作り直さず検査・編集できる。
- Advancedを閉じても、出力へ影響する接続、scope、policy、由来、errorは要約表示する。
- Advancedで許せる例外は、型、scope、評価順、循環、複製、cache、preview/exportを宣言・試験できる追加意味だけ。
- `force connect`、文字列expression、名前検索、隠れcontroller、型検査解除はAdvanced例外に含めない。

## 8. 共通component契約

正規componentは見た目だけでなく、次の挙動をまとめて所有する。

- selection / hover / keyboard focus。
- enabled / disabled / warning / error / loading。
- label / tooltip / Info / screen-reader name。
- drag preview / Commit / Cancel / Escape / capture loss。
- D2 command、gesture merge、Undo単位。
- DPI / global UI scale / theme / contrast / reduce motion。
- typed target検査、拒否理由、欠落参照表示。

### 8.1 Silent disabledを禁止する

controlやtargetをgray/dimにするだけで説明を終えない。ユーザーが存在を認識でき、実行しようとする可能性がある操作を無効化する場合、同じcomponentが少なくとも次を返す。

- **何ができないか**: 拒否されたactionまたはtarget。
- **なぜできないか**: 型不一致、循環、選択不足、read-only、依存未完了等の具体理由。
- **どうすれば進めるか**: 必要な選択、解除操作、対応target、代替入口。回復不能ならその事実。

理由はhoverだけへ隠さず、keyboard focus、screen reader、接続/drag中のカーソル近傍説明からも到達可能にする。色、opacity、禁止cursorは補助表現であり、理由の代わりにしない。

次の2状態を混同しない。

| 状態 | 投影 |
|---|---|
| 現在の文脈と無関係で、存在を知らせる必要もない | 非表示にしてよい。ただしlayoutが不規則に跳ねないこと |
| 操作候補だが現在は実行不能 | disabled/dim + 理由 + 回復方法。操作中ならその場で表示 |

「接続できません」だけでも不十分である。`PositionはLayer targetを要求します / 選択中のAudio Trackは対象外です`のように、期待型と実targetを含む型付き診断を人間向け文へ投影する。UI文言をdomain errorの正本にはせず、同じtyped reasonから短文、詳細、screen-reader説明を生成する。

### 8.2 オブジェクト自身ではなく操作境界が診断する

Document内のLayerやTrackへUI説明責任を持たせない。複数object間の接続、編集、drop、削除等を審判するpolicy/preflightがread-onlyなDocument snapshotと対象IDを読み、成功時は準備済み操作、失敗時は領域固有の型付きrejectionを返す。

```text
Source ID ─────┐
Target ID ─────┼→ Policy / Preflight ─→ Prepared Operation
Arc<Document> ─┘                    └→ Domain Rejection
                                              ↓ adapter
                                      Diagnostic Envelope
                                              ↓ projection
                         Brief / Context / Inspect / Screen reader
```

これは概念上`Result<PreparedOperation, DomainRejection>`に相当するが、本節はRust公開signatureを凍結しない。重要なのは依存方向である。

- Layer等のDocument objectはUI文言、Slint型、tooltip、`CommandId`を知らない。
- `ConnectionRejection`、`EditRejection`、`DropRejection`等は各domainに置き、原因の構造を失わない。
- UI境界は領域固有rejectionを、小さな共通`Diagnostic Envelope`相当へ適応する。
- 全domain errorを一つの巨大enumへ集約しない。共通化するのは表示に必要な最小意味だけ。
- DiagnosticはTransientな値であり、Document、journal、Undo、cache keyへserializeしない。

共通envelopeが意味として持つ最小項目は次である。具体的なRust型と配置crateはU2c-4で既存error型を棚卸ししてから決める。

| 項目 | 意味 | 禁止 |
|---|---|---|
| stable reason code | 翻訳、test、同一診断追跡の鍵 | 人間向け英文をIDにする |
| action kind | 何を試みたか | mouse event列やbutton名 |
| subjects | 関係する安定object ID群と役割 | layer名/property path文字列を参照正本にする |
| typed facts | expected/actual型、循環経路、read-only理由等 | 文字列へ平坦化して原因構造を失う |
| recoverability | 回復可能、別操作が必要、回復不能 | 常に「再試行してください」で潰す |
| recovery candidates | 次に取りうるDomain Intent候補 | UI callback、物理key、暗黙の自動修復 |

recovery candidateは提案であり、診断表示だけでDocumentを変更しない。ユーザーが選んだ時に通常のIntent→D2 command→single writerを通し、その操作自身のUndo/Cancel規則に従う。

### 8.3 結果ではなく次の一手を段階投影する

同じ診断値を場所ごとに別実装せず、情報密度だけを変えて投影する。

| 段階 | 表示内容 | 用途 |
|---|---|---|
| Brief | 結果+最短の原因 | badge、status、一覧 |
| Context | 結果+原因+直近の回復方法 | cursor近傍、drag、connection、focus |
| Inspect | 関係ID、expected/actual、scope、評価順、回復不能理由 | Advanced、診断詳細 |
| Assistive | Context以上を順序立てた完全な文 | screen reader、keyboard-only |

予測可能な拒否はCommit後まで待たず、Target/Preview中に返す。invalid候補をdimにする場合も、hover/focus時点で同じ診断を表示する。実行後にしか分からない競合やstaleは結果時に表示するが、原因構造を一般的な「失敗しました」へ潰さない。

通常操作を成立させるために外部検索を要求しない。Help URLやmanualは追加学習の入口であり、原因と次の一手の代用品ではない。長文modalを常時出すことも目的ではなく、同じ診断をその場では短く、必要時だけ深く開く。

初期の共通語彙は少なくとも次を含む。

```text
InspectorSection
ParameterControl
ConnectionTargetPicker
TargetChip / SemanticBadge
DragPreview / GhostOverlay
TypedErrorBadge
DiagnosticBrief / DiagnosticContext / DiagnosticInspect
TimelineItem / EffectUseSlot
BrowserItem / DropTarget
```

個別機能はlabel、型、値、validation、Domain Intentまたはcommand factoryを渡し、hover、focus、Cancel、Undo、色、spacing、説明、error投影を再実装しない。共通componentで表せない場合は、component拡張を先に検討する。

## 9. 漏れた実装を完成扱いしない

一つの例外でも「他にも例外があるかもしれない」と学習させ、全機能の予測可能性を壊す。次をM3の受入条件とする。

1. 新規UIは既存componentのvariant→組合せ→新componentの順で判断する。
2. 新componentは状態matrix、keyboard、Cancel、Undo、error、scale、theme、accessibilityを同時に定義する。
3. 同じIntentの複数入口をconformance fixtureへ通し、Document意味、Undo回数、Cancel結果を比較する。
4. reference screenで既存componentと同居させ、追加分だけ別製品のように浮かないことを確認する。
5. theme外raw color、独自spacing、独自icon、直接的なSlint型流出を機械検査する。
6. componentを迂回する局所UIは、理由、非目標、再利用不能の証拠、正規componentへ戻す条件を記録する。
7. disabled/invalid fixtureはtyped reasonと回復方法を持ち、gray/dimだけの状態を拒否する。
8. 同じrejectionをBrief/Context/Inspectへ投影してもreason code、subject ID、typed factsが一致し、表示文字列を再解析しない。
9. recovery実行は通常のDomain Intentとsingle writerを通り、診断componentがDocumentを直接変更しない。

テスト緑だけで操作互換性の代わりにしない。一方、目視だけにもせず、状態matrix、操作列、serialize差分、screenshot/lightness/CVD、keyboard focus順、UI scale注入を分けて証跡化する。

## 10. Plugin UI境界

pluginごとの自由UIは共通文法から漏れる最大の経路である。

- 通常parameterは`NodeDesc`からHost標準panelを生成する。
- custom UI採用後も、全保存parameterをHost標準panelから検査・編集できるfallbackを残す。
- target picker、file/asset picker、keyframe、DataTrack、error、UndoはHost componentを使う。
- custom UIにしか存在しない保存値、plugin独自のDocument mutation、layer名検索、隠れcontrollerを禁止する。
- plugin固有表現が必要でも、Host shellのfocus、scale、theme、Cancel、error契約を迂回しない。

具体的なcustom UI能力はGAP-13の判定前に公開契約へ焼かない。

## 11. 非目標

- Ableton、任天堂、CAPCOM等の画面・asset・固有UIを複製すること。
- DAW、ゲームHUD、node editor、自由dockingをMotoliiの主操作モデルにすること。
- 全情報を大きくし、Timelineやparameter一覧の制作密度を失うこと。
- 逆に「プロ向け」を理由に、主要操作、説明、errorを小さいpanelへ押し込むこと。
- 全機能を1クリック化すること。
- 共通componentのために異なるDocument意味を同じ曖昧な操作へ畳むこと。
- Advancedやpluginを共通規則から逃げる場所にすること。
- ポップな色、過剰な丸み、bounce、celebration、巨大prompt等で、target、scope、評価順、失敗理由の不明瞭さを覆うこと。
- 機能数を多く見せるために、同じ意味のbutton、panel、mode、設定を増やすこと。
- 「業界標準」「安全」「実装都合」を理由に、一部shortcutだけを機能内へhard-codeすること。
