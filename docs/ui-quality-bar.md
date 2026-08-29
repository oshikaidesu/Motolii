# 「普通に使える」品質バー(quality bar)

- 制定: 2026-08-12(利用者目標「普通のUI・普通のUX・トンマナ維持・違和感ゼロ・常時滑らか・Stageリアルタイム追従・**触れる物は全部本物**」の数値化・規則化)
- 位置づけ: 全UI発注の常設oracle。トンマナ(色・密度・語彙)の正本は[ui-visual-language](ui-visual-language.md)/[ui-interaction-language](ui-interaction-language.md)であり、本書は**時間予算(B)と操作品質規則(Q)**を持つ
- 検収規則: UI系orderのoracleへ本書のB/Q番号を明記して引用する。予算・規則は反証可能 — 実測で満たせない場合は条件つきで本書を改訂する(黙って破らない)

## Q. 操作品質規則

### Q0. 触達性 — 触れそうな物は必ず機能する(最優先・利用者裁定 2026-08-12)

見えて押せそう/掴めそうなUI要素は**必ず実機能へ接続されている**こと。接続できない機能のchrome(ボタン・タブ・ツール・ダイヤル)は**置かない**(実装時に戻す)。「触れるかと思ったら触れない」はノイズであり、1件でも検収不合格。
- disabled表現が許されるのは「**今この文脈で無効**」の時だけ(例: 選択なし時のAdd Position Key)。「未実装だからdisabled」は不可(=撤去)
- fixture/デモ用の飾りを製品面に混在させない。host接続時はfixture UIを出さない
- **建設中の皮(probe)に在るダミーは、撤去対象ではなく実装目標として置いてある**(2026-08-29利用者裁定)。Q0が撤去を求めるのは**製品面**の死にchrome。皮のダミーを剥がすのではなく、繋いで本物にする

制定時点の違反inventory(全て「接続 or 撤去」の対象):
| 場所 | 要素 | 現状 |
|---|---|---|
| titlebar | `Settings` / `Export` | 死にtext |
| commandbar | ツール7種(↖✥◇T⌁Δ▣)・`COLOR BOOK`・breadcrumb | 死にtext |
| Stage tools | `Fit` / `100%` | 死にtext |
| transport | `\|‹ ▶ ›\|`・timecode・`DRAFT · FP16` | 死にtext(再生spine未接続) |
| Timeline左panel | KEYS/LAYERSモード・align/stagger/stretch群 | hint文言を変えるだけ |
| Inspector | Echo Bloom identity・Intensity/Spreadダイヤル・Blend行 | fixture(Documentに書かない) |
| Inspector | 日本語IME probe入力 | probe残置 |
| Browser MEDIA | 5000件のダミーasset・railフィルタ | fixture(import未実装) |
| Browser EFFECTS | railのCOLLECTIONS/TAGS/PACKS | 死にtext |

### Q0b. 触れる物は全て編集可能(2026-08-30利用者裁定)

**Q0の一段強い形。**「触れそうな物は必ず機能する」だけでなく、**触れる物は編集できる**。
読み取り専用の値を製品面に並べない — 見えているのに変えられないのは、
利用者に「ここは触っても無駄」を覚えさせる負債。

表示専用でよいのは**導出値**(行数・尺の合計など、元を変えれば従うもの)だけ。
元の値そのものが出ているなら、その場で編集できること。

制定時点の違反:
| 場所 | 違反 |
|---|---|
| Inspector COLOR行 | 読み取り専用(編集の入口をBrowser側に置いた設計。**その場で打てるべき**) |
| Inspector Z列 | 全行に `0.000` が出るが編集不可。**`property::` に `position.z` が無い**ので意味の追加が要る(裁定待ち) |
| Inspector 識別行 | 層名が出ているが rename できない |
| Inspector EFFECTS | param は触れるが、**エフェクト自体を外す/無効にできない** |

### Q1. 一貫した操作文法

click=選択、drag=移動、端drag=trim、release=確定、cancel=復元、double-click=生成/適用、Delete=選択物の削除、Cmd+Z/Shift+Cmd+Z=undo/redo。同じ見た目の物は面をまたいで同じ文法で動く(Timeline/Stage/Inspector/Browser)。

### Q2. 完全な可逆性

Documentを変える操作は**全て1回のUndoで戻る**。undo不能な破壊操作を置かない。

### Q3. 全操作に報酬(沈黙禁止の積極形 — 利用者裁定 2026-08-12)

**ユーザーの操作には必ず報酬(知覚できる応答)がある。触ったのに何も変化がないのは違和感であり、1操作でも不合格。**

- 接触の報酬: hover/pressの即時視覚応答(押した感・掴んだ感。カーソル変化・押下状態)
- 成功の報酬: 結果そのものが見える(置けば現れる、動かせば動く、消せば消える — 遅延なく、B4以内)
- 拒否の報酬: できない操作は**その場で理由が分かる**(黙って無視しない)。accepted-no-op系はUI側で入口を塞ぐ(範囲clamp・文脈disabled)か、視覚で応える
- 無反応の禁止: 「触れて・操作が成立して・何も変わらない」経路を作らない。変化ゼロが正しい操作(同値への編集等)でも、受理されたことは見える
- 切り捨て(cap超過)は必ず表示する
- mock／fixtureの説明文やhint変更は報酬に数えない。製品面のaffordanceはaccepted Document／Stage結果へ接続するか撤去する
- Stage操作の成功は同じStageの評価結果、拒否は同じ操作地点の理由表示で返す。panelだけの値変更や近似boundsだけの移動は成功報酬にならない

### Q4. 予測可能性 — preview = 結果

drag中のpreviewはrelease後のDocument結果と**同じ意味**(跳ね戻り禁止)。gestureの判定は決定的(タイミング依存で挙動が変わらない)。

### Q5. 単一の真実

選択・時刻・幾何の正本はhost(Document)一つ。全面(Timeline/Stage/Inspector/label)が同じ真実を表示し、二重帳簿を作らない。

### Q6. 頑健性

どの入力列でもpanic・クラッシュ・データ喪失を起こさない(gesture嵐testは常設回帰網)。render失敗はfallback表示で、**画面を空にしない**。

### Q7. 空状態の正直さ

空projectは空として表示し(幽霊番号・ダミー行禁止)、空でも全入口(place・scrub・keymap)が普通に機能する。

### Q8. トンマナ不変の機械検収

UI変更のorderは、意図した変更以外の見た目差ゼロを機械で示す(fixture preview PNG sha、新色・新styleゼロ、既存定数流用)。意図した変更はorder本文に明記された物だけ。

### Q9. キーボードとフォーカス

キーボードで到達すべき操作(undo/redo/delete)は面を問わず効く。TextInput編集中はテキスト編集が優先(構造保証)。フォーカスの所在は視覚で分かる。IMEを壊さない。

## B. 時間予算(display 60Hz基準)

| 項目 | 予算 | 測り方 |
|---|---|---|
| B1 定常フレーム | Timeline/Stageともrender thread CPU **p99 ≤ 8ms**、平均 ≤ 4ms | 既存 `[MotoliiRerunStage]`/`[MotoliiTimelineProbe]` telemetryへp99を追加 |
| B2 gesture中のフレーム落ち | drag/scrub/zoom中に**16.7msを超えるフレームを連続2枚出さない** | 同telemetryのgesture区間タグ |
| B3 入力→視覚(local preview) | pointer入力から当該surfaceの描画反映まで**≤1フレーム** | 構造保証(同tick反映)+計測 |
| B4 入力→視覚(host往復) | dispatch起因の状態反映(選択・編集結果)まで**≤2フレーム**(応答snapshot即時適用の現構造を維持) | 計測 |
| B5 Stageのリアルタイム追従 | scrub中、Stage実フレームの評価+描画を**p95 ≤ 16ms**(現行規模のDocument、DRAFT品質)。超えたら品質段階降下(解像度/品質)で**フレームは落とさない** | seam内計測 |
| B6 起動系スパイク | 初回shader/pipeline構築を除き、定常運転で**50ms超のフレームを出さない**。初回も500ms以内(warm-up先払い) | max telemetry |
| B7 メインthread停止 | UI入力threadを**4ms超**塞ぐ同期処理を置かない(JSON parse・lock待ち含む) | 計測+review |

制定時点の既知B違反①②③は2026-08-13の段差掃討wave Bで解消済み(実機実測: 初回スパイクStage 489ms/Timeline 175ms → max 20.7ms/1.3ms。経緯は[段差台帳](ui-friction-ledger.md)のwave B焼却記録)。

## 非目標

予算内での過剰最適化、トンマナ・レイアウトの変更(正本は視覚/操作言語)、120Hz最適化(別粒)。
