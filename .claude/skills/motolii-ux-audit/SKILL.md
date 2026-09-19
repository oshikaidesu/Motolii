---
name: motolii-ux-audit
description: Motolii の窓(Flutter, dark, 密)の違和感を、利用者が気付く前に監査して報告する。画面ごとに写真を撮り、Motolii 自身の裁定を物差しにして blocker / major / minor の表と file:line を出す。UI を触った後、panel を足した後、「なんか変」と言われた時に使う。code は直さない — 直すかどうかは利用者が決める。
---

# Motolii UX 監査

対象は **デスクトップのプロ道具**(Flutter / dark / 密。row 20 px、font 11、Material の語彙なし)。
一般の web・スマホの物差し(44 pt の指、余白たっぷり、CTA、変換率)は**そのまま当てない** — 下の Motolii の物差しが優先。

同梱の外部 skill を土台に使う:
- `.claude/skills/ux-audit`(MIT) — 写真から severity と heuristic 出典を付ける手順、`scripts/annotate.py` の注釈。
- `.claude/skills/ux-heuristics`(MIT) — Krug / Nielsen の severity と dark pattern の照合。

**この skill は報告だけをする。UX の合否は利用者の専権。** 裁定の無いまま code を書き換えない。

## 物差し(Motolii の裁定 = 破ったら所見)

出典: `docs/reviews/2026-09-19-design-craft-ledger.md`(50 規則)、`2026-09-19-gui-existing-devices.md`(守る順 5)、`2026-09-19-layout-panel-survey.md`(利用者との詰め)、`2026-09-19-daily.md`(決めた事)、`docs/ui-visual-language.md`。

| # | 規則 | 破りの見つけ方 |
|---|---|---|
| R1 | **籠と取っ手は意図で出す** — 触っている物だけ。カメラの gizmo は選択中だけ | 何も選んでいないのに Stage に取っ手・枠・gizmo が出ている |
| R2 | **数値は隠さない** — 消す・隠す・幼くするは禁止。値は常に読めて、打ち込める | 値がアイコンや絵だけになっている、hover しないと数が出ない欄 |
| R3 | **Layout は言葉を出さない** — 見えるのは数と pad と glyph だけ。CSS の語(flex / direction / justify)は裏(台本・書類・tooltip)へ | Inspector の Layout の行に英語の CSS 語が出ている |
| R4 | **群だけ・格子だけ** — Layout の親の行は群のみ。並べ方は columns × rows の 2 つの数だけ(横一列 = rows 1) | 文字層や単体の物に Layout の行が出る、direction の選択肢が UI に在る |
| R5 | **応えは 0.1 s 以内** — 入力 → 絵は 100 ms 以内。越えるなら代理表示、1 s 越えなら進捗。ドラッグ中にアニメを挟まない | 掴んで動かして絵が遅れる、panel の開閉が 100 ms を越す、drag 中に補間が入る |
| R6 | **当たり判定 24 px 以上** — 下限 24、基準 28 pt、縁なしのハンドルは周り 24 pt | 見た目 6〜8 px のハンドルに padding が無い、密な帯の分割線 |
| R7 | **色は theme の token だけ** — `EditorTheme` / `EditorInk`。`Color(0x…)` を panel に直書きしない | `dart run bin/check.dart lib` の `raw_color` が clean でない |
| R8 | **1 画面・page を切り替えない** — 所在は一覧で示す。panel は幅可変・全隠し可、ステージが主 | 全画面を覆う page・modal・wizard、戻らないと前が見えない造り |
| R9 | **Material の語彙を使わない** — `package:flutter/material.dart` を lib で import しない。`Icons.*` / `Colors.*` は `Glyph` / theme へ | `material_import` が clean でない。Material の形(FAB・Snackbar・Card の影)が見える |
| R10 | **EditorMetrics の密度** — row 20 / control 24 / section 26 / bar 28、font 11(micro 9 / dense 10 / title 13)。裸の数は `raw_dimension` が拒む | 行が 20 を越えて緩い、文字が 11 より大きい、`raw_dimension` が clean でない |
| R11 | **drag→preview→commit の契約** — 掴んでいる間の絵が確定値。Esc と focus 外れで取り消し、undo は 1 回で戻る | Esc が効かない、panel の外を押しても値が残る、1 つの drag で undo が 2 回要る |
| R12 | **重なりは影でなく白 8〜16% の overlay**、純黒 #000 の地を使わない、文字は 4.5:1 | 影で浮かせた panel、真っ黒の地、読めない灰色の文字 |
| R13 | **hover は色 1 段だけ** — 拡大・影・ばねを付けない。値の吹き出しは遅延 0 で出し、離れたら即消す | hover で部品が膨らむ、値に 500 ms の tooltip 待ちが掛かる |
| R14 | **UI 効果音を付けない** — 鳴るのは作品と再生だけ。エラーは Status の文字 | — |
| R15 | **選択は 1 色・スナップ線は別の 1 色** — 作品の色と混ざらない UI 色 | 選択枠の色が作品の色に紛れる、hover と選択が同じ太さ |

## 手つき

1. **窓を出す**: `scripts/motolii-ui.sh dev`(必要なら先に `native`)。利用者の作品を出すなら `dev <path>.rrd`。
   落ちた・build が通らない時は、そこで止めて報告する(監査より先に compile)。
   窓の起動・再起動・写真は**利用者の手**を借りる場合がある(実窓操作は拒否される事がある)。
2. **写真**: panel ごとに 1 枚 —— Stage / Camera / Inspector の Layout / Timeline / Browser / Desk。
   状態も撮る: 何も選んでいない時、1 つ選んだ時、drag の途中、空(empty)の棚、error の行。
   写真は `docs/reviews/<date>-ux-audit/assets/` へ。
3. **静の照合(写真の前でも後でも)**:
   `cd motolii/ui/tool/motolii_lints && dart run bin/check.dart ../../lib` →
   `raw_dimension` / `raw_color` / `material_import` が clean か。clean でない行は R7 / R9 / R10 の所見にそのまま乗せる。
4. **panel ごとに監査**: `ux-audit` の 4 パス(目的の歩き / 枠組みの掃き / 名前付きの点検 / 画面を跨ぐ一貫)を、上の R1〜R15 を**先に**当ててから回す。
   外部 skill の所見が Motolii の裁定と衝突したら(例: 「余白を増やせ」「44 pt にせよ」「文言を足せ」)、**Motolii が勝つ**。衝突は報告の末尾に 1 行で残す。
5. **写真から分からない物は断定しない**: 応えの速さ(R5)、Esc の取り消し(R11)、focus の順は、実際に触るか code を読むまで「未検証」と書く。
6. **表を出す**(下の型)。最後に、利用者へ聞く事を 3 つ以内にまとめる。

## 報告の型

```markdown
# Motolii UX 監査 <date>

対象: <panel の一覧> / 版: <git rev> / lint: raw_dimension … raw_color … material_import …

| 度 | # | panel | 破った規則 | 見えた物(写真) | 場所 |
|---|---|---|---|---|---|
| blocker | B-01 | Inspector/Layout | R3 言葉を出さない | 「Direction」の選択が行に出ている | motolii/ui/lib/panels/inspector.dart:412 |
| major | M-01 | … | … | … | … |
| minor | N-01 | … | … | … | … |

## 未検証(触らないと分からない)
- …

## 良い所(残す物)
- …

## 衝突(外部 skill と Motolii の裁定)
- …

## 利用者に聞く事(3 つ以内)
1. …
```

度の意味 —— **blocker**: 裁定を正面から破る / 触れない・取り消せない。**major**: 毎回の作業で当たる、目が止まる。**minor**: 気付くが進める。
場所は必ず `motolii/ui/lib/**` の `file:line`。行番号が曖昧な時は関数名を添える。

## やらない事

- code を書き換える(利用者の裁定の前)。
- 新しい名前・新しい意味を発明する(`docs/` の語を使う)。
- 一般の web/スマホの物差しで Motolii の密度・語彙を「直す」提案をする。
- 写真に見えない挙動を断定する。
