# 最小の核(提案)— 持つのは物の id と時刻だけ、あとは口の上

状態: **提案(2026-09-17 夜、利用者の裁定待ち)**。定規は [拡張の面の台帳(6)](2026-09-17-extension-surface-ledger.md)(AE の PF / AEGP / 式、AviUtl の anm / .auf)。背景は同日の対話: 「意味は元から外部(rerun の entity + time)にあった」「例外的な動作(データモッシュ)もコアの外」「そこからは本当の始まり」。

## 1. 核

**物の id と時刻。** それだけ。rerun の記録が entity の path と timeline しか持たないのと同じ。

- **id は path。** `/comp/box/text/word/3`。親が箱、兄弟の順が番号(index / num)。語も字も写しも物理の物も同じ木の葉。
- **時刻は唯一の時計。** 時刻を指せば画が 1 枚に決まる。物の in 点 = 生まれた時刻。法はそこから数える。
- **あとは全部 component。** 箱の矩形・変換・透明・色・画・札の値は、(id, 時刻) の上に置かれた数。書類が持つのは**作者が置いた値だけ**。法の出力は毎コマ計算し直す(純関数)。
- **法は component → component の関数。** taffy(子の大きさ → 位置)、Rapier(重さ・硬さ → 位置)、ブロック(番号・時刻 → ずれ)、Clip(箱 → mask)、補間(鍵 → 値)、解析(元素材 → 値)。全部口の上に置ける物で、核ではない。
- **記憶の置き場は 1 つ。** AE の `sequence_data` に当たる物 = PHYSICS(Rapier の焼き)と PERSISTENT の予約。増やさない。

## 2. 口 — AE と AviUtl の交わり(拡張が見る物 / 書く物)

| 口 | 拡張が受け取る物(定規の交わり) | 拡張が書く物 | Motolii の今 |
|---|---|---|---|
| **画を返す**(PF 効果 / .auf / pass) | 時刻、自分の画と大きさ、名指しした他の層の画(別の時刻でも)、host が補間した欄の値 | 画 1 枚(大きさは交渉) | ✓ ISF INPUTS、TIME_OFFSET / TIME_AT / LAYER / SOURCE、PASSES |
| **物を動かす**(anm の obj / ブロック) | 番号(index / num)、箱・部屋・隣、時刻、欄 | 変換 8 つ(位置・回転・拡大・透明、+aspect)を**書き戻す** | 読む ✓(`k`・`objects[k]`・`neighbor`)。書く △: translate + rotate だけ(scale は WorldPass で捨て、alpha・色は無し、block_program.rs:425) |
| **物を作る**(.obj / Repeater / Split / 個別オブジェクト) | 元の物 1 つ | 番号付きの物 n 個 | △ Repeater は札(Rust)、Split の単位は物にならない(blocks.rs:235 `copy == 0`) |
| **補間**(.tra) | t、区間の鍵の値列 | 値 1 つ | ✗ ease は固定名。ISF の型で `STAGE: ease` を足せば同じ棚 |
| **他を読む**(getvalue / CHECKOUT_PARAM / valueAtTime) | どの物の・どの時刻の値、元素材の他コマ、復号の副データ、音、道の点列 | — | △ 画は ✓(TIME_OFFSET・LAYER)。数値・音・道の点は無し。全部「値の形」で足り、核は増えない |
| **札の宣言**(--track0 / PF_Param) | 頭 1 行で欄を宣言、鍵と移動の法は host | — | ✓ vism の INPUTS。△ 箱の札は names.rs(Rust)= manifest に出せば file を置くだけ |
| **出口**(3 つ) | 画 1 枚 / 自分を描く / 書類を書く(AEGP・ExtendScript・台本 = 一度組む側) | | ✓ pass / block の Offset / 台本と Intent |

定規が「要らない」と言う物: 画素 1 つずつの API と DLL(pass が肩代わり)、内蔵効果を台本から呼ぶ(列に置けば足りる)、カメラ台本(Camera の札)、Lua の global(状態は焼くか閉形式)。「draw を何度でも」は Repeater の札に畳んである(意図的な差)。

## 3. 今日の穴は全部ここに落ちる

| 今日見つけた穴 | どの口の欠け |
|---|---|
| Split の単位が物にならない(証明の絵で語を 1 層ずつ書いた) | 物を作る: 単位が木の葉になれば番号も時計も付く |
| Overflow Clip が Arrive を切らない(行が箱の外へ飛ぶ) | 箱の component と物のずれの座標(wip で直し済み 0594ed6e7) |
| 効果の時計が in 点を知らない | 核の「in 点 = 生まれた時刻」 |
| lerp・弾性を 120〜181 鍵で焼いた(留め具の重さ) | 補間 / 物を動かす: 留め具は 4 つ目の住所 |
| 蛍光ペン・grid formations で色や大きさを場で変えられない | 物を動かす: 書ける先が位置だけ |
| データモッシュ・Blob・Tracery を core に入れそうになる | 他を読む: 元素材の他コマと副データを値として引く口 |

## 4. build の線

- **要らなくなる側:** 表現(wgsl)、場、ブロック、補間、解析、例外的な動作、台本、箱の札(manifest 化の後)。file を置けば載る。今日の見張りで wgsl は保存 → 数秒で絵。
- **残る側:** host そのもの(id と時刻の記録、fork の描画、借りた解き手 taffy / Rapier / cosmic-text)と、**口を切る数回**。
- 今日の build(Split・Clip・Loop・文字組み 3 法)は「口の上に置くべき物を Rust に書いた」build。AGENTS.md「先にその file を data / manifest の口にして、以後は行を足すだけで載る形にする」の通りにする。

## 5. 裁定を仰ぐ所(法)

1. **物を動かす口の書ける先** — scale・alpha を Offset に通す。色は pass に任せるか、ブロックからも書けるか。
2. **Split・Repeater の単位を木の葉(物)にする** — 番号と時計が付く。resolve の写しではなく entity。
3. **箱の札を names.rs から manifest へ** — 札を足すのに build が要らない形。
4. **他を読む口の形** — `getvalue(物, 属性, 時刻)` を値の口として。元素材の他コマ・副データ・音・道の点も同じ口。
5. **補間を拡張の単位に**(`STAGE: ease`)。
6. 既出: 留め具の重さ、傾いた箱の壁、飾りの子。

これが通れば、核の文書は「id と時刻、口 6 つ、出口 3 つ」の 1 ページで終わり、あとは口の上に置く物の一覧(棚)が増えるだけになる。
