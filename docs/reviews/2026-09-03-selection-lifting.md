# 動詞を選択へ持ち上げる — 各アクションが他の全アクションを考慮しなくて済む縫い目

- 日付: 2026-09-03
- 位置づけ: **設計提案**(コードに触らない上流工程)。判断済みと裁定待ちを分ける
- 発端: 利用者「各々のアクションが全てのアクションを考慮しなくても良いようにしたい。引き継ぎ形の物
  (複数選択で M を押すと全部に掛かる等)は実装できているか」→ 未実装(§1)。
  「その形を発明にせず、外部の思想に委託しておきたい」→ §0

## 0. 外部の思想への委託(発明しない)

| 借りる物 | 出典 | ここで担う役 |
|---|---|---|
| **Composite pattern** | Gamma, Helm, Johnson, Vlissides『Design Patterns』(1994) | 「1 つ」と「集まり」を同じ口で扱う。**動詞は対象 1 つに対して書き、集まりは同じ interface で受ける**。「各アクションが全てを考慮しなくてよい」は本 pattern の定義文そのもの |
| **Command / MacroCommand** | 同書 | 操作を値にして、複数を 1 つの undo 単位に束ねる。Motolii の `Intent` と `apply_all` が既にこれ。**MacroCommand を組む場所を 1 箇所にする**だけ |
| **lifting**(`map`) | 関数型の常識 | Composite を型で言い直した物。1 層用の関数を集合へ持ち上げる |
| **multi-object editing** | Blender Manual「Multi-Object Editing」(2.8〜) | operator は 1 オブジェクト用に書き、context の selected objects へ harness が回す。**対象の解決を 1 箇所で行い動詞は知らない**運用の実物 |
| **相対 / 絶対の慣習** | AE・Premiere の複数レイヤー編集(ドラッグ = 各層の元値に相対、数値入力 = 絶対) | 動詞が宣言する **1 ビット**の出典 |
| **混在値の表示** | Unity Editor `EditorGUI.showMixedValue` | 値が混在していれば「—」を出し、打ち込めば全員に入る。Inspector 側の表示規則 |

自前の規則はこの表の外に作らない。分類としては、構造 = GoF、運用 = Blender/AE、表示 = Unity。

## 1. 現在地(2026-09-03、`main` c2c7a89 を読んだ事実)

複数選択の型(`Selection` = `Vec<LayerId>`、末尾が主選択、2026-08-30)は在るが、**消費する側が無い**。
`selection.all()` を読むのは PR #479 で足した Delete / Duplicate だけ。

| 面 | 動詞 | 今の対象 | 場所 |
|---|---|---|---|
| Timeline 層行 | M / S / L | **クリックした行だけ** | `timeline_shell.rs` glyph の onclick(`layer` 固定) |
| Timeline 帯 | Move / TrimStart / TrimEnd | 掴んだ帯だけ | `timeline_widget.rs` `DragState.layer` |
| Stage ギズモ | Move / Rotate / Scale | 主選択だけ | `stage_widget.rs:392` `selection.get()` |
| Inspector | 値のスクラブ・打ち込み・key ◇・色 | 主選択だけ | `app.rs:222` が `selected()` を渡す |
| Browser | エフェクト追加・色の適用 | 主選択だけ | `browser.rs:388,432` `selected()` |
| 打鍵 | Split | 主選択だけ | `dispatch.rs` |
| 打鍵 | Delete / Duplicate | 選択全体 | `dispatch.rs`(#479) |

## 2. 縫い目は 2 つの関数だけ(判断済み: §0 の写し)

```text
targets(clicked: Option<LayerId>) -> Vec<LayerId>
    clicked が選択に含まれる     → 選択全体
    clicked が選択に含まれない   → [clicked] だけ(選択は変えない)
    clicked 無し(打鍵・Inspector)→ 選択全体

lift(verb, targets) -> Vec<Intent>
    for layer in targets: intents.extend(verb(layer))
    doc.apply_all(intents)          // 1 ジェスチャ = 1 undo(Q2)
```

- **動詞は `fn(&StoreView, LayerId, payload) -> Vec<Intent>` の形で 1 層分だけ書く**。複数選択の存在を知らない。
- **動詞が宣言するのは 1 ビット**: payload が**絶対**か**差分**か。
  - 絶対: 「クリックした行の新しい状態」を全員に書く。M が 3 つ ON・1 つ OFF の状態で押しても全員が揃う。
  - 差分: 各層が**自分の元値**に足す。帯のドラッグ・ギズモ・スクラブ。差分の元値は drag 開始時に各層分を取る
    (今の `DragState.orig` を `Vec` にする)。
- **Q4(preview = 結果)**: drag 中の transient overlay も同じ `lift` を通す(各層に overlay を置く)。
  確定と preview で対象集合が違う経路を作らない。
- **拒否は層ごと**: 1 層が拒んでも(frozen 等)他は通し、拒んだ層の理由をその場で返す(Q3)。
  `apply_all` の all-or-nothing が要る動詞(Split の head/tail)は**1 層の中だけ**で束ね、層をまたいでは束ねない。

## 3. 動詞台帳(ほぼ全てに適用する)

| 動詞 | 面 | 対象の解決 | 絶対 / 差分 | Intent | 備考 |
|---|---|---|---|---|---|
| M / S / L | Timeline 行・Inspector | `targets(clicked)` | 絶対 | `SetAttrs{patch}` | 押した行の新しい値を全員へ |
| 2D / 2.5D / 3D(カメラ設計 §4) | Timeline 行・Inspector | `targets(clicked)` | 絶対 | `SetAttrs{patch}` | 同上 |
| 帯 Move | Timeline | `targets(掴んだ帯)` | 差分(フレーム) | `SetTiming` + key 追随の `SetTrack` | 各層の `orig` に足す。key 追随も層ごと |
| 帯 Trim(頭 / 尻) | Timeline | `targets(掴んだ帯)` | 差分 | `SetTiming` | clamp は層ごと(Media の壁は素材ごとに違う) |
| ギズモ Move | Stage | `targets(掴んだ層)` | 差分(comp px) | `SetTrack(position)` | 各層の元 position に足す |
| ギズモ Rotate | Stage | 同上 | 差分(度) | `SetTrack(rotation)` | **回転の中心は各層自身の anchor**(選択の重心ではない。AE と同じ) |
| ギズモ Scale | Stage | 同上 | 差分(倍率) | `SetTrack(scale)` | 各層自身の固定点 |
| Inspector スクラブ | Inspector | 選択全体 | 差分 | `SetTrack(prop)` | 混在なら「—」を出し、スクラブは各自の元値に足す |
| Inspector 打ち込み | Inspector | 選択全体 | 絶対 | `SetTrack(prop)` | 混在でも打てば全員に入る(Unity) |
| key ◇ 追加 / 削除 | Inspector | 選択全体 | 絶対(現在値) | `SetTrack` | 各層の**自分の現在値**で key を打つ |
| 色の適用 | Browser / Inspector | 選択全体 | 絶対 | 色の `SetTrack` | 対象に色系 property が無い層は拒否理由を返す |
| エフェクト追加 | Browser | 選択全体 | 絶対(末尾へ追加) | `SetEffects` | 各層の**自分の**列の末尾へ |
| Split | 打鍵・右クリック | 選択全体 | 絶対(playhead) | `SetTiming`+`AddLayer`+… | playhead が尺の外の層は no-op を報酬付きで |
| Delete | 打鍵・右クリック | 選択全体 | — | `RemoveLayer` | 実装済み(#479) |
| Duplicate | 打鍵・右クリック | 選択全体 | — | `AddLayer`+`SetMeta`+… | 実装済み(#479)。写しの集合を新しい選択にする |
| 層の頭 / 尻へ跳ぶ | 打鍵 | **主選択のみ** | — | (clock) | 跳ぶ先は 1 つしか無い。**選択全体にしない例外**、台帳に明記 |
| rename | Inspector | **主選択のみ** | — | (rename 未実装) | 名前は 1 つ。例外 |

例外は 2 つ(跳ぶ・rename)で、どちらも「結果が 1 つしか存在しない動詞」。それ以外は全て `lift` を通る。

## 4. 不変量(台帳へ足す行)

| 不変量 | 測り方 |
|---|---|
| **動詞 v を選択 S に掛けた結果 = S の各層に v を 1 つずつ掛けた結果** | `Document` 水準。S = {a, b} へ v → S = {a} へ v、S = {b} へ v を順に、で同じ Document |
| **1 ジェスチャ = 1 undo(層数に依らない)** | S の大きさを変えても undo 1 回で全部戻る |
| **絶対動詞は混在を揃える** | M が混在する S に M を押すと全員が押した行の新しい値 |
| **差分動詞は差を保つ** | S の 2 層の position の差は Move の前後で不変 |
| **拒否は局所** | S の 1 層が拒んでも他の層には掛かり、拒否理由が返る |

## A. 利用者裁定待ち

| 件 | 分岐 |
|---|---|
| クリックした行が選択に**含まれない**時 | 推し = その行だけに掛け、選択は変えない(Blender/Finder)。反対側: その行を選択に足してから掛ける(AE) |
| ギズモ Rotate/Scale の中心 | 推し = 各層自身の anchor(AE)。反対側: 選択の重心(Figma/Illustrator の group transform) |
| 混在表示のグリフ | Unity は「—」。M/S/L の glyph は半点灯か |

## B. 実装順(器が在る)

1. `targets` と `lift` を `dispatch.rs` に置く(#479 の `run_intent` の隣)。Delete / Duplicate をこれに乗せ替える
2. M / S / L と 2D/2.5D/3D(絶対の代表)
3. 帯 Move / Trim(差分の代表。`DragState.orig` を `Vec`)
4. ギズモ 3 種
5. Inspector(混在「—」+ スクラブ差分 + 打ち込み絶対)
6. Browser の色・エフェクト

1 が済めば、以後の新しい動詞は「1 層分の関数 + 絶対/差分の 1 ビット」を書くだけになる。
