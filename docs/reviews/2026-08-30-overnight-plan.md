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
利用者「色とかは一覧で見たいからここもブラウザで選択できるようにすべきやね」— **Browser が入口**。
ただし一覧は「**この作品で使われている色**」であって出荷時の標準パレットではない(上の既決事項と両立させる)。

### 仮コードで測った結果(色)

**色は property ではなく素材側の静的値。**書き口が対象ごとに違う:

| 対象 | 書き口 | キーフレーム |
|---|---|---|
| Solid の色 | `Intent::SetSource{ source: LayerSource::Solid{rgba,..} }` | **不可** |
| Text の fill / stroke | `Intent::SetTextDocument`(`TextDocumentStyle::{fill,stroke_color,stroke_width}`) | **不可**(組版は静止、`content` だけ時間変化) |
| Shape の fill / stroke | `Intent::SetShapes`(`motolii_vector::{Fill,Stroke}`) | **不可** |

`property::` に色系の定数は**1つも無い**。`Value::Color` は在るのに property として使われていない。

- **色を変える UI は今すぐ作れる**(3つとも書き口がある)
- **色をキーフレームで動かすのは意味の追加**になる。柵1(意味を触らない)に当たるので**夜間には入れない**。
  必要なら `FINDING` に書いて朝の裁定を待つ

## 順番(上から。着いた所で止まる)

1. **Split at playhead** — 土台の一周の「切る」。**仮コードで測定済み: 新しい Intent が要らない**。
   `LayerTiming{start,duration,source_in,speed}` を2本へ割り、既存の `SetTiming`/`AddLayer`/
   `SetMeta`/`SetTrack`/`SetEffects` を並べるだけ。約30行。`LayerAttrs → LayerAttrsPatch` の
   変換口だけ無いので手で組む
2. **色と縁を触れるようにする** — Solid の `rgba`、Text の fill/stroke。
   入口は **Browser**(利用者裁定)。一覧は「この作品で使われている色」で、
   **固定標準 swatch は作らない**。Inspector 側にも行を出してよい。
   **キーフレームは効かない**(上の測定)。効かせようとするな — 意味の追加になる
3. **ギズモの拡縮/回転ハンドル** — 四隅=Scale、辺=片軸、枠の外=Rotation。
   書き先は既存の property、経路は今日通した transient overlay + `write_key` と同じ。
   **新しい機構を作らない**。Shift=等比/軸拘束、Alt=中心基準まで着けば上出来
4. **`increment()` が宣言の `MIN`/`MAX` を見る** — いま `intensity` を30px引くと `1.5 → 31.5`
   (宣言の範囲 0.0〜4.0 を無視)。今日見つけた綻び
5. **ベジェを Browser の Create から** — `PathSource::Bezier` で層を作る。
   S16(Text/Rectangle が生まれる口)と同じ形
6. **キーマップを表にして、設定で変えられるようにする**(利用者要望 2026-08-30
   「Config変えれるショートカットキーもほしい」)。
   **監督の読み**: キーバインドは**意図に結ぶ**(機構名ではなく「割る」「複製」等の意図名 —
   裁定174「UI動詞は意図を語り機構を語らない」)。表は**データ**にして差し替え可能にする。
   **朝に読みが違っていたら直す。**
   最初に載せるのは `ui-inherited-grammar-gap.md` の Tier 1 で「既存の器で即撃てる」物だけ:
   ←/→ frame step(Shiftで10)・Home/End・Esc(選択解除/gesture cancel)・
   Cmd+D 複製・Cmd+K split(上の1番)。**器が無い物は載せない**
7. **反射の通し検分** — `docs/ui-inherited-grammar-gap.md` の Tier 1 を台本に窓を実操作。
   直すのは配線だけの物に限る
8. **`vello_encoding config.rs:185` の減算オーバーフロー** — 仕切りを引きすぎると落ちる。
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

## 夜間に見つけた欠陥(朝の裁定待ち)

### 【誤報】Create で生まれた層が Stage に描かれない

**欠陥は存在しなかった。監督の観測ミス。**

Create(Rectangle / Bezier)は最初から正しく動いていた。`screencapture` の切り取りを
Stage の左上に固定したまま見ていたため、**中央に描かれる新層が枠の外**だった。
シェイプはキャンバス中央基準(`shape_is_centered_on_the_canvas_not_anchored_to_the_top_left`
というテスト名がそう言っている)。

代償: 診断レーンを3本走らせた。3本とも正しく「データもラスタライザも store も engine も
無罪」と報告し、`resolved_layers` に新層が出て画素が変わることまで実測していたのに、
**私が窓の見え方を疑わなかった**。

**監督の反省(3回)**:
1. 「座標が板の外だから」— テスト名が中央基準だと言っていた
2. 「古いビルドを見ている」— 起動時刻と mtime を突き合わせれば1分で否定できた
3. 「合成に入っていない」— 実際は入っていて、**私の切り取りが狭かった**

3回とも**手元に答えがあるのに先に推測した**。
[[surprise-triggers-rederivation]] の「同種の行動の2回目は補正=止まれ」が2回目で止められなかった。
**窓を見る時は、切り取る前に全体を1枚撮る。**

副産物: `spawn_layer` が `revision` を書いていなかったのは本当の欠陥で、直してある
(`c8c8d975`)。`probe/src/browser.rs` の `mod spawn_diagnosis` は再現器具として残す。
