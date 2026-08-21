# Browser 束の縫い目調査(ζ)— RESEARCH_RETURN 保全(要点)

日付: 2026-08-21 / 発注: 後任セッション(利用者の Browser 質問を受けて優先度繰上げ)/ レーン: read-only 調査(sonnet)
関連裁定: 143(パネル4種・multiwindow 要件)。**本調査の EVIDENCE_GAP 1 への裁定= 2026-08-21-browser-asset-ledger-decision.md(裁定162)**

## map 台帳

browser/bin/import 系の採用予定行は**計50行**。freq≥2 は2行のみ: id980 Project/Media Pool panel(freq3)・id982 Effects & Presets panel(freq2)。束分類: A パネル骨格10 / B 取り込み16(596 Bin・600 Import from Media Browser 等)/ C 素材編集・置換・プロキシ9 / D タグ・整理4 / E 欠落検知・重複統合2 / F 外部ライブラリ2 / G 周辺5。

## 旧世界資産台帳(要点)

- `browser_panel/`(egui、1,741行): 視覚は手本にしない(旧 iced 版 doc が明記)。意味関数のみ参照可
- `browser_blitz/`(1,005行): HTML 検証**器具**であって Browser 実装ではない(「Blitz≠ブラウザ」柵)。移植候補から除外。例外= thumbnail.rs の「表示寸法からの逆算」という手法(コードでなく考え方)
- **正しい移植元**: `motolii-shell-iced/src/browser.rs`(499行 read-model: rail 3席・dedupe・Document を1バイトも書かない)+`browser_pane.rs`(1,410行 view: 視覚正本= browser-library.html。Q0 適用実例集)+ RN `Browser.tsx`(544行、MEDIA/EFFECTS/CREATE の意味の正本)+`drive_browser.rs`(14 test、oracle の型)
- thumbnail: 静止画のみ実装済み(248×168 ≈166KB — iced 2MiB 同期予算内)。**動画サムネは旧世界でも未実装**(FINDING 1)。`motolii-media::read_frame_at` が next に未配線のまま実在
- 旧 `motolii-doc/src/asset.rs`(740行): `AssetId`/`Asset`/`AssetTable`/`SourceFingerprint` — **Document 所有の素材台帳**。next の store に対応物が無い

## 設計案(要点)

- `motolii-browser-pane`(layer 1、pane split 流儀): iced+core+store+tokens+shell-state+**media**(stage-pane の engine 依存と同型の単独例外)
- shell 組み込み: 現行 `Shell::view` は単一ウィンドウ固定 column/row。裁定143 の multiwindow 要件は iced daemon API への切替検証(タスク#16 spike)が未着手

## 切片割り

B0 骨格(挙動ゼロ・PNG 一致)→ B1 素材列挙 → B2 rail/filter → B3 view(視覚、トンマナ読み替え要)→ B4 静止画サムネ ∥ B5 動画サムネ(代表フレーム未決)→ B6 ドラッグ配置。**全部 MEDIA 種別のみ**(EFFECTS/CREATE/パレットは意味起草タスク#14 が空席)。

## EVIDENCE_GAP(→裁定162 で 1 を解決)

1. 未配置素材の台帳が store に無い →**裁定162: 旧 asset.rs を移植**(派生一覧への意味縮小は却下)
2. 「クリエイト」「パレット」の意味は旧世界のどこにも無い(RN は3タブ)— タスク#14 未着手
3. multiwindow spike 未着手(現行は単一ウィンドウ)— Browser 第一波は現行レイアウト内で进め、spike は別枠
4. 視覚正本 browser-library.html は線化トンマナ以前 — B3 は構造のみ借用し色/罫線は tokens 読み替え
5. 動画サムネ代表フレーム規則未決 — B5 着手時に決定

## FINDING

1. 動画サムネは旧世界でも一度も実装されていない(実質新規)
2. browser_blitz は器具であり移植対象外
3. Q0 適用実例集(history‹›/tag editor/Collections/右クリック菜单を機能欠如で落とした判断列)は今回もそのまま踏襲可能
4. 裁定143 は next 世界の裁定で、要件骨格のみ確定・実装/意味起草とも空席
