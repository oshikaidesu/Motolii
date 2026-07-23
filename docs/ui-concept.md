# UIコンセプト — 表現をすぐ画にする制作面

日付: 2026-07-16

ステータス: **設計方針**。2026-07-22に旧仮説「1曲を演奏する譜面台」「First Beat」「楽曲が背骨」を[撤回](reviews/2026-07-22-ui-music-metaphor-retirement.md)し、音楽に依存しない体験像へ再構成した。凍結契約、Document schema、公開API、M3 taskの完了条件は本書だけでは変更しない。

関連: [concept.md](concept.md)、[specs/M3-ui-integration.md](specs/M3-ui-integration.md)、[VISION.ja.md](../VISION.ja.md)、[時間面UI構成モデル](ui-score-model.md)

## 0. なぜこの文書か

予防規律や拒否条件だけでは、初めて画面を開いた人に何が起きてほしいかを示せない。本書は、既存の製品判断をUI体験として束ねる。

以前はそれを「1曲を演奏する譜面台」と説明した。しかし、説明用の比喩がTimeline、Vism、plugin、初回制作順の意味へ広がり、音楽をMotolii全体の前提に見せていた。MV向けSoundtrack／BPMは具体機能として残し、UI全体は素材、生成、合成、時間編集、3D、pluginを等しく扱える中立な制作面として定義する。

## 1. 一文

**Motoliiの画面は、表現を作り、試し、組み合わせ、作品へまとめる制作面である。**

制作者はpluginやVismの実装方式を知らなくても意図した表現を見つけ、結果をすぐpreviewし、時間と空間の上で編集できる。作者はHost全体をforkせず、新しい表現を追加できる。Soundtrackは作品に必要なら使う時間guideであり、画面全体の所有者ではない。

5本の柱に展開する。

| 柱 | 一言 | 主な既存決定 |
|---|---|---|
| 1 | 結果と時間が見える | Preview、共通時刻`t`、一枚の時間面、任意のSoundtrack／BPM guide |
| 2 | 外殻は既知、因果は見える | 業界標準操作、型付きlink、由来と失敗理由の投影 |
| 3 | 語彙は意図 | 意図単位effect、性格で選ぶeasing、文脈別plugin呼称 |
| 4 | 密度は資産、装飾は負債 | 高密度一覧、意味色+形、装飾gradient禁止 |
| 5 | 軽さは機能 | 起動、操作応答、反復回数を守る性能目標 |

## 2. 体験の北極星: 最初の結果

初回起動から、**自分が加えた素材または表現の結果がpreviewで動く瞬間**までを「最初の結果」と呼ぶ。教材なしで次を完走できることを体験の北極星候補とする。

1. **起動する** — 数秒で編集可能になる
2. **素材またはgeneratorを選ぶ** — Browserから作品へ追加し、Stageと時間面に同じ対象が現れる
3. **値を変える** — Stageの直接操作またはInspectorで意図した変化を作る
4. **時間変化を加える** — keyframe、easing、driver等、対象に合う入口から変化を作る
5. **再生またはseekする** — PreviewとExportが同じ評価関数を通る結果を確認する

Soundtrackをdropし、BPM gridへsnapして音と同期させる経路は、この流れの有力なMV fixtureである。ただし唯一の初回経路にはせず、SVG、shape、generator、3D素材等から始めても同じ成功へ到達できるようにする。

## 3. 五本柱

### 柱1: 結果と時間が見える

Previewは作品の現在結果を示し、時間面は同じ時刻`t`にある素材、表現、key、readinessを示す。どちらもDocumentの投影であり、独自の正本を持たない。

- Soundtrackは設定時に波形を表示し、BPM／拍gridを時間guideとして使える
- Soundtrack未設定時に空の楽曲領域を強制せず、通常の時間編集を妨げない
- 時間code、frame、拍等の表示は同じ時間意味のviewであり、別clockや別Timelineを作らない
- Preview、Timeline、Inspector、Stageは同じstable ID、selection、revision付きsnapshotを読む

審判の問いは「何が選ばれ、いつ、何が結果へ効いたかを一画面から追えるか」である。

### 柱2: 外殻は既知、因果は見える

左に素材、中央にStage、右にInspector、下にTimelineという既知の外殻を使う。革新の予算は専用gestureではなく、scope、評価順、値の由来、失敗理由等、従来隠れていた因果を見せることへ使う。

### 柱3: 語彙は意図

UIに出す名詞は実装方式でなくユーザーの意図を表す。「グロー」「シェイク」「追従」「バウンス」のように結果から選べ、Advancedでは型付きparameterや実値を検査できる。拡張全体はplugin、Effect stackではeffect、生成入口ではgenerator、一時編集ではtoolと役割を示す。`Vism`は配布・開発者・file詳細の語彙であり、通常制作で暗記させない。

### 柱4: 密度は資産、装飾は負債

制作ツールでは、対象、状態、結果、戻し方が同時に読めることを優先する。頻用値を折り畳みの奥へ隠さず、位置、形、icon、意味色を併用する。親しみやすさを過剰な丸み、彩度、celebrationで作らない。

### 柱5: 軽さは機能

軽さは「今これを試す」を守る制作機能である。起動、drop後の反映、scrub、編集→比較→Undoの往復を測り、同じ時間で試せる案の数を増やす。平均fpsだけで、初回compile、同期readback、panel再生成等の中断を隠さない。

## 4. 画面の物語

```text
┌─────────────────────────────────────────────┐
│ Transport — 時間と再生の手元                  │
├────────┬──────────────────────┬─────────────┤
│ 素材と表現 │ Stage                │ 選択の意味    │
│ (左)      │ (中央: 結果と直接操作) │ (右: 由来と調整)│
├────────┴──────────────────────┴─────────────┤
│ 時間面 — 素材と表現の時間、key、状態を読む       │
├─────────────────────────────────────────────┤
│ 文脈Help — 対象、次の操作、実行できない理由       │
└─────────────────────────────────────────────┘
```

- **Project Explorer**: Project内の素材と外部filesystemを同じExplorerで探し、未配置素材はInboxへ受け取る
- **Stage**: 作品の結果と直接操作を同じ場所に置く。Draft等の品質縮退は正直に示す
- **Inspector**: 選択対象、値の由来、変形、Effect、driverを評価の流れに沿って示す
- **時間面**: 固定Lane所有を作らず、Document項目のbarを一枚の面へpackingする。Soundtrack波形やBPM gridは存在する時だけ同じ時間guideとして重ねる。詳細は[時間面UI構成モデル](ui-score-model.md)を正本とする
- **文脈Help**: hover／focus中の対象名、可能な操作、shortcut、実行できない理由を短く示す

固定座標、色、icon、px値は本書の対象外であり、[UI視覚言語](ui-visual-language.md)とG0系審判に従う。

## 5. 根拠の強さ

- **製品決定**: MV完成条件、任意のSoundtrackとBPM guide、共通時刻`t`、意図単位effect、型付きlink、高密度、軽さ、既知操作
- **体験方針**: 制作面という統合、最初の結果を動的審判にすること、五本柱の役割語
- **撤回済み**: 譜面台、First Beat、楽曲が背骨、波形領域の常設・非表示禁止

## 6. 状態と非目標

- 本書はDocument schema、公開API、plugin capability、永続設定を追加しない
- Soundtrack／BPMの既存意味や音声muxの完成条件を削らない
- VST類比からDAWのTrack、Mixer、instrument ownershipを導入しない
- 「制作面」「最初の結果」をそのまま製品UI labelへ固定しない
- renderer、React／native所有、OS window、Core／Host module／plugin分類は各責任境界文書で別に判定する

今後の反対側審判は、特定の音楽fixtureだけでなく、Soundtrackなしの素材起点、generator起点、3D素材起点でも「最初の結果」と同じ因果が読めるかを確認する。
