# 夜間自走の台本(2026-08-30)

- 位置づけ: 利用者が就寝中に進める作業の**範囲と柵**
- **利用者は1ユーザーの視点を保つため実装に首を突っ込まない。**朝に窓を触って合否を出す。
  だから「判断を仰ぐために止まる」のではなく、**当たり前の物は作って見せる**

## 柵(これを破るなら止まって朝を待つ)

1. **意味を触らない** — `motolii-store` の property 語彙・`Value`・`LayerSource` を増やさない。
   足りないと分かったら `FINDING` に書いて止める
2. **新しい概念を作らない** — 既にある口の組み合わせで書けない物は着手しない
3. **既決事項を先に引く** — `docs/motolii-deltas.md` の3審判表と「しない」節。
   **UI の分岐の審判は意図論であって Lottie ではない**
4. **窓で見えた物だけを「通った」と書く** — cargo 緑は通行証ではない
5. **全部積む** — 捨てる時も一度 commit してから

## 利用者が名指しした欠落(2026-08-30)

> ギズモとインスペクターの拡張かな、**今色変える部分ないしな、あと縁とか**、
> **ベジェ変形はブラウザから選択できるように**

どれも「AEユーザーが教わらずにやること」= 引き継ぐ側。モデルには既に在るのに UI が無い:

| 欠落 | モデル側の在庫 |
|---|---|
| **色** | `Value::Color([f64;4])`、`LayerSource::Solid{rgba}`、`TextDocumentStyle::fill` |
| **縁** | `TextDocumentStyle::{stroke_color, stroke_width, stroke_over_fill}`、`motolii_vector::Stroke` |
| **ギズモの拡縮/回転** | `scale`/`rotation`/`anchor` property は在る。ハンドルの当たり判定だけ無い |
| **ベジェ** | `Value::Path`、`PathSource::Bezier`、`motolii_vector::ShapeNode` |

既決事項: **固定標準 swatch は作らない**(`motolii-deltas.md`「しない」節)。

## 順番(上から。着いた所で止まる)

1. **Split at playhead** — 土台の一周の「切る」。**仮コードで測定済み: 新しい Intent が要らない**。
   `LayerTiming{start,duration,source_in,speed}` を2本へ割り、既存の `SetTiming`/`AddLayer`/
   `SetMeta`/`SetTrack`/`SetEffects` を並べるだけ。約30行。`LayerAttrs → LayerAttrsPatch` の
   変換口だけ無いので手で組む
2. **色と縁を Inspector に出す** — Solid の `rgba`、Text の fill/stroke。
   `Value::Color` は既に `lerp` を持つのでキーフレームも効くはず。**swatch は作らない**
3. **ギズモの拡縮/回転ハンドル** — 四隅=Scale、辺=片軸、枠の外=Rotation。
   書き先は既存の property、経路は今日通した transient overlay + `write_key` と同じ。
   **新しい機構を作らない**。Shift=等比/軸拘束、Alt=中心基準まで着けば上出来
4. **`increment()` が宣言の `MIN`/`MAX` を見る** — いま `intensity` を30px引くと `1.5 → 31.5`
   (宣言の範囲 0.0〜4.0 を無視)。今日見つけた綻び
5. **ベジェを Browser の Create から** — `PathSource::Bezier` で層を作る。
   S16(Text/Rectangle が生まれる口)と同じ形
6. **反射の通し検分** — `docs/ui-inherited-grammar-gap.md` の Tier 1 を台本に窓を実操作。
   直すのは配線だけの物に限る
7. **`vello_encoding config.rs:185` の減算オーバーフロー** — 仕切りを引きすぎると落ちる。
   再現条件の特定まで

## 触らない(利用者の裁定待ち)

- **裁定115**(層の姿勢が `Affine2`)— これが開かないと3D回転は空打ち
- **多段パス**の連鎖 — Triangle LED の本物(2パス)はここが要る
- **グループレイヤーの UI** — ⌘G の動詞は決まっているが、見え方は判断が要る
- `MAPS` の式展開(1 param → 複数 const)

## 朝に見せる物

- 触った物の一覧(commit の並び)
- **窓のスクショ** — 直した所と、直せなかった所
- 検分で出た欠陥一覧。**直していない物は「なぜ直さなかったか」付き**
