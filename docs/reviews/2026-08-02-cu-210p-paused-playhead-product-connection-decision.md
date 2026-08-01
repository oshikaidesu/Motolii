# CU-210P paused playhead 製品接続決定

- 日付: 2026-08-02
- 状態: **実装・通常製品E2E DONE**
- CU-210P: **DONE**

## 1. 利用者成果

通常製品windowで native Timeline ruler をclickすると、同じ editor playhead が native playhead、Stage timecode、
Stage評価時刻へ即時投影される。

## 2. 既存契約接続票

| 項目 | 接続 |
|---|---|
| AUTHORITY | M3 U5、U3a-2Q-P2 / P4、M2 D5 |
| INTERNAL TARGET | `ProductApp` Host coordinator、`RationalTime`、`EvaluationTime`、`ProductTimelineProjection`、`TimelinePrepareInput` |
| OWNER | editor playhead は既決どおり `Project session`。native Timeline / React Stage は read-only 投影 |
| WRITE ROUTE | native ruler click → Host private setter。Document / journal / Undo へ書かない |
| GAP | 製品runtimeは評価・描画・timecodeを0へ固定し、単一playhead値を持たない |
| RESOLUTION ROUTE | `REUSE`: 既存pointer monitor、time mapping、render mailbox、Stage snapshotを接続 |
| DISPOSITION | `PASS` |

## 3. 決定

1. fresh Host coordinator の安全な初期位置は composition start とする。現行compositionのstartは既存viewport startと同じ`RationalTime::ZERO`である。
2. paused seek入力は time surface と同じX写像を使う **ruler clickだけ** とする。content clickのselection、bar pressのmove/trimを横取りしない。
3. click値はcomposition範囲へclampし、`ProductApp`のprivate `RationalTime`を一回更新する。
4. 同じ値をnative playhead X、Stage timecode、`RenderRequest.evaluation_time`へ投影する。
5. project reopenでは値を保存・復元せず、fresh Host coordinatorが1の初期位置を再構築する。
6. Rectangle Placeの時刻は既存の保護済み受入が`RationalTime::ZERO`を固定しているため本粒では変更しない。試験期待値を変更せず、契約解消を別粒へ残す。

## 4. 非目標とSTOP

- playback、audio clock、GPU timestamp、DRS、連続drag scrub、step key、Space shortcutは`CU-209`後の`CU-210`へ残す。
- visible range、Project-session永続codec、公開API、Document / journal / Undo、plugin契約を追加しない。
- React stateを正本にしない。content click、clip move/trim、selectionの既存意味を変える必要が出たらSTOPする。
- Rectangle Placeの既存zero時刻oracleを迂回・変更しない。
- seekごとに新thread、blocking channel、同期GPU readbackを作らない。既存latest render mailboxだけを使う。

## 5. 必須oracle

- ruler左右端と中央がstart / end / midpointへ写り、範囲外は入力対象にならない。
- playhead描画Xは同じviewport写像を使い、0固定ではない。
- seek後のrender requestが同じ時刻を読む。
- seekはDocument bytes、journal、Undo depth、selectionを変えない。
- content clickとbar pressは従来どおりselection / interval gestureへ届く。
- Rectangle Placeは既存保護oracleどおりzero時刻を維持する。

## 6. 実装と通常製品E2E

- `ProductApp`のprivate `RationalTime`を単一ownerとし、ruler hit、native playhead、Stage snapshot、既存latest render mailboxへ接続した。
- 自動試験はruler/content分離、midpoint写像、playhead X clamp、Stage timecodeを追加。`cargo test --locked --workspace`、`cargo clippy -p motolii-ui --all-targets -- -D warnings`、`npm --prefix ui/motolii-web run check:host`、`./scripts/check-docs.sh`、`git diff --check`は緑。
- 既存保護試験`cu110_product_place_commit`がPlace zero時刻を固定していることを検出し、期待値を変更せずPlace接続を本粒から除外した。
- 通常製品window `Motolii`でruler中央をclickし、Stage timecode `00:00.0 → 00:05.0`、赤いnative playheadの中央移動、Stage画像の同時更新を確認した。selection、interval gesture、Document編集は行っていない。
- binary SHA-256: `96a0b544cd1debedd91057d8b57d16c52c62d55b7a3772480b951025604a844f`
- seed SHA-256: `dc758e46300ddd19cc68d30ee1bb1fded8b71236a13b982a352dbdfd621a8176`
- E2E screenshot SHA-256: `75d50372729abb24377f68e5cd92dffcc64ec728511e0a9ea5cd4c6bd161ccaa`

本完了はpaused ruler seekだけを閉じる。親U5 / CU-210のplayback、audio clock、latest continuous scrub、停止後idleは未完である。
