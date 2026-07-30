# CU-107W CU-107 W0表・CU-110依存の閉集合名反映
- 日付: 2026-07-28
- 状態: **決定**
- CU-107W: **DONE**

## 1. 目的
`CU-107N`で確定した4前提（`CU-107PV` → `CU-107TC` → `CU-107AD` → `CU-107TD`）をW0表と`CU-110`依存リストへ反映し、現況鏡像を同期して`CU-107W`を`DONE`へ閉じる。

## 2. 事実
- `CU-107N`は4前提の閉集合化と親待ち（`CU-107`/`CU-110`/`CU-111`/`CU-0B05`の`WAIT`据え置き）を採択。
- `CU-107R`は`CU-110`に必要な`CU-107`責任を厳密な部分集合へ限定。
- `CU-110D`は`CU-110`の`CU-107`依存を1前提ずつの分割へ置換する方針を採択。
- 事前実績として、`CU-107W`のlane行は`DO`、`CU-107N`までの発注依存証跡は`DONE`。

## 3. 裁定
- **W-1**: §8 W1表へ`CU-107PV`/`CU-107TC`/`CU-107AD`/`CU-107TD`の4行を追加する。4行は`CU-107N` §4 の順（`PV`→`TC`→`AD`→`TD`）で、`CORE / WAIT`、入力責任・出力責任・依存を`CU-107N` §4の逐語で転記する。
- **W-2**: 親`CU-107`は`CORE / WAIT`のまま据え置く。状態・依存・一成果・STOPを変えず、合格cellだけを4前提のroll-upへ書き換える。理由: `CU-107N` §8 handoffが`CU-107` = `WAIT` 据え置きを明記し、`CU-107N` N-6／`CU-110D` B-3 が本粒へ委ねた射程は**閉集合の名前への書換え**に限られる。状態変更は名前書換えではない。`SPLIT`等へ変えない。
- **W-3**: 親`CU-107`の合格cellをroll-upへ書き換える。4子行を追加した後も4前提そのものを親完了条件へ残せば`7` clause重複（N-3）と責任重複（N-4）に反するため、`CU-107N` N-6／`CU-110D` B-3が委ねる射程である名前置換範囲に限定する。`CU-107R` R-4の`D2未接続で検証`は`not load-bearing`の検証postureとして本行へ残す。
- **W-4**: `CU-110`の依存を`CU-102` + `CU-107PV`／`CU-107TC`／`CU-107AD`／`CU-107TD` + `CU-109`へ置換し、親`CU-107`名を除外する。
- **W-5**: `CU-110`の`WAIT`は解かない。non-test production drop sourceは`CU-110D`で据え置きとされているため、本粒では`CU-110`行へ追記しない。
- **W-6**: 停止線を3本そのまま維持する。`CU-0B05`は`CU-107`経由で4前提全依存を保持し未決と扱う、既存D&D spikeは製品到達性の証拠に数えない、`CU-110`の非test production drop sourceは`WAIT`維持。
- **W-7**: 次のPRODUCT-ASSET `DO`は**0件（未選定）**と裁定する。導出: 本粒完了後のPRODUCT-ASSET laneの非`DONE`・非`SPLIT`行はすべて`WAIT`である — `CU-107PV`/`CU-107TC`/`CU-107AD`/`CU-107TD`/`CU-107`は`CU-0B05`待ち、`CU-110`はnon-test production drop source待ち、`CU-111`は`CU-109`+U0c/U2b の製品接続待ち、`U3a-2Q-V`/`CU-106P`/`CU-106F`/`U2h-1P`は実consumer surface待ち、`CU-0A08BT`/`CU-0A08IT`/`U2c-2`は`U4a-2`/`U4c`製品入口待ち。authority上`DO`へ上げられる行が存在しないため、進捗継続のためだけのdocs粒を新設しない。表現は既存先例（`CU-104R`）の「未選定」を使い、状態語彙を新設しない。
- **W-8**: PRODUCT-ASSET 0件という事実を他lane（`G0-6H`/`VSM-A4S`/`GAP-25`/`P0I`）の状態変更に使わない。VS-1側閉塞は`CU-0B05`←`CU-0B04N/R`←`CU-0B02`/`CU-0B03`←`CU-0B01`としてのみ記録し、当該stateは維持する。

## 4. 反映後の写し
| ID | 種類 / 状態 | 一成果 | 依存 | 合格と必須負例 | STOP |
|---|---|---|---|---|---|
| `CU-107PV` | `CORE / WAIT` | 非空虚なpreview phaseが存在し、preview配送がterminalを生じさせずに完結すること | `CU-107`経由の既存D&D spike、`CU-0B05` | 一active dragの非terminalなpreview進行を受け取り、上記一成果を満たす。負例: preview配送へterminal生成の責任を持たせる、test / dummy / smoke / `#[cfg(test)]` / lint抑制 / env-gated smokeを到達性の証拠に数える | transport ID / drag epoch / layout epochをD2 / Document / journalへ保存したくなる、またはexact wire / event shape / verdict値 / 閾値 / 表サイズを決めないと成立しない |
| `CU-107TC` | `CORE / WAIT` | 各候補terminalへ、認可済みの非commit原因（Esc / outside / capture loss）のちょうど一つを付すか、そのいずれでもないと分類すること。分類は排他かつ網羅 | `CU-107PV`、`CU-107`経由の`CU-0B05` | `CU-107PV`が確立したactive dragに対して生じた候補terminalを入力とし、上記一成果を満たす。負例: 1つの候補terminalへ2つ以上の原因を付す、どの原因にも分類しない候補を残す、admissionまたは配送の責任を兼ねる | transport ID / drag epoch / layout epochをD2 / Document / journalへ保存したくなる、またはexact wire / event shape / verdict値 / 閾値 / 表サイズを決めないと成立しない |
| `CU-107AD` | `CORE / WAIT` | 一active dragにつきadmittedを高々1件に抑え、staleおよびduplicateの候補をadmitしないこと | `CU-107TC`、`CU-107`経由の`CU-0B05` | `CU-107TC`が「認可済み非commit原因のいずれでもない」と分類した候補terminalだけを入力とし、上記一成果を満たす。負例: exactly-onceを本前提単独の出力責任として書く（配送保証を`CU-107AD`へ持たせる）、`(webview_instance_epoch, drag_ordinal, event_sequence, layout_epoch)`を`相当`抜きの確定wireとして書く | transport ID / drag epoch / layout epochをD2 / Document / journalへ保存したくなる、またはexact wire / event shape / verdict値 / 閾値 / 表サイズを決めないと成立しない |
| `CU-107TD` | `CORE / WAIT` | admitごとにちょうど1回、単一の下流commit境界へ配送し、admitされていない候補を配送しないこと | `CU-107AD`、`CU-107`経由の`CU-0B05` | `CU-107AD`がadmitしたterminalだけを入力とし、上記一成果を満たす。負例: at-most-onceを本前提単独の出力責任として書く、admitされていない候補を配送する経路を残す | transport ID / drag epoch / layout epochをD2 / Document / journalへ保存したくなる、またはexact wire / event shape / verdict値 / 閾値 / 表サイズを決めないと成立しない |

`CU-107N` §4-2 のclause→owner写像は変更なし。

- (a)→`PV`
- (b)→`TD`
- (c)(d)(e)→`TC`
- (f)(g)→`AD`

## 5. 非目標
- exact wire、event shape、WebView contract、exact dedupe tuple、verdict enum／値、公開API名、bounded table size、閾値、rejection precedence、表サイズの決定。
- Document意味、journal形式、永続serde面、公開API、plugin契約、Undo/Redo、`RAW` mutation APIの決定。
- `CU-107N`の4 owner／7 clause／一本鎖を改変しない。
- `CU-107W`範囲外の他ID・隣接行の状態変更や到達性証拠の改変。

## 6. 必須負例
- `CU-107`行の他列を変更しない。
- `CU-107W`の追加4行を4行以外の件数で追加しない。
- `CU-107`/`CU-110`/`CU-111`/`CU-0B05`の`WAIT`を解かない。
- `CU-110`に親`CU-107`名を残さない。
- PRODUCT-ASSET laneに0件以外の新規`DO`を生やさない。

## 7. STOP条件
- `CU-107W`反映で`CU-107`/`CU-110`/`CU-111`/`CU-0B05`の既存状態を変える必要があると読む場合。
- `CU-0B05`や既存D&D spikeを解決済みと扱わざるを得ない場合。
- `CU-107PV`〜`CU-107TD`の4前提列挙が`CU-107N` `§4`の逐語と一致しない場合。
- 本粒で到達性証拠をtest/dummy/smokeで代替しようとする場合。

## 8. handoff
| ID | 状態 |
|---|---|
| CU-107W | DONE |
| CU-107PV | WAIT |
| CU-107TC | WAIT |
| CU-107AD | WAIT |
| CU-107TD | WAIT |
| CU-107 | WAIT |
| CU-110 | WAIT |
| CU-111 | WAIT |
| CU-0B05 | WAIT |
| 次PRODUCT-ASSET `DO` | 未選定（0件） |
