# 仮コード — 区間の継ぎ目9件と Stage×M5 迎え入れ判定

日付: 2026-08-08
状態: **観察 / 修理許可でも実装許可でもない**

## 0. 扱い

`AGENTS.md`「findingは権限ではない」に従う。本文書は**報告と分類だけ**。
本文書を根拠に発注・実装・決定改訂・休止契約の解除を行わない。

[仮コード器具境界決定](2026-08-07-provisional-call-site-sketch-instrument-decision.md)に従い、
仮コードは非compile・非authorityの器具である。

## 1. 継ぎ目の検査（新規の検査層）

7区間を別々のagentが並列起草したため、**区間の内側は検査済みだが継ぎ目は誰も見ていなかった**。
`Use → Tune → Compose → Inspect → Fork → Author → Publish → Reuse` を1本へ統合し、
区間Aの出力が区間Bの入力として成立するかを検査した。

前段の[区間内側の合成失敗14件](2026-08-07-call-site-sketch-composition-failures.md)とは別層である。

### 検出した9件

| # | 継ぎ目 | 種別 | 内容 |
|---|---|---|---|
| 1 | Use-A ↔ Use-B(media) | 責任の重複＋lifetime | **Document書き込み主体が二重**。Use-A/Cは`queue`→`DocumentEditRuntime::process_next`経由だが、Use-Bのmedia挿入は`DocumentWriter::begin_gesture`/`apply_command`を直接呼ぶ |
| 2 | Use-B(New/Save) → Use-C(export) | 順序の断絶 | `prepare_project_export(&project_path)`は保存済みpathを要求するが、New/Save Asが未決のため鎖内でpathを産出する呼び出しが無い。**既存fileをOpenした場合にしか書き出しへ到達できない** |
| 3 | Use-A ↔ Use-B(再生)・Tune | identityの断絶 | **playheadの正本が2つ**。Use-A(RN)は`host.current_time`、再生とTuneは`product_runtime.rs`の`editor_playhead.current` |
| 4 | Use-A → Tune | identityの断絶 | 選択出力`published.primary`はRN route産だが、実装済みparameter編集はWebView island(再基線で凍結)のみ。**選択identityを編集入口へ運ぶ経路が両区間のどちらにも無い** |
| 5 | Use-A ↔ Tune | 責任の重複 | **Position key値の書き込み入口が2箇所・2runtime**。stage gizmo release(`document_edit_runtime.rs:108`)と Inspector gesture(`product_runtime.rs:3416`) |
| 6 | Compose → Inspect | 型の断絶 | Compose終端`???_KitComposite`の型が存在せず、Inspect入口`gesture_identity(document, LayerId, EffectId)`はlayer/effect粒度のみ。**Kit複合を渡す型が両側とも無い** |
| 7 | Use-B → Fork | lifetime/順序の断絶 | ACG-O2のatomic adoptionは開始revision照合を要求するが、A4Sの採用はrebuild/restartを伴い、restart後の再openは`DocumentWriter::new`が常にrevision=0。**照合すべきrevisionがrestartを越えて存在しない** |
| 8 | Fork ↔ Author | 責任の重複 | 作者経路が2区間に別々に存在。Fork(A4S)はRust crate fork、AuthorはTypeScript。**どちらが鎖のAuthor段かを繋ぐ決定が双方の元テキストに無い** |
| 9 | Author → Publish | 型の断絶 | `publish_vism`の入力は`&PluginContract`だが、**実行時にそれを産出する作者経路が型ごと無い** |

加えて、capabilityを表す型の不在が Author と Publish の**両区間に現れる**（同根の可能性）。

### 主担当が作った負債の顕在化

**#3 は2026-08-07の[RN transient時刻席決定](2026-08-07-r2-rn-transient-time-seat-decision.md)が
「明示的に受け入れる負債」として記録したもの**である。継ぎ目で実際に衝突することが確認された。
返済期限は同決定どおり`R2-FOCUS-PLAYHEAD-AUTHORITY`の閉鎖時とする。**本文書で前倒ししない。**

**#1 は絶対規律4（single writer）に触れる可能性がある。** ただし
`DocumentWriter::apply_command`が単一writer thread内の正規経路である可能性を排除していないため、
**規律違反と断定しない。** M2 Document ownerへの確認事項とする。

## 2. Stage×M5 迎え入れ判定

> 問い: M5（3D・点群・spatial renderer）をM3のStageへ「今」迎え入れられるか。

M5休止契約は開放条件を「IDやtest件数でなく**意味境界**」と定めている。
仮コードは非compileの器具であり実装ではないため、**判定手段として用いた。**

### 絶対規律の成立確認

| 規律 | 判定 | 根拠 |
|---|---|---|
| **2 色変換一元化** | **成立** | 鎖上の変換は`YuvToRgba::convert`(`motolii-gpu/src/yuv.rs:179`)の一箇所のみ。render内はSrgb+premul固定、Stage presentは無変換blit(`rn_product_host.rs:808/859`)。ただしM5のlinear-light意味を載せるには合流点が必要で、provider内変換で代用すれば規律2違反になる |
| **6 Preview/Export同一評価** | **成立** | Preview(`render_worker.rs:524/532`)とExport(`motolii-export/src/lib.rs:267/298`)が同一の`build_document_frame_graph`＋`render_graph_cached`を呼び、差は`Quality`のみ。受入試験`d3e_preview_export_same.rs`、`d3f_preview_export_camera.rs`が実在 |

**落とし穴F-5（色変換が散る＝Oliveの死因）はStage×M5の鎖上で発生しない。**

### 判定: `NO`（ただし席は実在する）

Provider席は**設計だけでなく配線として実在**する。点群Providerは`LayerSourcePlugin`として
`RenderStep::Plugin`経由で`render_graph_cached`へ入り、premultiplied RGBAを返してComposite合流し、
Preview/Export両経路と`DisplaySlot`→Stage presentを**新しい経路なしで**通る。
これは`concept.md`の点群の扱いと一致する。

閉じないのは**席に置くものの側**で3点:

1. Providerへ渡す評価済み観測が`CompCamera`(Planar)しか実在せず、**その昇格はC0決定が禁止している**
2. 点群payloadを解決するimporter / GPU resource ownerが無い
3. M5決定のlinear-light合流点の席が無い

**3つとも休止契約が「M3意味開放まで追加禁止」とした当のものである。**
すなわち`NO`の理由は「未着手だから」ではなく「**自ら禁止している対象だから**」であり、
休止契約が意図どおり機能していることの確認になる。

### 不足している一つ

> **C0-Schema（projective observationの公開型）**

M5-C0の意味論は決定済みで、private `glam` fixtureが5/5通過している。
未成立なのは**provider wire / Document schema**、すなわち公開型だけである。

休止契約の再開順序でもM3開放後の**最初のM5固有gate**にあたる。
これ無しではProviderはPlanar偽装でしか動けず、「spatial」の意味自体が鎖へ載らない。

`RESOLVED: 27 / UNKNOWN: 4`

## 3. 確定した順序

本判定により、次の順序が鎖の上の事実として裏づけられた。

```
M3の背骨を閉じる → 休止契約が意味境界で開く → C0-Schema（翻訳層）→ 点群/spatial Provider
```

`concept.md`は「Hostの役割: **知覚表現を翻訳する**」と定め、
「**薄いtranslation／admission adapter、製品policy、fixtureだけ**を製品固有codeとして持つ」としている。
C0-Schemaはこの翻訳層に該当し、engine輸入でもscene graph移植でもない。

## 4. 非目標

- 本文書を根拠に休止契約を解除すること
- 継ぎ目9件をM3の接続粒として修理すること
- #3（playhead二重）の返済を`R2-FOCUS-PLAYHEAD-AUTHORITY`より前倒しすること
- #1を規律違反と断定すること（M2 ownerの確認前）
- C0-Schemaを本文書で起草・提案すること
