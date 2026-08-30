# 打鍵からの逆算 — AEと一般NLEの一次資料突き合わせ

- 日付: 2026-08-30
- 契機: キーが窓に届くようになった(`dee9c68d`)
- 一次資料: Adobe公式(AE [ショートカット一覧](https://helpx.adobe.com/after-effects/desktop/get-started/keyboard-shortcuts/keyboard-shortcuts-reference.html)・
  [層の属性](https://helpx.adobe.com/au/after-effects/desktop/work-with-layers/layer-properties/layer-properties.html)・
  [キーフレーム操作](https://helpx.adobe.com/after-effects/desktop/animate-in-after-effects/animation-keyframes/setting-selecting-deleting-keyframes.html)、
  Premiere [既定ショートカット](https://helpx.adobe.com/premiere/desktop/get-started/keyboard-shortcuts/default-keyboard-shortcuts.html))、
  Resolveは公式表を直接取れず二次資料
- 位置づけ: **測定であって裁定ではない。** 既定の打鍵は決めない(config化が利用者裁定で先行している)

## 初版(同日・記憶から作成)の誤り

1. **「`Cmd+K`は一般編集ソフトの共通打鍵」は測定でなく思い込みだった。**
   実際は Premiere=`Cmd+K` / Resolve=`Cmd+B`(または`Cmd+\`) / Final Cut=`Cmd+B` / AE=`Cmd+Shift+D` で、
   **分割に共通打鍵は存在しない**
2. **`P`/`S`/`R`/`T`/`A`と`U`を1項目にまとめていた。** 公式の定義では別物(下記)

## 二層の測り方が変わる

Resolveは**Premiere/Final Cut 7/Avidの打鍵配置をpresetとして読み込める**。
業界自身が「共通の打鍵」を決めずに**preset**で解いている。つまり:

**土台=打鍵の交差集合ではない。土台は「どのソフトにも在る*操作*」で、打鍵はpreset。**

これは利用者の既決(ハードコードせずconfigで差し替える)と同じ形であり、
初版の「交差集合から既定を導く」という方針は成り立たない。

## 衝突 — 同じ文字が世界をまたぐと別の意味になる

**逆算の最大の収穫はここ。** 既定を1つに決められない実証。

| 文字 | AE | 一般NLE(Premiere/Resolve/FCP) |
|---|---|---|
| `I`/`O` | 選択層の頭/尻へ**跳ぶ** | in/out点を**打つ**(mark) |
| `J`/`K` | 時間ルーラー上の前/次の項目へ跳ぶ(キー・マーカー・作業領域端) | `J`/`K`/`L`=**シャトル**(逆再生/停止/再生) |
| 分割 | `Cmd+Shift+D` | `Cmd+K`(Pr) / `Cmd+B`(Resolve, FCP) |

`J`/`K`/`L`のシャトルはNLEで最も普遍的な手癖の一つで、AEはそこに別の意味を置いている。
**AEをそのまま土台にすると、この層を丸ごと失う。**

## 属性を出す — 述語は引き継ぎ、文字と属性の1対1は引き継がない

公式の定義:

| 打鍵 | 公式名 | 出る物 |
|---|---|---|
| `P`/`S`/`R`/`T`/`A` | — | その属性(Position/Scale/Rotation/Opacity/Anchor Point) |
| `U` | Reveal Animating Properties | **キーまたは式を持つ**属性 |
| `UU` | Reveal Modified Properties | **既定値から変わった**属性 |
| `SS` | — | **選択中**の属性 |
| `Shift`+上記 | — | 出ている集合へ**足す** |

`U`/`UU`/`SS`は**属性を名指さない述語**で、属性が何個増えても効く。
`P`/`S`/`R`/`T`/`A`は**1属性=1文字**で、AEが変形属性を固定5個に凍らせているから成り立っている。

**Motoliiではvismのエフェクトが任意の宣言済みパラメータを持つので、1対1方式は最初の1本目で破綻する。**
篩はこの項目を割る: **述語は引き継ぐ、1対1は引き継がない。**
`U`/`UU`/`SS`はAnimationメニューに項目があり、打鍵だけの隠し機能ではない(発見可能性も満たす)。

## Motoliiの現状との差

現状の打鍵は`motolii/probe/src/keymap.rs`の10本、操作意図の`Intent`は7つ。

| 操作 | 現状 |
|---|---|
| 属性の絞り込み(述語) | **器ごと無い。** 層あたりの「今出ている属性」がtimelineに存在しない |
| 層の頭/尻へ跳ぶ | 尺は`placement`に在る。`Intent`追加のみ |
| 前/次のキーへ跳ぶ | キー時刻は読める。`Intent`追加のみ |
| 頭/尻を現在時刻へ揃える | trim機構は既存(`DragMode::Trim*`)。打鍵の口のみ |
| 複製 | `duplicate_track_item`が既存(在庫台帳L28) |
| rename | **無い**(Q0b違反として既出) |
| undo/redo | 機構は在る。打鍵は未接続 |
| シャトル(`J`/`K`/`L`) | 無い。**再生がついさっき通ったばかり**なので次の層 |

## 見えたこと

1. **器ごと無いのは属性の絞り込み1つだけ。** 残りは既存の器へ`Intent`と打鍵を足せば届く。
   しかもこの器は**Documentを変えない表示状態**なので、意味の裁定を待たずに作れる
2. **逆算で出たのは「入口の不足」であって「能力の不足」ではない**
3. **既定を1つに決める作業は、preset方式を採る限り不要**。決めるべきは*操作の名簿*の方

## 保留(利用者裁定)

- `N`/`B`(作業領域)がMV制作の背骨に要るか
- `Cmd+Y`(平面生成)— Createの口は既に在り、AEの語彙を置くかは意味の話
- 初期presetをAE寄りにするか一般NLE寄りにするか(衝突表の3行が具体的な分岐)

## 篩で落とした物

`Cmd+Shift+C`(precompose)— AGENTS.mdで名指しの除外。
