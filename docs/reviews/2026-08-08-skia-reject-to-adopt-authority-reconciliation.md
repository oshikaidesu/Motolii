# Skia の `REJECT` → `ADOPT` — authority衝突の裁定

日付: 2026-08-08
状態: **決定 / 旧`REJECT`を撤回**

## 1. 衝突の内容

| 日付 | 文書 | Skiaの処分 |
|---|---|---|
| 2026-07-21 | [native surface renderer 再選定](2026-07-21-native-surface-renderer-reselection.md) §3 | **`REJECT`**。理由「既存wgpu/Velloと**重複するrenderer、cache、alpha、backend lifetime**を持ち込む」 |
| 2026-07-27 | [U3a-2A renderer採択決定](2026-07-27-u3a-2a-renderer-adoption-decision.md)（状態: 決定 / DONE） | 上記を引用し `REJECT for product path` を維持 |
| 2026-08-07 | [RN/rust-skia runtime再基線](2026-08-07-m3-react-native-rust-skia-runtime-rebaseline.md) | **rust-skia を Timeline／Curve Editor／Stage overlay の標準**に |

**2026-08-07の再基線は、この`REJECT`に一言も触れていない**（`REJECT` / `再選定` / `U3a-2A` / `撤回` のいずれもgrep 0件）。
旧WebView、direct-wgpu/Vello、eguiについては明示的に処分しているにもかかわらず、
**Skiaの既決`REJECT`を覆したという記述が無い。**

## 2. 裁定

**2026-08-07の再基線を正とし、2026-07-21 §3 および `U3a-2A` の Skia `REJECT` を撤回する。**

## 3. 日付だけでなく、実質的に成立する

`REJECT`の理由は「**既存wgpu/Velloと重複する**」であった。

2026-08-07の再基線は**同一文書内で**次を決めている。

> 旧標準の…**direct wgpu/VelloをTimeline／Stage UIの既定rendererとする構成**は、
> **新規製品実装の標準から外す**

**`REJECT`の前提が、再基線によって消えている。**
Velloが製品標準から外れた後にSkiaを入れるのは、**重複ではなく置き換え**である。

さらに`concept.md`の規律と整合する。

> **発明工程を持たない**: …identity、Undo、layout、scheduler、codec等は解決済みのOSS、標準、実装patternを採択し、
> **薄いtranslation／admission adapter、製品policy、fixtureだけ**を製品固有codeとして持つ

再基線の採択理由2も同旨である。

> rust-skiaのpath、text、stroke、clip、transformを使うことで
> **primitive rendererやcurve tessellationのスクラッチを減らせる**

**スクラッチの最小化**という一貫した方向であり、`REJECT`時点とは前提条件が変わった結果である。

## 4. `REJECT` のうち、撤回**されない**部分

「重複する renderer / cache / **backend lifetime**」はVello退役により解消するが、
**alpha と色**の懸念は消えない。

`AGENTS.md` 絶対規律2:

> **色変換一元化**: 色変換はrender直前の**一箇所だけ**

`docs/README.md`:

> 散らばった瞬間に**Oliveの二の舞（全書き直し）**（落とし穴F-5）

2026-08-08のcapsule gateも同じ点を検出している。

> Skia→wgpu のpixel契約 / color type、RGBA/BGRA対応、
> **premultiplied/unpremultiplied alpha**、color space、row stride、texture format を実装者が選べる

したがって次を**撤回せず維持する**。

- **Skiaの色・alpha処理を第二の色変換点にしない。** overlayはpremultipliedで一貫させ、
  変換点を増やさない。実装時に pixel 契約を明示固定する
- **Skia型をdomain／公開plugin契約／Document schemaへ漏らさない**
  （2026-07-21 §181 の規律をそのまま維持）
- 用途は**canonical出力外のoverlay描画のみ**。base previewとfinal compositeはwgpuが所有する

## 5. 手続き上の失敗として記録する

本衝突は、**2026-08-07の再基線が既決`REJECT`を引用も撤回もしなかった**ために発生した。

さらに 2026-08-08 の主担当は、依存ゲートで `docs/references.md` に skia の項目が無いことを
**「記録層の欠落」と誤読**し、`ADOPT` として台帳へ登録した。
**実際は`REJECT`されていたため載っていなかった。**

`decision-index`にはrust-skiaの行が3本あるが、いずれも2026-08-07再基線由来であり、
**旧`REJECT`との関係を書いていない**ため、主題からの逆引きでは衝突が見えなかった。

> **既決を覆す変更は、覆す対象を明示的に引用して処分を書く。**
> 新しい決定を足すだけでは、逆引きで衝突が見えない。

## 6. 本裁定に伴う更新

- `decision-index`: Skia `REJECT` 行の状態を**撤回**へ更新し、本文書を正本として指す
- `docs/references.md`: rust-skia 項目に**旧`REJECT`からの撤回経緯**を明記
- 2026-08-08 の [N-OVERLAY依存ゲート](2026-08-08-n-overlay-dependency-gate.md) は
  `KNOWN IMPLEMENTATION SEARCH` が旧`REJECT`を捕捉できていなかった。本文書で補う

## 7. 非目標

- Skia を Timeline／Curve／Stage overlay 以外へ広げること
- base preview / final composite の owner を wgpu から移すこと
- 2026-07-21 §3 のその他の処分（GPUI=PATTERN、Slint/Iced/Qt Quick=REJECT、lyon=PATTERN 等）を変更すること
- `vello` の即時削除（新route出口を同じoracleで確認するまで retire しない）
- 色・alpha の懸念を「撤回済み」と扱うこと
