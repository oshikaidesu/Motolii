# Blitz HTML/CSS の設計・検証方針

日付: 2026-08-16
状態: **決定**

## 決定

Browser、Inspector、chrome の可視面は、**通常の意味的な HTML/CSS として設計する**。`flex` だけに縮めない。繰り返すサムネイルやパラメータ格子には CSS Grid、一次元の帯や行には Flex を、構造に合う方を使う。

この方針は「ブラウザで描いた画面を製品へ持ち込む」決定ではない。二つの renderer に同じ HTML/CSS を通す、次の分業である。

```text
browser preview       = 視覚設計を利用者と確定する場
Blitz (固定crate版)   = 製品への採用可否をレンダーで確定する場
Rust / egui           = 入力、Document/D2、shortcut、dock、Stage を所有する場
```

ブラウザでの見え方だけでは製品採用にしない。同時に、過去の `spikes/blitz-probe/` に一度出た CSS property だけを許す制約も撤回する。

## 根拠

Blitz の公式 CSS status は `display: block / flex / grid`、`gap`、`aspect-ratio`、`border-radius`、`overflow: scroll` などを対応済みとしている。Blitz 自身も modern HTML layout に Grid/Flex/Block/absolute を含めることを目標に掲げる。

- [Blitz CSS status](https://blitz.is/status/css)
- [Blitz GitHub README](https://github.com/DioxusLabs/blitz#goals)

Motolii が固定する `blitz-dom = 0.3.0-beta.1` と upstream の current status は同一性を保証しない。したがって公式statusは**設計可能性の根拠**、固定crate版の `motolii-blitz-dump` は**製品採用のoracle**である。

## authoring loop

1. 既存RN source asset の情報階層、label、色・寸法の出所を読む。Browser は `Browser.tsx` / `productStyles.ts`、Inspector は `Inspector.tsx` / `productStyles.ts` が出所である。
2. 普通の browser preview で HTML/CSS を利用者と確認する。これは視覚設計だけであり、Document、host、input、drag、Undo、保存の owner を作らない。
3. 同じ構造を Blitz の `HtmlDocument` に通す。新しい property は既存probeへの文字列照合で拒否せず、固定crate版で最小の direct dump または対象panel dumpを取り、正常・狭いpane・代表的な実画像の三条件を確認する。
4. browser と Blitz の差は、property、crate version、viewport、実画像の有無を添えて記録する。ブラウザだけに合わせて「Blitzでは使えない」と設計を縮めず、まず固定版で再現する。

CSSの採用後は、既存の `MOTOLII_BLITZ_CSS_DIR` 経路で再ビルドなしにCSSを差し替えてdumpできる。これは設計反復の高速化であり、browser preview の代替oracleではない。

## 現在の注意点

公式status上でも `position: fixed` / `sticky`、`overflow: auto`、`text-overflow`、`line-clamp` などは未対応または部分対応である。これらを使う時は同じ固定版direct dumpを先に作る。JS engine は無いので、script、inline event handler、JS依存ライブラリは製品HTMLへ持ち込まない。

Timeline の clip/key のような高密度面は、この一般則の例外ではない。CSSは周囲の行・header・labelを担い、面そのものは既決の custom widget 1ノードで描く。Dock は `egui_tiles`、Stage はRerunのegui widgetのままである。

## 不変の境界

- HTML/CSSは表示だけ。input、shortcut、drag、Document/D2、Undo、host dispatch はRust/egui側に残す。
- Browser preview は設計artifactであり、製品window、Blitz dump、入力接続の代替ではない。
- 色・寸法・構造を移植する時の出所は既存RN source assetのfile:lineであり、新しいtokenやDocument意味をHTML/CSSから作らない。
- 実画像のthumbnail owner は既存 `media_library` と `thumbnail.rs` のまま。CSS Gridの採用は画像管理の再実装を意味しない。

## 受入

Browser と Inspector の表示移植を「HTML/CSSで確認済み」と報告するには、次を分けて示す。

1. browser preview で利用者が視覚設計を確認したこと。
2. 固定Blitz crate版の同じ構造・CSSが `motolii-blitz-dump` で描けたこと。
3. 値の出所と、Blitz固有の差・未対応propertyを明示したこと。

入力やDocumentへの配線はこの受入に含めない。
