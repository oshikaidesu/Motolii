# CU-201E Timeline interval通常製品route E2E receipt

- 日付: 2026-08-01
- 状態: **E2E DONE**
- 親: CU-201 / U3b / VS-2
- route: 通常製品window `Motolii`

## 1. 完走結果

CU-201R完了後の現行binaryを一時Mac app bundleから既存projectへ接続し、通常製品windowで
保存済みの三つのRectangle、trim後の短縮bar、move後のoffset barを再表示した。

先行する同じ製品route実操作では、次を一続きに完走済みである。

1. native Timeline bar右端をdragしてtrim
2. 同じbar bodyをdragしてmove
3. Command+ZでmoveをUndo
4. Command+Shift+ZでmoveをRedo
5. windowを閉じ、同じprojectを再open

CU-201R後の再openでも結果が一致した。diagnostic画面、React mock、旧比較面は使っていない。

## 2. identity / interval / transientの証拠

journal tailのdurable commandは次の四件である。

- `TrimClipOut target=0 old_duration=10 new_duration=229/30`
- `SetClipStart target=0 old=0 new=17/15`
- Undo: `SetClipStart target=0 old=17/15 new=0`
- Redo: `SetClipStart target=0 old=0 new=17/15`

trim、move、Undo、Redoを通じてtarget LayerIdは`0`のまま。再open後のTimeline表示は
`start=17/15`と`duration=229/30`に対応し、別のClip copyを作っていない。

WALの可読command列にgesture、preview、logical px、layout epoch、projection generation、snap情報は0。
永続化されたのは既存Document commandの確定intervalだけである。

## 3. 固定証跡

- binary SHA-256: `cdf65d913d9f3a54e762cf5d41024fa6624e748e164d374d123928cd121d2375`
- project base SHA-256: `dc758e46300ddd19cc68d30ee1bb1fded8b71236a13b982a352dbdfd621a8176`
- journal SHA-256: `ff76d5868b26129f1b16d876bbd08e5eb8d4f746727c997293440bd0f4fa8e4a`
- app bundle ID: `com.motolii.cu201p`

一時bundleは検証器であり製品sourceではない。観測後にprocessを終了しproject lockを解放した。

## 4. 裁定

CU-201Eを`DONE`、全子粒を閉じた親CU-201 / M3 U3bを`DONE`とする。
これは固定Mac Local Alpha全体の完了を意味しない。Easing、transport、project lifecycle、export等の
残り製品接続は各authorityで別に閉じる。
