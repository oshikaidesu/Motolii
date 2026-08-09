# 素材不足時の扱い — 既定はFreeze、変更は利用者へ問う

日付: 2026-08-08
状態: **決定 / 実装未着手 / 仕様粒は未閉鎖**

## 1. 決めること

素材差し替え等でclipの尺に対し**ソース素材が不足**した場合の扱い。

| | 決定 |
|---|---|
| **既定値** | **`OverrunMode::Freeze` を維持する。変更しない** |
| **不足を検出したら** | 黙って処理せず、**利用者へ問う** |
| **提示する選択肢** | ① そのまま（Freeze） ② 引き伸ばす（`TimeMap` speed変更） ③ ループ（`OverrunMode::Loop`） |
| **`Loop`／`Black`** | **利用者が明示選択した時だけ**有効。既定にしない |

## 2. 現行の実コード事実

```rust
// crates/motolii-core/src/time_map.rs
pub enum OverrunMode {
    #[default] Freeze,   // 近い側の端フレームへクランプ
    Black,               // 非描画
    Loop,                // available 範囲で wrap
}

pub fn require_freeze_overrun(&self) -> Result<(), TimeMapError> {
    match self.overrun_mode {
        OverrunMode::Freeze => Ok(()),
        mode => Err(TimeMapError::UnsupportedOverrunMode(mode)),
    }
}
```

`Loop` / `Black` は **schemaへ予約済みだが v1 実装は typed拒否**である。
`graph.rs::build_clip` が `require_freeze_overrun()` を呼び、
「active窓の外でも Black/Loop を黙って通さない」を実行している。

`TimeMap` は `speed_num` / `speed_den` を持ち、**定速変更（引き伸ばし）はv1で表現可能**である。

## 3. 既定値を動かさない理由

### 3.1 互換性

既定値変更は**既存fieldの再解釈**にあたる。`concept.md`:

> **改善可能性を互換性破壊の免罪符にしない**: 後から直せることは強みだが、
> 公開Documentへ一度焼いた意味は利用者の制作資産になる。
> **既存fieldの再解釈ではなく、migrationと意味論goldenを伴う**

`overrun_mode` を省略保存しているprojectは、既定値を変えると**開き直しただけで挙動が変わる**。

### 3.2 気づける失敗を既定にする

2026-08-08の利用者シミュレーション（別family 2レーン独立）は、
**「気づかないまま進み、書き出し後に発覚する」失敗を最も重い**と判定した。

| 既定 | 素材が尽きた時の見え方 | 気づけるか |
|---|---|---|
| **Freeze** | 静止する | **異常として即座に見える** |
| Loop | 内容が繰り返される | **意図的に見えてしまう。気づきにくい** |

3秒素材を10秒span へ置いた場合、Loopは3回繰り返し＋1秒となり、それらしく見える。
**Loopを既定にすると、素材不足という事実を見えなくする方向に働く。**

## 4. 「問う」ことの位置づけ

`concept.md` の既決の適用であり、新しい原理ではない。

> **意味は厳格に、表現は自由にする** … 拒否するのは、因果を追えず局所回復できない仕組みである。
> **失敗はCommit前に型付きで説明し、操作はCancel/Undo可能**にする

同時に次の緊張を認識する。

> **複雑さをユーザーへ転嫁しない**

毎回問えば「隠さない」が達成される一方、問いすぎれば転嫁になる。
**この頻度・条件は本決定では閉じない**（§6）。

## 5. 先例の状況（記録）

素材差し替え時の属性不一致について、**業界は解いていない**。

| 製品 | 不一致の扱い |
|---|---|
| After Effects | alpha解釈の"Guess"が不確実な時に**ビープ音**のみ。尺／解像度／fpsの警告は公式資料で確認できず |
| Premiere Pro | `Show Clip Mismatch Warning` は存在するが、**空タイムラインへのdrop時**の挙動として説明。Replace時のtriggerは未確認 |
| Final Cut Pro | **fps不一致ファイルはそもそもRelink候補にしない**（回避であって解決ではない） |
| Foundry Nuke | フォーマット情報を新ファイルのmetadataから**サイレントに更新** |
| DaVinci Resolve / Blender VSE | 不明（一次資料取得できず） |

`WITH_MISMATCH_WARNING: 3`（いずれも部分的）。

したがって本決定は**先例の収束点からの採択ではなく**、
`concept.md`「隠さない」「失敗はCommit前に型付きで説明」からの導出である。

## 6. 本決定が閉じないこと

- **問う頻度・条件**（毎回か、不一致が閾値を超えた時か、既定選択を記憶するか）
- **UI形式**（dialog / inline diagnostic / 遅延通知）
- **`Loop` / `Black` 実装の解禁時期**と `require_freeze_overrun` の緩和方法
- **尺以外の不一致**（fps／解像度／縦横比／alpha）の検出と提示。
  解像度は絶対規律5（正準座標、絶対pxを永続意味にしない）により構造的に影響が小さいが、
  縦横比変化は正準が高さ基準のため横範囲が変わる。**未検証**
- **音ズレ検出**。`Transport` に seek が無く mixed audio 接続は `GAP-28` のため**検出手段が現状ない**

## 7. 非目標

- `OverrunMode` の既定値を変更すること
- `Loop` / `Black` を既定または暗黙適用にすること
- 本決定を根拠に実装を発注すること（仕様粒が未閉鎖）
- 属性不一致全般（fps／解像度／alpha）の扱いを本決定へ含めること
- 先例に無いことを理由に独自機構を新設すること（[既知実装採択モデル](../known-implementation-adoption-model.md)へ従う）
