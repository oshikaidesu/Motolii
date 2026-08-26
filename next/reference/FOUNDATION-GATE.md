# 基盤ゲート

## 目的

この文書は、Motolii の基盤ゲートの理由・範囲・検収条件を示す。現在の段階状態そのものは
`next/reference/foundation/phase.json`が正本であり、`check_foundation_phase.py`、
`plan_waves.py`、`rehearse_parallel.py`がその状態を出力する。引き継ぎ文書を読まないと
分からない状態を作らない。

機能を増やす計画ではなく、既存のコンセプトと配置を保ったまま、Makepad の面へ
共通の意味文法を載せ、利用者検収で「意味の無い散らかり」に見えない状態を作るためのゲートを定める。
iced の view/widget/Theme は凍結ホストであり、このゲートの製品 interface ではない(裁定251/252)。

このゲートを通過するまで、現在の作業を「並列コンポーネント作成段階」と扱わない。
`plan_waves.py` が分割候補を出しても、それは作業を開始してよいという許可ではない。

## 現在の判定

```text
PHASE: FOUNDATION_SERIAL
DESIGN_PROFILE_V0: NOT_CLOSED
CORE_M0_REAL_WINDOW: NOT_CLOSED
PARALLEL_COMPONENTS: LOCKED
3D_CAMERA: DEFERRED_EXTENSION
```

現在は一人の責任範囲で基盤を閉じる段階であり、独立した Browser / Timeline / Stage
コンポーネントを大量に発注する段階ではない。

## 構造としての正本

```text
next/reference/foundation/phase.json
        ↓ check_foundation_phase.py
FOUNDATION_SERIAL / PARALLEL_COMPONENTS=LOCKED
        ↓ plan_waves.py / rehearse_parallel.py
生成された作業割りは「候補」であり、並列解禁ではない
        ↓ Design Profile v0 → CORE-M0実窓
PARALLEL_COMPONENTS=UNLOCKED
```

`phase.json`は機能項目の台帳ではない。段階、依存、owner、並列解禁条件だけを持つ制御構造で、
機能の意味や証拠は既存のnormal-map、axis、component契約、コードから導出する。

## 利用者検収から得たこと

利用者検収では、配色や雰囲気としてのトンマナは存在するが、Panel、Section、操作状態、
編集対象、結果表示の意味が同じ文法で整理されておらず、画面が散らかって見えた。

したがって問題はコンセプトや配置ではなく、次の共通層の欠落である。

```text
Makepad widget / state style
        ↓
Motolii Design Profile
        ↓
shared chrome recipe
        ↓
各 Pane の chrome
        ↓
Stage / Timeline の固有 canvas
```

iced の Theme/recipe 接続は凍結ホスト側の歴史であり、製品経路にしない。

## 維持するもの

- Browser、Stage、Timeline、InspectorというPaneの役割と配置
- 現在のコンセプトと情報の入口
- Stage / Timeline内部の密なcanvas表現
- Documentに属する作品データの色と、UI chromeの色の境界
- `Shell::update → Intent → Document::apply/apply_all → StoreView → Engine/Compositor` の一本の意味経路

DocumentのShape fill、waveform、clipなど作品データの色をUIテーマへ移さない。根拠は
`next/ui/motolii-inspector-pane/src/shape_fill.rs:200` と
`next/ui/motolii-timeline-pane/src/waveform_view.rs:198` にある。

## 意味文法として残すもの

製品 front は Makepad(`next/probes/r7-makepad-panel`)。新しい UI を iced widget に足さない。
借りるのは Iced の見た目ではなく、**操作の意味役割**である。色と密度は Design Profile が与える。

先に共通化するrecipeは次の7つに限定する。実装面は Makepad。iced 側へ移植しない。

- `panel`
- `section_header`
- `button_state`
- `input_focus`
- `toggler`
- `row`
- `tab`

StageとTimelineのcontent/canvas固有表現は、このゲートの共通化対象にしない。

## 現在ある道路と詰まり

### 既にあるもの

- ColorsとDimensionsはJSON正本から読む。`next/ui/motolii-tokens-rs/src/lib.rs:1`
- `UiTheme`はspace、text、size、stroke、targetを意味名へ束ねる。
  `next/ui/motolii-tokens-rs/src/theme.rs:14`
- Iced Themeへの変換は凍結ホスト向けの歴史経路である。`next/ui/motolii-tokens-rs/src/colors.rs:391`
- Settings由来のchrome recipeは凍結ホストのInspectorなどから一部再利用されている。
  `next/ui/motolii-settings-pane/src/chrome.rs:38`
- readabilityとdesign-valuesの静的検査器がある。

### このゲートで閉じるもの

1. `Design Profile v0`をTokens / `UiTheme`の上に置く(枠非依存の意味役割)
2. 7つのshared chrome recipeを一方向依存の共通層へ集める
3. recipeを Makepad の基準面(Settings、Inspector、Export、Shell chrome)へ接続する
4. 同じrecipeをBrowser、Timeline、Stageの Makepad chromeへ接続する
5. Presentation設定のprofile参照または明示的な適用操作を決める
6. 設定変更を表示へ再適用する(再起動または明示操作を含む一つの製品経路)
7. Profileを一箇所変更した時に、接続済みの Makepad chromeが同じ意味役割で変わることを実窓で確認する

## Design Profile v0

Profileはraw colorや個別Paneの数値を集める場所ではない。次の意味役割だけを持ち、
recipeが状態に応じて組み合わせる。

```text
surface: app / pane / raised / hover / selected
text: primary / secondary / muted / disabled
border: default / strong / focus
action: active / selected / warning / danger
layout: spacing / typography / control size / stroke / target
state: normal / hover / pressed / selected / focused / disabled
```

共通値は `dims.theme().space.*`、`text.*`、`size.*`、`stroke.*`、`target.*`から読み、
Pane固有の比率・幾何だけを `dims.components.*`に残す。raw literalを新しい共通値として
増やさない。

## 作業順序

### 0. 基盤ロック

- 現在の配置、Paneの役割、DocumentテーマとUIテーマの境界を変更しない
- P4のような新しい手順ペルソナを追加しない
- Browserコンポーネントの大量発注を停止する
- `plan_waves.py` / `rehearse_parallel.py`は依存確認に使うが、並列開始の許可とは解釈しない

### 1. Design Profile v0を閉じる

- 共通recipe層のownerと一方向の依存を固定する
- 7つのrecipeを Makepad の Settings、Inspector、Export、Shell chromeへ適用する
- 同じrecipeをBrowser、Timeline、Stageの Makepad chromeへ適用する
- `derive_design_values.py --check`、readability、responsibilityを緑にする
- Profile変更後の表示再適用を、再起動または明示操作を含む一つの製品経路として閉じる

### 2. CORE-M0を実窓で閉じる

10秒程度のタイトル作品で、次を一つの閉ループとして検収する。

```text
Composition
→ Text / Shape
→ transform / opacity / text style
→ 2点以上のkeyframe
→ scrub / playback
→ Stageの可視変化
→ Undo
→ save / reopen
→ export
```

各項目は `control → meaning → evaluation → render → observable` の5粒を持つ。
一般的なモーショングラフィックスのM0では音声を必須にしない。P1 lyric-MVを再開する時だけ、
音声デバイス、再生時計、音声同期を別の実窓ゲートとして追加する。

### 3. 並列コンポーネントを解禁する

次の条件をすべて満たすまで `PARALLEL_COMPONENTS` は `LOCKED` のままにする。

- Design Profile v0が一つの正本として選択できる
- 7つのshared recipeが Makepad の基準面で使われている
- 共通UI値に未承認のraw literalが残っていない
- hover / pressed / selected / focus / disabledの状態が実窓で意味として読める
- Profile変更が接続済みの Makepad chromeへ一括適用される
- CORE-M0の作成、編集、再生、Undo、保存、再開、書き出しが実窓で通る
- 残る赤が、意図的な拡張または別の実窓ゲートとして分類されている

解禁後も、各コンポーネントは独立したwrite-setと
`entry → meaning → evaluation → render → observable`を持つ。Shellの共有結線は一つの
WIRE ownerに集約し、意味レーンへ混ぜない。

## 解禁後も最小コアを止めない拡張

次はM0の解禁を待たずに実装しない。

- Observer Cameraの名前付き視点、Orbit、Dolly、Panなどの全機能
- 高度な3Dカメラ・3Dレイヤーソース
- 高度なGraph Editor / velocity編集
- Render Queue、AME連携、交換形式の拡張
- tag / attribute / color editorの一括整理
- frame cacheやanalysis providerの最適化

Render CameraはDocumentと出力の意味、Observer CameraはShellの表示状態として境界だけを
維持する。3Dカメラはこの基盤ゲートを止める理由ではない。

## 検査と証拠

### 静的検査

```bash
MOTOLII_REPO="$(git rev-parse --show-toplevel)"
python3 scripts/derive_design_values.py "$MOTOLII_REPO" --check
python3 scripts/check_ui_readability.py "$MOTOLII_REPO"
python3 scripts/check_responsibility.py "$MOTOLII_REPO"
python3 scripts/check_coherence.py "$MOTOLII_REPO"
git diff --check
```

生成物は手で編集しない。`design-values.tsv`などは生成器から更新する。

### 実窓検収

実窓の証拠は `next/reference/UI-OBSERVATION.md:1` の道路を使う。
オフスクリーン画像を実窓証拠に昇格させない。Profile変更、状態表示、CORE-M0の操作結果を
同じ実窓シナリオへ記録する。

### 出典

- 製品 front: 裁定251/252、`next/probes/r7-makepad-panel`
- Makepad: [oshikaidesu/makepad](https://github.com/oshikaidesu/makepad/tree/motolii-magnify)
- 配置と概念: `next/ui/motolii-presentation-config/src/lib.rs:50`
- 可読性の検査: `next/ui/motolii-tokens-rs/tokens/readability.json:1`
- 人による検収: 2026-08-25の利用者検収
- 凍結ホスト(iced): `next/Cargo.toml:90-97`、rev `73e686ee05efd7d1b61cfea2647186b336d9ab9c`。製品 interface ではない

## 引き継ぎ報告の固定形

```text
PHASE: FOUNDATION_SERIAL / CORE_M0 / PARALLEL_COMPONENTS
PARALLEL: LOCKED / UNLOCKED
OWNER: 今回の単独ownerまたはWIRE owner
DONE: 状態が変わったものだけ
RED: Design Profile、recipe、実窓、5粒の残り
NEXT: この文書の次の番号だけ
DO-NOT-TOUCH: 配置、Documentテーマ、他Paneのwrite-set
EVIDENCE: file:line と実窓scenario id
```

`PARALLEL: UNLOCKED`は、上記の解禁条件をすべて満たした時だけ書く。
stepの静通、componentの静的GREEN、`plan_waves.py`の分割候補だけでは解禁しない。
