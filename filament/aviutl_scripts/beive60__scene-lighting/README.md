# aviutl2-scene-lighting

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

AviUtl2 用のシーンライティング アニメーション効果です。
背景レイヤーの環境光を前景オブジェクト（キャラクターイラスト等）のエッジに動的に適応させ、
合成映像のリアリティを向上させます。

## 機能

- **環境色サンプリング & ブレンド** — 背景を低密度サンプリングして代表色を抽出し、乗算・スクリーン・オーバーレイ・加算の 4 種のブレンドモードで前景に合成
- **ライトラップ** — アルファ境界の内側エッジ領域に背景色を加算合成し、背景光の回り込み（回折）をシミュレート
- **疑似法線リムライト** — エッジマスクの勾配から疑似法線を生成し、エッジ領域のみにリムハイライトを付加（全画素 Sobel によるピローシェーディングを回避）

## 必要環境

- AviUtl2（最新版推奨）
- Windows x86_64

## インストール

[Releases](https://github.com/beive60/aviutl2-scene-lighting/releases) から最新の `@scene_lighting.anm2` をダウンロードします。

1. `@scene_lighting.anm2` を AviUtl2 の Script フォルダにコピーする
   - 既定パス: `C:\ProgramData\AviUtl2\Script\`
2. AviUtl2 を再起動する

## 使い方

1. 前景オブジェクト（キャラクター画像など）にアニメーション効果「scene_lighting」を追加する
2. **背景レイヤー** トラックバーで参照先レイヤーを選択する（例: −1 = 1 つ下のレイヤー）
3. 必要に応じて各パラメータを調整する

### パラメータ一覧

| パラメータ | デフォルト | 説明 |
| --- | --- | --- |
| 背景レイヤー | −1 | 参照する背景レイヤーの相対オフセット（−20 〜 −1） |
| ブレンドモード | 1（スクリーン） | 0=乗算 / 1=スクリーン / 2=オーバーレイ / 3=加算 |
| 強度 | 50 | エフェクト全体の強度（%） |
| ブラー半径 | 15 | 平均サンプリングのステップ幅（大 = より広域平均） |
| エッジ幅 | 10 | ライトラップ / リムライトを適用するエッジ幅（px） |
| エッジ閾値 | 16 | エッジ検出のアルファ閾値 |
| ライトラップ | ON | 背景光の回り込み効果の有効 / 無効 |
| リムライト | ON | 疑似法線によるリムハイライトの有効 / 無効 |
| リムライト角度 | 45 | リムライト方向の角度（−180 〜 180 度、45 = 右上） |
| ベースカラー | 白（0xffffff） | 環境色に乗算するティントカラー |
| サンプリング | 0（平均） | 0=平均サンプリング / 1=中央点サンプリング |

## プロジェクト構成

```text
aviutl2-scene-lighting/
├── src/
│   └── @scene_lighting.lua    # 開発用ソース
├── docs/
│   └── architecture.md        # アーキテクチャ設計書
├── aviutl2.toml               # au2 CLI 設定
├── .stylua.toml               # StyLua 設定
├── .luacheckrc                # Luacheck 設定
└── README.md
```

## 開発

[aviutl2-cli](https://github.com/sevenc-nanashi/aviutl2-cli)（`au2` コマンド）を使用します。

開発時の正本は `src/@scene_lighting.lua` です。
`au2 develop` / `au2 release` 実行時に `Script/@scene_lighting.anm2` として配置されます。

### 初回セットアップ

```powershell
au2 prepare
```

### 開発用 AviUtl2 の起動

```powershell
au2 develop
```

### Lua formatter / linter

- Formatter: StyLua
- Linter: Luacheck

インストール例:

```powershell
cargo binstall stylua
luarocks install luacheck
```

実行例:

```powershell
stylua src
luacheck src
```

VS Code ではタスク `Lua: Format` / `Lua: Lint` をそのまま実行できます。

## クレジット

- [aviutl2-cli](https://github.com/sevenc-nanashi/aviutl2-cli) — AviUtl2 スクリプト開発用 CLI

## ライセンス

[MIT](LICENSE) © 2026 べいぶ
