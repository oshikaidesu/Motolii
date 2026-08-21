# 裁定162: Browser の一覧の正本= Document 所有の素材台帳(旧 asset.rs 移植)

日付: 2026-08-21 / 決定者: supervisor(後任セッション、全面委任の範囲内)/ 種別: Document 構造+Browser 第一波の範囲裁定

## 問い(ζ 調査 EVIDENCE_GAP 1)

next の store には「取り込んだが未配置の素材」を保持する場所が無い(`LayerSource::Media` は配置後の属性)。Browser が見せる一覧の正本は (i) Document へ素材台帳を足す か (ii) 配置済み layer からの派生一覧に意味を縮小する か。

## 裁定

**(i) を採る — 旧 `crates/motolii-doc/src/asset.rs`(740行: `AssetId`/`Asset`/`AssetTable`/`SourceFingerprint`)を next の store へ移植する。**

- bin-first(取り込んでから配置)は AE/Premiere/Resolve に共通する基本ワークフローで、map の B 束(596 Bin・600 Import from Media Browser 等16行)の意味そのもの。(ii) はこの意味を再現できず、「メニューはあるが意味が違う」に落ちる
- 移植であってスクラッチではない(保守最低限)。旧台帳は fingerprint(サイズ+内容 hash)による同一性・重複統合(map 527)・欠落検知(map 1393)の下地を既に持つ
- 台帳は Document 所有(undo/persist に乗る)。`AdmitPaths` は「台帳へ記帳」が正となり、layer 配置は台帳からの参照になる(結線は後続切片 — 第一波では台帳と読み口まで)

## 付随裁定(ζ の他 EVIDENCE_GAP の範囲確定)

- **Browser 第一波は MEDIA 種別のみ・現行単一ウィンドウ内**。裁定143 の multiwindow 要件で第一波を block しない(タスク#16 spike は別枠のまま)
- **B3(view)は HTML mock の構造のみ借用し、色/罫線は tokens 読み替え**(視覚正本が線化トンマナ以前のため)
- 動画サムネ代表フレーム規則は B5 着手時に決める(第一波対象外)

## 影響

第一波の発注: η= 台帳移植(store 単独)∥ θ= pane 骨格 B0(挙動ゼロ)。以後 B1(台帳読み)→B2→B3…の順。
