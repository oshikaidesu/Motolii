# U3a-2Q-P2 playhead 再 open lifetime 決定

- 日付: 2026-07-27
- 状態: **決定**
- U3a-2Q-P2: **DONE**

## 1. 目的と非目標

[U3a-2Q-P](2026-07-27-u3a-2q-p-playhead-owner-evidence-supplement.md) §10 が残した一点、
editor playhead を project の再 open 時に復元するか、安全な初期位置へ戻すかを決める。

本粒は五層 state owner、値 shape、初期位置の具体値、復元規則、serialization、永続形式、公開 API、
`DomainIntent`、Document / journal / Undo 意味、Transport の seek / seed、製品 surface を決めない。
visible range と `U3a-2Q-V` も扱わない。

## 2. authority から引いた事実

1. [快適利用 work-map](2026-07-22-m3-comfortable-use-work-map.md) §7 は、playhead の runtime owner を
   Host coordinator とし、再 open 時の永続化を本粒まで未決としていた。
2. [G0-2 五層表](2026-07-16-m3-preflight-decisions.md#22-状態の持ち場と寿命)は、Project session を
   project identity 単位の best-effort cache、Transient を event / session 内だけで保存しない状態とする。
   永続化形式を U0b で発明せず、保存実装時の version、未知 field 原本保全、reset、破損 fallback は
   別タスクで閉じる。
3. [detachable / multi-window 契約](2026-07-22-m3-detachable-panel-window-contract.md) §1〜§4 は、
   playhead を window ごとに複製せず、全 window が Host の同じ revision 付き snapshot を
   read-only 投影するとする。
4. `crates/motolii-transport/src/lib.rs` の Transport は再生中の audio clock owner であり、
   editor playhead の再 open、paused seek、永続化を決めない。
5. [M3 仕様 U2b](../specs/M3-ui-integration.md) は Rectangle Place が playhead を入力として読むとするが、
   owner、初期値、再 open 規則は決めない。
6. [U3a-2Q-P](2026-07-27-u3a-2q-p-playhead-owner-evidence-supplement.md) §5 の `T1` は、
   E1〜E4 だけでは五層 owner を一意に導けないと判定した。同 §10 は本粒へ lifetime 判断だけを委ねた。

## 3. 選択肢と反対側負例

| 選択肢 | 利点 | 反対側負例 |
|---|---|---|
| best-effort 復元 | 長い編集の文脈を保ち、再 open 後の再 orient を減らせる | 未決の永続面、破損 fallback、復元値の配送経路を v1 背骨へ要求しやすい。再生中 clock を editor head の seed と誤認する圧力も生む |
| 復元せず安全な初期位置へ戻す | 永続形式・復元 API を要求せず、欠落・破損・fresh coordinator が同じ開始規則になる。将来の復元は追加的に導入できる | 再 open 後に作業位置を探し直す UX 負担があり、Timeline scroll / zoom の best-effort 復元と体感が揃わない場合がある |

復元案の UX 利点は棄却理由にしない。v1 の背骨では、未決の永続面を先に固定しないことと、
将来の復元を追加的に導入できる可逆性を優先する。

## 4. 決定

fresh な Host coordinator instance が project identity を open した時、editor playhead は以前の値を
**復元しない**。再 open 直後の観測値は、その project から決定的に定まる安全な初期位置とする。

ここでいう再 open は、project identity の open により fresh な Host coordinator instance を構築する境界を指す。
window の detach / re-dock / close / 再表示、panel の tab 切替、同じ Host coordinator instance 内の
surface 再生成では本決定を発動せず、単一 playhead の read-only 投影を維持する。

初期位置の具体値は決めない。「0 秒」「composition 先頭」「work area 先頭」等を本決定から推測してはならない。

## 5. 境界と論理的帰結

- 本決定は五層の特定行を state owner として**採択しない**。owner の正式採択は `U3a-2Q-P3` が行う。
- ただし五層が寿命で区別される以上、本決定が owner 候補の比較へ影響する論理的帰結を隠さない。
  `U3a-2Q-P3` は本決定を独立証拠として使い、再 open 復元を必要条件とする候補を無条件に採ってはならない。
- 本決定は Transport の再生中 clock、`PlaybackCounters`、`FramePlan.timeline_time` を変更せず、
  editor playhead の seek / seed API を要求しない。
- 本決定は Document への採否、Undo / journal、Rectangle Place の `playhead〜composition end` 意味を変更しない。
- 将来 best-effort 復元を追加する場合も、本決定の安全な初期位置を欠落・破損時 fallback として保ち、
  owner、永続形式、version、未知 field 原本保全、reset、破損 fallback、配送経路を別の決定と審判で閉じる。

## 6. `U3a-2Q-P3` entry gate

次をすべて満たす場合だけ playhead owner 採択粒を起票する。

1. `U3a-2Q-P` と本 `U3a-2Q-P2` が `DONE` で、前者の E1〜E4 / T1 と本決定 §4〜§5を参照できる。
2. candidate は [U3a-2P](2026-07-27-u3a-2p-playhead-visible-range-scope-decision.md) §3 の五層だけとし、
   Host coordinator、native module、React、Transport を第六層以降へ足さない。
3. runtime owner、surface owner、playback clock owner、state owner を分離したまま一層を採択できる。
4. state shape、初期位置の具体値、serialization、永続形式、公開 API、製品 surface を同時決定しない。
5. `U3a-2Q-V` を `WAIT` のまま維持し、PRODUCT-ASSET lane の `DO` を `U3a-2Q-P3` 一件にできる。

## 7. STOP 条件

1. 本粒を閉じるために五層 owner、初期位置の具体値、state shape、serialization、永続形式を決める必要が出た。
2. reset を成立させるために Transport / UI / Document へ seek、seed、setter、field、公開 API を足す必要が出た。
3. detach / re-dock / surface 再生成を project 再 open と同一視しないと文章が成立しない。
4. `U3a-2Q-P` の E1〜E4 / T1、`U3a-2P` §3 / §4 / §6を書き換える必要が出た。
5. visible range owner、`U3a-2Q-V`、production pointer 入力、製品 window を進める必要が出た。
6. 外部 model の助言または外部製品の挙動を authority として引用しないと結論が立たない。
7. `U3a-2Q-P3` 以外の新 ID、または複数の次 `DO` が必要になった。

## 8. 必須負例

- **N1**: 初期位置を 0 秒、composition 先頭、work area 先頭等の具体値で決める。
- **N2**: reset を Transient、復元を Project session と同義にして、本粒で owner を採択済みと書く。
- **N3**: Transport の clock owner を editor playhead の state owner とする、または復元用 seed API を予約する。
- **N4**: window detach / re-dock / close / 再表示、surface lost を project 再 open として reset する。
- **N5**: playhead を Document / journal / Undoへ入れる、または Rectangle Place の既決意味を変える。
- **N6**: state shape、serialization、serde default、永続 workspace / session 形式、復元 codec を決める。
- **N7**: UX 利点だけで best-effort 復元を既定化する、または UX 負担を存在しないものとして扱う。
- **N8**: `U3a-2Q-V` を `DO` にする、visible range の lifetime / owner を先取りする。

## 9. 次の最小粒

| ID | 状態 | 内容 |
|---|---|---|
| `U3a-2Q-P3` | **DO** | `U3a-2P` §3 五層から playhead state owner を一層だけ採択する docs 粒。`U3a-2Q-P` E1〜E4 / T1 と本決定 §4〜§6を使う。state shape、初期位置の具体値、serialization、製品 surface は束ねない |
| `U3a-2Q-V` | **WAIT** | actual consumer surface evidence 待ち（据え置き） |
| `U3a-2` 本体 | **WAIT** | 製品 window / consumer 入力待ち（据え置き） |

PRODUCT-ASSET lane の `DO` は `U3a-2Q-P3` ただ一件とする。
