# 入力マップ/ショートカット スキーマ設計(2026-07-15)

ステータス: **【設計草案】**(M3E-9 / GAP-6)。実装はM3期間。本書がキーマップデータモデル・永続化方針・a11y境界の正本である。

出典: [M3ガード5](../specs/M3-ui-integration.md#実装ガード先行ツールの失敗ユーザー不満クロスチェック-2026-07-11)、[M3ガード11](../specs/M3-ui-integration.md#実装ガード先行ツールの失敗ユーザー不満クロスチェック-2026-07-11)、[GAP-6](../backlog.md)、[先例調査 C章](2026-07-12-prior-art-gap-survey.md#c-キーマップ入力gap-6設計への直接材料)、[反対側レビュー C章](2026-07-12-prior-art-gap-counter-review.md#c-キーマップ入力)。

## 1. 目的と非目標

### 目的

- **準恒久ユーザーデータ**としてキーマップを設計する(台帳③恒久物)。後付けだとウィジェット毎のハードコードが先に増える([M3-M4ゲート台帳](2026-07-12-M3-M4-gate-ledger.md))。
- **初日から全ショートカットをカスタマイズ可能**にする(M3ガード5 / AviUtl層の獲得条件)。
- **Document(プロジェクトJSON)とは別系統**で永続化する。物理キー・修飾・gesture割当をプロジェクトへ焼かない([Relative Move決定](2026-07-15-relative-scope-duplicator-decision.md)と同型)。

### 非目標(本書のスコープ外)

- `Document` / serde スキーマへのキーマップフィールド追加 — **禁止**。別タスクでも採らない。
- 製品UI(キーマップ編集画面)の実装 — M3 UIタスク(U2以降)で別PR。
- AE/AM/FCPX等の**互換プリセット出荷** — v1ではMotolii標準のみ([反対側レビュー C-2](2026-07-12-prior-art-gap-counter-review.md#c-2-他アプリ互換プリセット--縮小))。
- 一般化された設定healing基盤 — 破損版を実際に出した時点まで作らない([反対側レビュー C-3](2026-07-12-prior-art-gap-counter-review.md#c-3-設定マイグレーションhealing--縮小))。

## 2. UX方針(既定操作)

### 2.1 既定は業界標準、革新はオプトイン

FCPXのマグネティックタイムライン**強制**は3,700筆超の抗議署名とプロ層の恒久流出を生んだ(M3ガード5)。Motoliiの**既定**は次を採る:

| 領域 | 既定挙動 | 備考 |
|---|---|---|
| タイムライン構造 | **トラック型**(クリップはトラック上に自由配置) | マグネティック/リップル優先モードは**オプトイン**機能として後から足す。既定ONにしない |
| 再生/一時停止 | **Space** | テキスト入力フォーカス中は無効(IMEガード3と整合) |
| スナップ | **キートグル**でON/OFF(既定ONかOFFかは**未決** — 下記§7) | ビート/フレーム/マーカーへの吸着はU7と連動 |
| 編集モデル | 選択→ドラッグ移動/トリム、Undo/Redo | OpenCut/Flow/一般的track型NLEに近い動線 |

「概念として優れていても、訓練されてきた全てに反する」操作は**製品のデフォルトにしない**。

### 2.2 初日カスタマイズ

- 出荷時のbuiltin presetは**出発点**であり、ユーザーは**初回起動後すぐ**任意の`CommandId`へ別キーを割り当てられる(M3ガード5)。
- ハードコードされた`if key == ...`分岐を製品コードに増やさない。解決は常に**キーマップレイヤ**経由。
- Relative Move等のmodifier+dragも、物理キーではなく`CommandId`へ割当([Relative Move決定](2026-07-15-relative-scope-duplicator-decision.md)§1)。

## 3. データの所在(プロジェクトと分離)

```
┌─────────────────────────────────────┐
│ Document (プロジェクト JSON)         │  ← 編集内容・ジャーナル。共有/バージョン管理対象
│ キーマップフィールドなし             │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│ ユーザーデータ (ホスト設定)          │  ← マシン/ユーザー単位。プロジェクトと独立
│ ・keymap overlay (本書)              │
│ ・theme / locale 等 (別スキーマ)     │
└─────────────────────────────────────┘

┌─────────────────────────────────────┐
│ builtin preset (アプリ同梱・不変)    │  ← バージョン管理はアプリリリースに追随
│ ・motolii-standard.json 等           │
└─────────────────────────────────────┘
```

- **分離理由**: プロジェクトを別マシンへ移しても、各自の筋肉記憶(キー配置)は持ち込まない。逆に、同じユーザーが複数プロジェクトで一貫したキー配置を使える。
- Documentマイグレーション規律(M2実装ガード8)と**別系統**のformat version・migrationを持つ。

## 4. スキーマ草案

### 4.1 識別子

| 概念 | 形式 | 例 |
|---|---|---|
| **CommandId** | 安定ドット区切り文字列。semver的にリネームしない | `motolii.transport.play_pause`, `motolii.timeline.snap_toggle`, `motolii.edit.undo` |
| **PresetId** | builtinは`motolii.*`、ユーザー作成は`user.*` | `motolii.standard`, `user.my-layout` |
| **ContextId** | 同一物理キーの文脈分岐(未決: 初期セットは§7) | `global`, `timeline`, `text_input` |

プラグインが新コマンドを登録する場合も、ホストが発行する`CommandId`名前空間に載せる。**プラグインがキー割当をDocumentへ書く経路は無い**。

### 4.2 Gesture(1つの入力意図)

1つの`CommandId`に対し、ユーザーは0〜N個の**Gesture**を割り当て可能(複数ショートカットで同一コマンド)。

```json
{
  "gesture_id": "g1",
  "event": "press",
  "chord": {
    "key": "Space",
    "modifiers": []
  },
  "pointer": null
}
```

| フィールド | 必須 | 説明 |
|---|---|---|
| `gesture_id` | overlay内で一意 | ユーザー編集・削除の単位 |
| `event` | ○ | `press` / `release` / `click` / `drag` — [反対側レビュー C-1](2026-07-12-prior-art-gap-counter-review.md#c-1-pressclickdragの区別--採用根拠を縮小)採用。click選択とdrag移動の同居に必須 |
| `chord` | キーボード系 | `key`(論理キー名) + `modifiers[]`(`Shift`,`Ctrl`,`Alt`,`Meta`の組) |
| `pointer` | ポインタ系 | `button`(`Left`/`Middle`/`Right`)、`modifiers[]`。`event=drag`時はドラッグ開始ボタン |
| `context` | 任意(省略=`global`) | `ContextId`。`text_input`中はテキスト系を優先しショートカットを抑止 |

**ホスト入力ポリシー(ユーザーデータに入れない)**: click判定の時間閾値・drag開始距離(px)はホスト実装の定数。キーマップファイルには書かない([反対側レビュー C-1](2026-07-12-prior-art-gap-counter-review.md#c-1-pressclickdragの区別--採用根拠を縮小))。

### 4.3 プラットフォーム差

| 論理修飾 | Windows/Linux | macOS |
|---|---|---|
| `Ctrl` | `Control` | `Control` |
| `Alt` | `Alt` | `Option` |
| `Meta` | `Win`/`Super` | `Command` |

- ファイルには**論理名**(`Ctrl`等)を保存し、実行時にOSへマップする。
- 論理`key`は[winit `KeyCode`](https://docs.rs/winit/latest/winit/keyboard/enum.KeyCode.html)に準ずる名前を正本とする(実装時にenum化)。**未決**: テンキー・国際配列の表記ゆれの正規化規則(§7)。

### 4.4 不変ベース + ユーザーオーバーレイ

[反対側レビュー C-4](2026-07-12-prior-art-gap-counter-review.md#c-4-不変ベースユーザーデルタ--採用)採用:

1. **builtin preset** — アプリ同梱JSON。リリース毎にversionを上げる。**ユーザー編集で上書きしない**。
2. **user overlay** — ユーザー設定ディレクトリの`keymap-overlay.json`。内容は**デルタのみ**:

```json
{
  "format_version": 1,
  "active_preset": "motolii.standard",
  "bindings": {
    "motolii.transport.play_pause": {
      "replace": [
        { "gesture_id": "u1", "event": "press", "chord": { "key": "KeyK", "modifiers": [] } }
      ]
    },
    "motolii.timeline.ripple_delete": {
      "disable": true
    }
  }
}
```

| オーバーレイ操作 | 意味 |
|---|---|
| `replace` | 当該`CommandId`のbuiltin割当を**丸ごと置換**(gesture配列) |
| `add` | builtinに**追加** |
| `disable` | 当該コマンドの全gestureを無効化 |
| (キー削除) | overlayからエントリ削除 → builtinへ**復帰** |

プリセット切替時、別presetへのuser deltaを**暗黙適用しない**。presetごとにoverlayを分けるか、単一overlay+`active_preset`のみ — **未決**(§7)。

### 4.5 衝突検出

保存時およびキーマップ編集UI確定時に検証する:

| 衝突種別 | 検出 | 既定ポリシー |
|---|---|---|
| 同一`(context, event, chord/pointer)`に複数`CommandId` | **エラー** | 保存拒否。ユーザーが明示的に解決するまで |
| 同一`CommandId`に重複`gesture_id` | **エラー** | 同上 |
| builtinとoverlayの関係 | 警告なし | overlayは意図的置換のため衝突対象外 |
| 未知`CommandId`への割当 | **警告**(保存は許可) | アプリ更新でコマンドが消えた場合のユーザー設定温存([反対側レビュー C-3](2026-07-12-prior-art-gap-counter-review.md#c-3-設定マイグレーションhealing--縮小)) |
| テキスト入力中のグローバルショートカット | **ランタイム抑止** | IME変換中Enter食い(M3ガード1③)と整合 |

**未決**: 衝突時に「後勝ち」「優先度フィールド」を許すか — v1は**拒否のみ**で足りるか実装時に再確認(§7)。

### 4.6 永続化ファイル(場所)

| 種別 | 所在(方針) | ファイル名(案) |
|---|---|---|
| user overlay | OS標準の**アプリ設定ディレクトリ** | `keymap-overlay.json` |
| builtin preset | アプリバンドル/リソース | `presets/motolii.standard.json` |
| インポート/エクスポート | ユーザー指定パス | **未決**(§7) |

**設定ディレクトリの具体パスは未決**。実装時に次を正本とする想定だが、本書では確定しない:

- Linux: `$XDG_CONFIG_HOME/motolii/`(未設定時`~/.config/motolii/`)
- macOS: `~/Library/Application Support/motolii/`
- Windows: `%APPDATA%\motolii\`

atomic write(一時ファイル→検証→rename)と、migration前の原本バックアップを採用([反対側レビュー C-3](2026-07-12-prior-art-gap-counter-review.md#c-3-設定マイグレーションhealing--縮小))。

### 4.7 format version と migration

```json
{
  "format_version": 1,
  "active_preset": "motolii.standard",
  "bindings": { }
}
```

- `format_version`を必須とする。未知の上位versionファイルは読み込み拒否+診断メッセージ。
- migrationは**冪等**。未知フィールドは保持(preserve)。
- 破損時はoverlayを無視してbuiltinへフォールバックし、壊れたファイルを`.bak`へ退避 — healing基盤は作らない。

## 5. 解決パイプライン(実装時の契約)

```
OS入力イベント
  → フォーカス文脈(text_input / timeline / global)
  → 論理キー正規化(§4.3)
  → (click/drag判定 — ホスト定数)
  → active preset + user overlay をマージ
  → 衝突なしの CommandId
  → UIコマンド / D2 macro 発行(Documentへはコマンドのみ)
```

- **物理キーをDocumentへ保存しない** — 上記パイプラインの出力は常に編集コマンドまたはtransport制御。
- U2(IPC)は`CommandId`解決**後**の意図のみ受け取る。キー解決はUIシェル層。

## 6. アクセシビリティ境界(M3ガード11)

Slintのa11y(特にTextInput)は未完成。カスタム描画タイムライン(M3ガード2)はアクセシビリティツリーに**載らない**。本節が「どこまでやるか」の正本である。

### 6.1 v1でやる

| 項目 | 内容 |
|---|---|
| キーボード完結操作 | 主要編集・transport・メニュー相当を**カスタマイズ可能ショートカット**で到達可能にする(M3ガード11の代替) |
| Slint標準ウィジェット | AccessKit連携が有効な範囲で、TextInput/Button等の**フォーカス移動(Tab)**とOSスクリーンリーダー連携を**ベストエフォート**で有効化 |
| フォーカス可視化 | キーボードフォーカスリングをsemantic tokenで表示(U0V) |
| IME | [M3ガード1](../specs/M3-ui-integration.md)チェックリスト。変換中ショートカット抑止 |

### 6.2 v1でやらない(明示)

| 項目 | 理由 |
|---|---|
| wgpuタイムラインのスクリーンリーダー木 | カスタム1枚面はノード木を持たない(M3ガード2)。代替=キーボードショートカット+将来の**未決**セマンティックAPI |
| タイムラインの完全キーボードナビゲーション仕様 | クリップ一覧の読み上げ順・仮想化との整合 — **未決**(§7) |
| WCAG 2.2 AAの全項目準拠 | スコープ外。達成レベルの数値目標は置かない |
| ハイコントラスト専用テーマ | semantic tokenでcontrastは担保するが、OS HCモード連動は**未決** |
| プラグインパネルのカスタムa11y契約 | v1は`NodeDesc`自動生成のみ。WidgetHint未決(GAP-13) |

### 6.3 AccessKit / Slint

- SlintビルドでAccessKitを有効にするかは**実装時にS1/U0Vで確認** — 本書では「有効化を試み、TextInputで最低限の読み上げが動くかスパイクで合否」とし、合否基準の数値は**未決**。
- タイムライン・プレビュー埋め込み面に`Accessible`メタデータを付与する公式APIが無い場合、**無理に付けない**(偽のa11yは害)。

## 7. 未決事項(推測で埋めない)

| ID | 内容 | ブロック |
|---|---|---|
| K-1 | 設定ディレクトリの確定パスとCLI `--config`上書き | U2実装前 |
| K-2 | `ContextId`初期集合(`global`/`timeline`/`text_input`/`canvas`等) | U2 |
| K-3 | スナップ既定ON/OFF | U7 |
| K-4 | preset切替時のoverlayモデル(単一 vs preset別ファイル) | U2 |
| K-5 | キーマップのインポート/エクスポート(JSON単体? preset+bundle?) | M3後半可 |
| K-6 | G0: builtin標準割当の競合表(全`CommandId`一覧) | U2/U3並行。本スキーマとは別文書 |
| K-7 | テンキー・JIS/US配列の`key`正規化 | 実装スパイク |
| K-8 | タイムラインキーボードナビのa11y代替仕様 | U3後 |
| K-9 | AccessKit有効化の合否チェックリスト具体項目 | U0V/U1スパイク |

## 8. 実装タスクへの接続

| タスク | 本書から持ち込むもの |
|---|---|
| U2(IPC) | `CommandId`→コマンドの解決はUI側。IPCは意図のみ |
| U3(タイムライン) | pointer `click`/`drag`gesture、衝突検出、G0競合表(K-6) |
| U5(transport) | `motolii.transport.*`のbuiltin割当 |
| U0V/U1 | AccessKitスパイク(K-9)、フォーカス可視化 |

## 9. 完了条件(M3E-9)

- [x] 本設計草案のマージ
- [x] [M3仕様](../specs/M3-ui-integration.md)からの参照
- [x] Documentスキーマへの焼き込みなし(§3)
- [ ] 実装(Rust型・読み書き・編集UI) — M3期間の別チケット
- [ ] G0競合表(K-6) — 別文書
