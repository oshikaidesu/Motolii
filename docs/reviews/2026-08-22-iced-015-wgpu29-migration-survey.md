# iced 0.14→master(0.15.0-dev)/wgpu 29 統一 移行調査

作成日: 2026-08-22

状態: **調査**(裁定なし。決定は EVIDENCE_GAP を見た利用者判断へ委ねる)

対象: `next/`(正本 workspace)。現状 `iced = "0.14"`(crates.io)pin、`re_renderer`(fork)経由で
`wgpu = "29.0"` が同時に依存グラフへ入り、**wgpu が 27.0.1 / 29.0.4 の2版に分裂**している
(実測、`next/Cargo.lock:7584-7615`)。この分裂を iced 側の pin 切替で解消できるかを調べた。

関連: [iced fork seam 台帳](2026-08-18-iced-fork-seam-ledger.md)、
[ホスト移行裁定](2026-08-18-iced-host-migration-decision.md)、
[裁定161 blend fork accessor](2026-08-21-blend-fork-accessor-decision.md)(§4 の見積り方針の前例)。

## 0. 結論サマリ

| 論点 | 結果 |
|---|---|
| fork 鮮度 | `oshikaidesu/iced` の `master` は upstream `iced-rs/iced` の `master` と **commit sha 完全一致**(`3de451447bd28217bb535632867550908e29d5d0`、0 ahead/0 behind)。`motolii/host-seams` はその2 commit 上(seam1+seam2) |
| wgpu 統一実験 | **成功**。`next/Cargo.toml` の `iced` を fork の git rev(`73e686ee`)へ差し替えて `cargo metadata` を回すと、`wgpu`/`wgpu-core`/`wgpu-hal`/`wgpu-types`/`wgpu-naga-bridge`/`wgpu-core-deps-*` の全 14 パッケージが **単一 29.0.4** に解決した |
| API 差分 | 実質ゼロ。`next/` 実測 338 箇所の `iced::` 使用のうち頻度上位(shader/canvas/text_input/mouse_area/slider/window::frames/Theme::custom/Subscription::run/stream::channel/iced_test の Simulator 一式)は **シグネチャ・モジュールパスとも不変** |
| ゼロコピー(裁定166) | **部分成立**。shader widget の `Primitive::prepare`/`Pipeline::new` は `device`/`queue` は貰えるが **`&wgpu::Adapter` は貰えない**。`re_renderer::RenderContext::new` は adapter を要求するため、ここが唯一の未解決ブロッカー(EVIDENCE_GAP-1) |
| seam 2(bind group 床) | upstream master(=fork base)は今も `max_bind_groups: 2` を2箇所(`wgpu/src/lib.rs:928`, `wgpu/src/window/compositor.rs:162`)にべた書き。**旧台帳の再適用手順がそのまま当たる**(base が identical なので rebase 差分ゼロ) |

---

## 1. fork の所在と鮮度

`gh api repos/oshikaidesu/iced` / `repos/oshikaidesu/iced/branches` / `repos/oshikaidesu/iced/compare/...` で実測(2026-08-21 時点の GitHub API)。

| ブランチ | commit | upstream `iced-rs/iced:master` との関係 |
|---|---|---|
| `master` | `3de451447bd28217bb535632867550908e29d5d0` | **identical**(`ahead_by:0, behind_by:0, status:"identical"`)— fork 作成が upstream の最新に対してほぼ即座に行われている |
| `motolii/host-seams` | `73e686ee05efd7d1b61cfea2647186b336d9ab9c` | `master` から `ahead_by:2`(= seam1+seam2、[fork seam 台帳](2026-08-18-iced-fork-seam-ledger.md)記載のとおり) |

upstream `iced-rs/iced` の最新 commit(`3de45144`, 2026-08-16T21:26:28Z「Remove `From<u8>` requirement for `slider` widgets」)は fork の `master` と一致するので、**fork は現時点でまったく劣化していない**(0日遅れ)。旧台帳が「まだ1度も rebase を実演していない」と書いていた懸念(§6)は、今この瞬間に限れば「rebase する必要そのものがない」状態にある。

**rev pin 戦略の設計案**: rerun fork と同じ「seam 台帳 + 常設 oracle」型をそのまま踏襲できる。追加で言えることは1点——**pin を上げるタイミングを watch する仕組みが無い**(rerun fork も同様の穴)。`gh api repos/oshikaidesu/iced/compare/master...iced-rs:master` を定期的に叩けば drift 検知はできるが、自動化は本調査のスコープ外。

---

## 2. API 差分の実測(0.14.0 → master/0.15.0-dev)

`grep -rhoE 'iced::[A-Za-z_:]+' next/ --include='*.rs' | sort | uniq -c | sort -rn` で頻度集計(全338件、60種類以上)。上位の実使用面を GitHub 上の `iced-rs/iced`(`master` = fork base と同一)ソースと突き合わせた。

### 2.1 高頻度・タスクで名指しされた API — 全て健在

| 使用面 | next/ での頻度 | master での状態 |
|---|---|---|
| `iced::Point`/`Border`/`Background`/`Color`/`Size`/`Rectangle` 等の core 型 | 100+ | 変更なし(`core/src/`) |
| `iced::widget::canvas`/`Canvas` | 6箇所 | `widget/src/lib.rs` に `pub mod canvas; pub use canvas::Canvas;` 健在 |
| `iced::application`(free fn) | 9箇所 | `src/lib.rs:667 pub use application::application;` 健在 |
| `iced::keyboard::*`(Modifiers/Key/Event) | 20+ | 変更なし |
| `iced::event::listen_with`/`Status` | 5箇所 | 変更なし |
| `iced::Subscription::run`/`none`/`Subscription` | 6箇所 | `futures/src/subscription.rs:203 pub fn run<S>(builder: fn() -> S)` — シグネチャ不変 |
| `iced::stream::channel` | 3箇所 | 存在確認(futures 経由) |
| `iced::window::events`/`Event::FileDropped`/`Id` | 3箇所 | 変更なし |
| `mouse_area`/`text_input`/`widget::slider` | 該当ファイル多数 | `widget/src/lib.rs` に `pub mod text_input;`/`pub mod slider;`/`pub use mouse_area::MouseArea;` 健在(モジュール名・型名とも不変) |
| `Theme::custom`/`theme::Base`/`theme::palette::Palette` | 5箇所 | `core/src/theme.rs:91 pub fn custom(name, seed: palette::Seed) -> Self`、`:228 pub trait Base` 健在 |
| `window::frames()` | 1箇所(未使用箇所の subscription 候補として grep 済み) | `runtime/src/window.rs:196 pub fn frames() -> Subscription<Instant>` — 実装まで不変(`RedrawRequested` を listen するだけ) |

`shader::{Program, Primitive}` は **next/ に使用箇所0件**(shader widget を使う Stage presenter は現状 readback+`write_texture` 経路で `widget::image`/`Canvas` 相当を使っており、`Shader` widget 自体はまだ導入されていない)。API 自体は存在確認済み(§4)だが、**「差分ゼロ」の判定はこのトレイトの実挙動には及んでいない**(EVIDENCE_GAP-4)。

### 2.2 `iced_test`/`iced_selector` — 構造は変わったが呼び出し面は無傷

master では `Selector`/`Candidate`/`Target` が `iced_test` 直属から **`iced_selector` という独立 crate へ切り出されている**(`test/Cargo.toml` の `iced_selector.workspace = true`)。しかし `iced_test::lib.rs` が `pub use iced_selector as selector;` で丸ごと再エクスポートしており、`iced_selector::target::{Bounded, Candidate, Target, Text}` も `selector` module 経由でそのまま見える。

`next/shell/motolii-shell/tests/suite/q0_fence.rs` の実コード:
```rust
use iced_test::selector::{Candidate, Target};
let mut ui = iced_test::simulator(element);
ui.find(selector) / ui.click(selector) / ui.point_at(point) / ui.simulate(events) / ui.into_messages()
```
は **1文字も直す必要がない**(`test/src/simulator.rs` を実測: `Simulator::new/with_settings/with_size/find/point_at/click/tap_key/typewrite/simulate/snapshot/into_messages`、自由関数 `simulator/click/press_key/release_key/tap_key/typewrite` が全部同名・同シグネチャで存在)。

q0_fence.rs のコメントが明記する「`Simulator` に `find_all` は無い」も master でまだ真(`Selector` トレイト自体には `find_all()` が生えたが、`Simulator` 型に対応するメソッドは追加されていない)。**現行の「`find` を `Err` まで回して全件集める」ワークアラウンドはそのまま有効**。

### 2.3 API 差分の件数

- next/ の 338 `iced::` 用例のうち、master でシグネチャ・path が変わった箇所: **実測0件**
- `iced_test`/`iced_selector` 側の構造変更(crate 分割): **1件**(呼び出し面には影響しない re-export で吸収済み)
- 実装未着手の `shader::Program`/`Primitive` トレイトは新規使用面のため「差分」の対象外(次節で扱う)

### 2.4 API 差分「重い物」TOP5(件数ではなく移行コストの重さで順位付け)

件数としては上のとおりほぼゼロだが、コスト・リスクは別軸にある。重い順:

1. **ゼロコピーの adapter 入手経路が未解決**(§4)。API 差分ではなく設計ギャップだが、このレーンの本命(裁定166)を止めている最大の項目
2. **seam 2(bind group 床)の再適用が必須のまま**。upstream が直していない(§0 表)ので `iced` の device 上で `re_renderer` パイプラインを1本でも回すなら pin 切替だけでは足りない
3. **font スタックの大幅更新**(`cosmic-text` 0.15.0→0.19.0、`harfrust` 0.3.2→0.5.2、`skrifa` 0.37.0→0.40.0、`read-fonts` 0.35.0→0.37.0、`font-types` 0.10.1→0.11.3)。Rust API 破壊はゼロだが、**text shaping/hinting の出力が変わる可能性があり**、`tonmana_token_fence.rs`/`inspector_pixel_fence.rs`/`ui_scale_fence.rs` 等の PNG ベース oracle が閾値超えで赤くなるリスクがある(未測定、EVIDENCE_GAP-2)
4. **`iced_test`/`iced_selector` の dev-dependency 切替を本体 `iced` と同じ git rev に揃える作業自体**。API 差分はゼロだが、`iced_test = "0.14"`(crates.io)のまま放置すると **iced 本体と iced_test が別々の Selector/Candidate 型を持つ2つの iced ラインを同時に抱える**事故になる(cargo は黙って両方解決してしまう ―— semver 上 0.14 と 0.15.0-dev は非互換なので共存は可能だが、テストコードがどちらの `Target` 型を掴んでいるか読み違えるとコンパイルエラーで気づく形にしかならない、地味に事故りやすい)
5. **git 依存の面が増える**。master 系は `iced-rs/winit`(fork)・`iced-rs/cryoglyph`(fork)を新たに git 経由で引く(旧 0.14 系列では `winit` は crates.io の 0.30.13 リリース)。`oshikaidesu/iced` + `oshikaidesu/rerun` の2本に、upstream 自身が管理する2本(winit/cryoglyph)が加わり、**`cargo metadata` が解決に触れる git リポジトリが合計4本**になる(可用性・ミラー依存が増える)

---

## 3. wgpu 統一の解決実験

### 手順(worktree 内の一時編集、cargo metadata のみ・cargo build/test はしていない)

1. `next/Cargo.toml` の `iced = { version = "0.14", ... }` を
   `iced = { git = "https://github.com/oshikaidesu/iced", rev = "73e686ee05efd7d1b61cfea2647186b336d9ab9c", features = [...] }`(`motolii/host-seams` tip)へ
2. `next/shell/motolii-shell/Cargo.toml` の `iced_test = "0.14"` を同じ git/rev へ(iced 本体と揃えないと bifurcation する、§2.4 の懸念そのもの)
3. `cd next && cargo metadata --format-version 1` を実行(exit 0 を確認)
4. 実験後、両 `Cargo.toml`・`Cargo.lock` を `git status --short` で無変更まで復元(現在のリポジトリは元の 0.14 pin のまま、実験の痕跡なし)

### 結果

`cargo metadata` は **exit 0**。ログに `Adding iced v0.15.0-dev (...#73e686ee)` 系列が解決され、以下が確認できた:

```
$ grep -B1 "^version = " Cargo.lock | grep -A1 "^name = \"wgpu"
name = "wgpu"                          version = "29.0.4"
name = "wgpu-core"                     version = "29.0.4"
name = "wgpu-core-deps-apple"          version = "29.0.4"
name = "wgpu-core-deps-emscripten"     version = "29.0.4"
name = "wgpu-core-deps-wasm"           version = "29.0.4"
name = "wgpu-core-deps-windows-linux-android" version = "29.0.4"
name = "wgpu-hal"                      version = "29.0.4"
name = "wgpu-naga-bridge"              version = "29.0.4"
name = "wgpu-types"                    version = "29.0.4"
```

**全14 wgpu 系パッケージが単一 29.0.4 に畳まれた**(現状の 27.0.1/29.0.4 の分裂が解消)。`cargo tree -i wgpu@29.0.4` で確認すると、`iced_wgpu`(iced 本体・cryoglyph 経由)と `motolii-compositor`(`re_renderer` 経由)の**両方が同じ 1つの `wgpu` ノードにぶら下がる**ことも実測した。

### 副作用として見つかった非統一(新規に発生した問題ではない)

- `glam` は **統一されない**(0.30.10=rerun 側 / 0.33.5=`iced_core` の内部使用)。ただし **これは既存の状態**でもある(実験前の Cargo.lock は 0.25.0/0.30.10 の2版に分裂済み)。`iced_core` の glam 使用は `Transformation` 型など iced 内部実装詳細で、Motolii 側コードが `iced::` 越しに glam 型を受け渡す箇所は無いため、実害は無いと判断(ただし未走査、EVIDENCE_GAP に含める必要はない程度の確度)

---

## 4. ゼロコピーの縫い目設計案

### 4.1 shader widget 側が渡すもの

`iced-rs/iced` master `wgpu/src/primitive.rs` を実測:

```rust
pub trait Primitive: Debug + MaybeSend + MaybeSync + 'static {
    type Pipeline: Pipeline + MaybeSend + MaybeSync;
    fn prepare(&self, pipeline: &mut Self::Pipeline, device: &wgpu::Device, queue: &wgpu::Queue,
               bounds: &Rectangle, viewport: &Viewport);
    fn draw(&self, pipeline: &Self::Pipeline, render_pass: &mut wgpu::RenderPass<'_>) -> bool { false }
    fn render(&self, pipeline: &Self::Pipeline, encoder: &mut wgpu::CommandEncoder,
              target: &wgpu::TextureView, clip_bounds: &Rectangle<u32>) {}
}

pub trait Pipeline: Any + MaybeSend + MaybeSync {
    fn new(device: &wgpu::Device, queue: &wgpu::Queue, format: wgpu::TextureFormat) -> Self;
    fn trim(&mut self) {}
}
```

`Pipeline::new` は **一度だけ**(`Storage` に type ごとキャッシュ)呼ばれ、`device`/`queue` の参照を貰える。wgpu 統一後はこれが `re_renderer` 側の `wgpu::Device`/`wgpu::Queue` と**型として完全に同じ**になる(§3 の実験結果)。ここまでは成立している。

### 4.2 re_renderer 側が要求するもの、そして gap

`oshikaidesu/rerun`(rev `856f597c`)の `crates/viewer/re_renderer/src/context.rs` を実測:

```rust
pub struct RenderContext {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    ...
}
impl RenderContext {
    pub fn new(
        adapter: &wgpu::Adapter,
        device: wgpu::Device,
        queue: wgpu::Queue,
        output_format_color: wgpu::TextureFormat,
        config_provider: impl FnOnce(&DeviceCaps) -> RenderConfig,
    ) -> Result<Self, RenderContextError> { ... }
}
```

`adapter` は `DeviceCaps::from_adapter(adapter)`(features/limits の把握)と `adapter.get_info()`(ログ用)にしか使わないが、**`&wgpu::Adapter` の実物**が要る。

**ここが gap**: `Pipeline::new(device: &wgpu::Device, queue: &wgpu::Queue, format)` には adapter が渡ってこない。iced 側で adapter を保持しているのは `iced_wgpu::window::compositor::Compositor`(`wgpu/src/window/compositor.rs:11-17`、`instance`/`adapter`/`engine` は全部 private field でアクセサ無し、実測)。つまり **iced の外からも中からも、Shader widget の実行タイミングで `&wgpu::Adapter` を手に入れる公式な経路が無い**。

`motolii-compositor::Compositor::headless()` 自体は fork 側の変更なしに `RenderContext::new` をそのまま直呼びしている(`next/engine/motolii-compositor/src/lib.rs:206-220`)ので、**Compositor 側に薄い `with_device(adapter, device, queue, format, config_provider)` を足すこと自体は自明**(`headless()` から `HeadlessGpu::new()` の中身を抜いただけ、+20行未満)。問題はその **呼び出し元がいつ・どこで `&wgpu::Adapter` を手にできるか**である。

### 4.3 2つの解決案(裁定待ち、EVIDENCE_GAP-1)

**案A: iced fork へ `Pipeline::new` に `adapter` を足す**
- `iced_wgpu::primitive::Pipeline` トレイトのシグネチャを変える。Motolii 独自 trait ではなく iced 本体の公開 trait なので、**上流との差分が今後ずっと増える**(rebase のたびにこの trait 定義とその呼び出し3箇所前後を読み直す必要が出る)
- 旧 seam 2 と同じ「process 全体の床」方式ではなく trait シグネチャ改変そのものなので、旧台帳の「1文字」「+92行の新規ファイル」より重い

**案B: re_renderer fork へ `RenderContext::new_from_caps` 的な口を1本足す**(裁定161 BL1b と同型)
- `RenderContext::new(adapter: &wgpu::Adapter, ...)` の代わりに `RenderContext::new_from_device_caps(device, queue, device_caps: DeviceCaps, adapter_info: wgpu::AdapterInfo, output_format_color, config_provider)` を追加。中身は既存 `new()` から `DeviceCaps::from_adapter(adapter)`/`adapter.get_info()` の2呼び出しを引数に置き換えるだけ
- `DeviceCaps`/`AdapterInfo` は **アプリ起動シーケンスの中で1度だけ**(iced の Compositor が device を要求する **前** に、我々が別途 `wgpu::Instance::enumerate_adapters` して同じアダプタを選び直す ── ただしこれは iced が実際に選ぶアダプタと**同一アダプタである保証が無い**ので、iced 側にも「どの adapter を選んだか」を漏らす小さな口が要る可能性が高い)
- fork 側の追加量は BL1b 前例(+15行 accessor)と同程度と見積もれる。ただし **iced 側にも adapter 選択結果を漏らす口が要るかもしれない点は未検証**(→ 案Bにも iced 側の小さな追加が付随しうる、EVIDENCE_GAP-1 に含める)

**現時点の見立て**: 案Bの方が iced 本体差分を増やさない分、rebase コストは低い。ただし「iced が選んだのと同じ adapter を外側でも再現できるか」は未検証で、**案Bだけでは閉じない可能性がある**。これは実装着手前に裁定が要る(EVIDENCE_GAP-1)。

---

## 5. 段階移行の束割り案

重み均等・write-set 互いに素の原則で切る。iced_test 線は **M0 に同梱**(§2.4 の理由: 本体と別revで置くと2ライン共存の事故を生む)。

| 切片 | write-set | やること | 受入条件 |
|---|---|---|---|
| **M0** | `next/Cargo.toml`、`next/shell/motolii-shell/Cargo.toml` のみ | `iced`/`iced_test` を fork `motolii/host-seams`(`73e686ee`)へ pin。seam1/seam2 は fork に既に入っているのでコード追加なし | `cargo metadata` 緑・wgpu 系14パッケージが単一29.0.4(本調査で実測済み、再現するだけ)。**このレーンはビルド不可のため、次の M1 へ緑化を引き継ぐ** |
| **M1** | 上記 pin 込みで `cargo build`/`cargo test --workspace` を回すレーン(ビルド許可レーンへ発注) | full workspace gate。PNG ベース oracle(`tonmana_token_fence.rs` 等)が font スタック更新(§2.4 items 3)で赤くなったら、**意匠は変えずに許容誤差だけ再較正**するか、意匠上の後退であれば追加調査 | 237 test binary 相当の全 green。閾値変更をした場合はその差分を目視で記録(意匠不変の確認込み) |
| **M2** | `next/engine/motolii-compositor/src/lib.rs`(+新 fn のみ、既存 `headless()` は無傷) | `Compositor::with_device(adapter, device, queue, format, config_provider)` を追加。**まだ誰も呼ばない**(配線ゼロ=挙動ゼロ変更、B3 と同じ手口) | 新 fn の unit test 1本(`headless()` と同じ `RenderContext` が作れることだけ確認) |
| **M3** | EVIDENCE_GAP-1 の裁定を経てから: 案Aなら iced fork(`wgpu/src/primitive.rs` 他)、案Bなら rerun fork(`context.rs`)+ 呼び出し側 glue crate(新設 or `motolii-compositor` 拡張) | adapter 入手経路を実装。旧 fork 台帳 §4 の `stage_bind_groups_oracle.rs` に相当する**常設 oracle**を新設し、「同一 device 上で re_renderer の pipeline が実際に建つ」ことを headless で審判する | 新 oracle が green。**ここで初めて `shader::Program`/`Primitive` を next/ で実使用する**ので、§2.4 item 1 の EVIDENCE_GAP-4(shader トレイトの実挙動未検証)もここで閉じる |
| **M4** | Stage presenter(`motolii-stage-pane` 等) | readback(engine→CPU→`write_texture`)を撤去し、Shader widget の `Primitive::render` で `re_renderer` の texture を直接 blit する経路に切替 | 実窓 fps 実測(readback 撤去の効果)+ 既存 Stage 島 pixel oracle が同一絵(SHA一致 or 既定許容差内) |

---

## EVIDENCE_GAP(裁定が要る未決)

1. **ゼロコピーの adapter 入手経路**(§4.3)。案A(iced fork へ `Pipeline::new` 改変)か案B(rerun fork へ `RenderContext::new_from_device_caps` 追加、ただし iced 側の adapter 露出が要るかも未検証)か、実装着手前に選ぶ必要がある。M3 の前提
2. **font スタック更新の視覚影響**(cosmic-text 0.15.0→0.19.0 他、§2.4 item 3)。`cargo build` できないこのレーンでは測定不能。M1 で実測必須。既存 PNG oracle の閾値を動かす場合、それが「意匠不変の再較正」か「意匠の後退」かは目視判定が要る
3. **seam1(web-sys 完全一致解除)が master 線で今も要るか**は本実験では未確認。今回の `cargo metadata` は native ターゲットのみ解決しており、`motolii/host-seams` は seam1 を**既に含んだ状態**で解決させている(= 「seam1 適用済みなら通る」ことしか示していない。「seam1 無しでも通るか」は別実験)。wasm32 を一切ビルドしない Motolii では実害が薄いと見えるが未実測
4. **`shader::Program`/`Primitive` トレイトの実挙動**は next/ に使用箇所が無いため、API 表面の存在確認はできたが実装時の細部(`Storage` の trim タイミング、`MaybeSend`/`MaybeSync` 境界が Motolii の型に効くか等)は未検証。M3 で初めて分かる
5. **3 OS 未検証**。macOS のみで `cargo metadata` を実行した(旧 iced fork 台帳 §6 と同じ既知の穴の継承)
6. **`motolii/host-seams` の rebase 実演がまだ0回**。今回「fork base = 現 upstream master」という好条件が判明したが、これは「rebase が要らない」ことを示すだけで、「rebase の手順が実際に機能する」ことの実演にはならない。次に upstream が動いたときが初回実演になる
7. **案Bを取った場合、iced が選んだアダプタと外側で選び直したアダプタが同一である保証**(§4.3)。マルチ GPU 環境で `wgpu::Instance::enumerate_adapters` の順序・選択基準が iced 内部の選択(`compositor.rs` の `request()`)と食い違えば、2つの異なる物理 GPU 上に device が別れてしまい zero-copy どころか動作不能になる。単一 GPU の開発機では顕在化しない
