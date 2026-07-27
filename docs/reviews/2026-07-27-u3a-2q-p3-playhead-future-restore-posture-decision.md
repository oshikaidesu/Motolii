# U3a-2Q-P3 playhead 将来 reopen 復元 posture 決定

- 日付: 2026-07-27
- 状態: **決定**
- U3a-2Q-P3: **DONE**

## 1. 目的と再分割理由

[U3a-2Q-P2](2026-07-27-u3a-2q-p2-playhead-reopen-lifetime-decision.md) 後の owner 採択 order は、
`Project session` と `Transient` を authority だけから一意に分けられず、Opus 5 が `ORDER: STOP` とした。
主担当 Codex はこの STOP を採用する。未決の discriminator を施工担当へ渡さず、本粒を次の一問へ再分割する。

> editor playhead の best-effort reopen 復元を、将来追加できる延期事項として残すか、恒久に棄却するか。

本粒は五層 owner を採択しない。`U3a-2Q-P4` が owner 採択だけを行う。

## 2. authority 事実

1. [G0-2 五層表](2026-07-16-m3-preflight-decisions.md#22-状態の持ち場と寿命)は、
   `Project session` を project identity 単位の best-effort cache、`Transient` を event / session 内だけで
   保存しない状態とする。
2. [U3a-2Q-P](2026-07-27-u3a-2q-p-playhead-owner-evidence-supplement.md) の `T1` は、
   P2 前の証拠だけでは owner を一意に導けないと確定した。
3. [U3a-2Q-P2](2026-07-27-u3a-2q-p2-playhead-reopen-lifetime-decision.md) §4 は、
   fresh Host coordinator が project を open した時に以前の値を復元せず、安全な初期位置へ戻す。
4. 同 §5 は、将来 best-effort 復元を追加する場合も安全な初期位置を欠落・破損時 fallback として保ち、
   owner、永続形式、version、未知 field 原本保全、reset、破損 fallback、配送経路を別審判で閉じるとする。
5. 同 §5〜§6 は P2 を owner 比較へ使う一方、P2 自体では owner を採択しない。
6. detach / re-dock / surface 再生成は project 再 open ではなく、同じ Host coordinator の単一 playhead を維持する。

## 3. 選択肢と反対側負例

| 選択肢 | 利点 | 反対側負例 |
|---|---|---|
| **延期・追加可能** | 現行 v1 の復元なしを保ったまま、将来の編集文脈復元を追加できる。P2 §5 の安全 fallback と段階的なformat審判を再利用できる | 未決事項を長く残し、将来 cache / version / corruption / reset / delivery の別審判が必要になる |
| **恒久棄却** | playhead の永続面を今後も要求せず、現在のno-restore規則を最小のまま固定できる | 実consumer evidenceなしに将来のUX改善を不可逆に閉じ、P2 §5 が残した追加経路を理由なく捨てる |

現行 v1 の挙動はどちらでも同じであり、fresh open では復元しない。ここで決めるのは将来能力の posture だけである。

## 4. 決定

editor playhead の best-effort reopen 復元は、**延期・追加可能**とする。恒久棄却しない。

これは復元機能の採択、実装予約、owner 採択ではない。現行 v1 は P2 §4 のとおり、fresh Host coordinator の
project open で以前の値を復元せず、安全な初期位置へ戻す。

将来復元を実際に採択するには、P2 §5 が列挙する owner、永続形式、version、未知 field 原本保全、reset、
破損 fallback、配送経路を別粒で閉じなければならない。未決のまま field、serde default、API、cache を足さない。

## 5. owner 比較への論理的帰結

- 本粒は五層 owner を**採択しない**。
- P2 の現行 no-restore と本粒の将来追加可能 posture の両方を満たす必要がある。
- `U3a-2Q-P4` は五層各行をこの二時点の寿命へ照らし、一層だけが残るかを `T2` で検証する。
- positive に決めた寿命との矛盾は候補比較へ使えるが、schema / code / caller の不在を排除証明へ昇格させない。
- `T2` で一層に定まらなければ owner を裁定せず STOP する。

## 6. `U3a-2Q-P4` entry gate

1. `U3a-2Q-P`、`U3a-2Q-P2`、本粒が `DONE`。
2. candidate は [U3a-2P](2026-07-27-u3a-2p-playhead-visible-range-scope-decision.md) §3 の五層だけ。
3. P2 §4 の現行 no-restore と本決定 §4 の将来追加可能 posture を独立した正の寿命証拠として使う。
4. 五層各行に採否と根拠節を一つずつ記録し、残存候補が厳密に一層である。
5. owner 以外の具体値、state shape、serialization、永続形式、公開 API、scrub / Cancel 意味、
   product surface、visible range を決めない。
6. `U3a-2Q-V` は `WAIT`、PRODUCT-ASSET lane の `DO` は `U3a-2Q-P4` 一件だけ。

## 7. STOP 条件

1. posture を閉じるために owner、値、shape、format、API、restore codec を決める必要が出た。
2. 外部製品または外部 model の助言を authority にしないと結論が立たない。
3. 本決定を使っても P4 の owner 候補が一層にならない。
4. U3a-2P §3 / §4 / §6、U3a-2Q-P E1〜E4 / T1、P2 §4〜§5、歴史 receipt の書き換えが必要になった。
5. scrub / Cancel の中間値寿命を決める必要が出た。
6. `U3a-2Q-V` を進める、または複数の次 `DO` が必要になった。

## 8. 必須負例

- **N1**: 「延期」を復元機能の採択、実装予約、Project session owner 採択と書く。
- **N2**: 現行 v1 が以前の playhead を復元すると書き、P2 §4を上書きする。
- **N3**: schema field、serializer、cache、API、restore codec、具体的な初期位置を決める。
- **N4**: `Project session` と `Transient` の定義を変更する、または第六層を足す。
- **N5**: code / caller / field の不在を候補排除の根拠にする。
- **N6**: detach / re-dock / surface 再生成を project 再 open と同一視する。
- **N7**: scrub Cancel 時の playhead、visible range owner、production pointer 入力を決める。
- **N8**: 外部助言を authority として引用する。

## 9. 次の最小粒

| ID | 状態 | 内容 |
|---|---|---|
| `U3a-2Q-P4` | **DO** | P2 §4 と本決定 §4を使い、五層から playhead state owner を一層だけ採択する docs 粒。五行の `T2` を必須とする |
| `U3a-2Q-V` | **WAIT** | actual consumer surface evidence 待ち（据え置き） |
| `U3a-2` 本体 | **WAIT** | 製品 window / consumer 入力待ち（据え置き） |

PRODUCT-ASSET lane の `DO` は `U3a-2Q-P4` ただ一件とする。
