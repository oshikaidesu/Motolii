# 出力解像度はCompositionが所有し、素材はfitで受ける

日付: 2026-08-17
状態: **決定**(段階実装。粒1が先、粒2は後続)

## 問題

現行exportは最初のvideo assetのnative解像度を出力フレームにする
(`crates/motolii-export/src/lib.rs` `resolve_export_frame_desc` — M1の単一source前提の名残)。
その結果、compositionのcamera aspect(既定16/9)と合わない素材——スマホ縦動画9:16が典型——は
`frame WxH does not match camera aspect`で**書き出し自体が拒否**される。E2E
`headless_mv_e2e`で実測。P3ペルソナ(スマホ世代)の最初のファイルで死ぬ「普通」違反。

## 先例(既知実装)

| ソフト | 出力解像度の所有者 | aspect不一致素材の既定 |
|---|---|---|
| Premiere | Sequence settings(プロジェクトに保存) | native配置。収まらなければはみ出し/余白(=fit相当の運用) |
| Resolve | Timeline resolution + per-clip「Fit/Crop/Fill/Stretch」 | **Fit**(letterbox) |
| CapCut | 比率設定 | **Fit + 背景ぼかし(blur-fill)がワンタップ**。スマホ世代の既定手癖 |
| AviUtl | プロジェクト設定で明示 | native配置(中央) |

全員一致している点: **素材の解像度が出力を決めることは無い**。出力はproject/composition側の設定。

## 決定

1. **Compositionが出力解像度を持つ**: `Composition`へ`resolution: Option<(u32, u32)>`(serde default `None`)。
   `None`は現行挙動(最初のvideo sourceから導出)のままにして旧Documentと旧readerの互換を守る——
   ロケータと同じ「空なら書き出さない」型の追加であり、版は上げない。
   書き込みは新Command(`SetCompositionResolution`相当、old/new対称)で行い、直書きしない。
2. **`new`(新規プロジェクト)の既定は1920x1080**。既定camera aspect 16/9と整合する現代の既定値。
3. **素材はfit(contain)でcanonical空間へ写す**: clipのsource矩形は素材のaspectを保ち、
   compositionに**収まる最大サイズ**で中央配置(letterbox/pillarbox。余白は透明=合成背景色)。
   これはclip transformの意味を変えず、source→canonical写像の既定を定めるだけ。
   Resolveの「Fit」既定と同じ。
4. **exportは`resolve_export_frame_desc`をcomposition解像度優先へ**: `Some`ならそれを使い、
   camera検証は構成上常に通る。`None`は現行導出(互換)。
5. **粒2(後続)**: blur-fill(CapCut型の背景ぼかし)はfitの上に置く表現でありv1.x。
   選択肢の席だけ本決定で予約し、実装はエフェクト基盤の粒に載せる。

## 実装順(粒1)

schema(field+validate)→ Command family(variant/apply/inverse/meta/replay、SetSoundtrackと同型)
→ `new_document`の既定値 → export分岐 → fit写像(render側のsource矩形決定) → E2Eへ縦動画fixture追加
(9:16素材をimport→place→export→出力が1920x1080でletterboxされることをピクセル審判)。

## 非目標

- per-clipのFit/Crop/Fill/Stretch切替UI(席は将来。既定はFitのみ)
- blur-fill実装(粒2)
- previewとexportで異なるfit(同一評価の不変量を破らない)
- 既存Document(`resolution: None`)の挙動変更
