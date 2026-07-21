# M3 U1a-1 静止viewport契約

作成日: 2026-07-21
状態: **決定 / U1a-1実装完了**

## 1. 目的

U1a-1は、M3の最初の可視製品shellとして、次の一本だけを成立させる。

`bootstrapが渡した同一Document`
`→ build_document_frame_graph`
`→ render_graph_cached(Quality::DRAFT)`
`→ 同一device上の独立display copy`
`→ egui native texture`

旧`cursor/m3-u1a-1-night`は調査材料に限り、直接mergeしない。同branchの
`fixed_document()`、起動前`spawn`直後の`join`、単一display texture、固定panel
placeholder、登録回数counterを現行契約の証拠とは扱わない。

## 2. U1a-1の閉集合

U1a-1で実装してよいものは次だけである。

1. `GpuCtx::new_for_ui()`が作った`Instance / Adapter / Device / Queue`を
   `egui_wgpu::WgpuSetup::Existing`へ渡す製品shell
2. bootstrapが所有するin-memoryの固定Document fixtureを、event loop開始前の
   setup workerへ`Arc<Document>`で渡すprivate seam
3. そのDocumentを既存の`build_document_frame_graph`と
   `render_graph_cached(..., Quality::DRAFT)`で一度だけ評価する静止frame
4. `RenderedFrame`とは別寿命の、`TEXTURE_BINDING | COPY_DST | COPY_SRC`を持つ
   `Rgba8Unorm` display slotとGPU上のtexture-to-texture copy
5. display slotの安定`TextureView`を、`eframe::CreationContext`でrendererを得た時に
   一度だけ`register_native_texture`へ登録し、`App::ui`では得られた`TextureId`を
   投影する経路
6. React `component-map.json`の`surface.stage-viewport`に対応する、中央Stage面だけの
   非永続bootstrap shell。Browser / Inspector / Timeline / statusを配置する組み込み
   既定presetはU1a-2へ送る
7. shellを自動終了しlifecycle列を実行できるprivate試験制御と、GPU/displayが利用できる
   環境での製品binary launch smoke

fixture Documentは製品の既定project、template、公開constructor、保存形式ではない。
U2b-1でwriterの`Arc<Document>`購読を接続するまで、実projectをshellへ注入する公開APIを
発明しない。

## 3. 「同じDocument frame」の機械オラクル

合格は「何か色が見えた」ではなく、単一のprivate閉路で固定する。

1. bootstrapはfixture Documentを構築してからsetup workerへ`Arc<Document>`を渡す。
   worker内部で別Documentを生成しない
2. 製品shellとheadless統合試験は、`Arc<Document>`、評価時刻、`FrameDesc`を受け、
   canonical graph build、`render_graph_cached`、独立display slot生成、GPU copyまでを
   完結する同じprivate preparation入口を呼ぶ。別々のrender試験とcopy試験を足して
   閉路の代わりにしない
3. headless統合試験は、内容だけが異なる2つのfixture Documentをその入口へ渡し、
   返された各display slotをdownloadする。期待画素は最初の実行結果を転記せず、fixtureの
   既知paramと既存render oracleから独立に固定する。各slotが対応する期待画素とbyte一致し、
   2つのslot同士は不一致であることを確認する
4. 同じ試験内で各stable viewをheadlessの`egui_wgpu::Renderer`へ一度登録し、
   register-once seamを再度呼んでも基底renderer登録counterが1のまま、cache済みの同じ
   `TextureId`が返ることを確認する。製品のCreationContextも、そのpreparation結果の
   同じregister-once seamだけを呼ぶ
5. 製品の`GpuOrigin::UiShared`経路では`download_rgba`と`poll(Wait)`が既存の型付きerrorで
   拒否されることを別の負例として確認する

display slotの`COPY_SRC` usageはheadless oracle用であり、UiSharedのreadback許可ではない。
test用headless readbackは製品previewのCPU bridgeではない。readback helperを
`motolii-ui`の製品公開APIへ出さず、製品shellから呼べる分岐も作らない。

## 4. 解像度、window、lifecycle

- Compositionが所有するのは有理aspectであり、window logical px、physical px、DPI、
  display scale、viewport sizeではない
- U1a-1の静止fixture用`FrameDesc`はtest/bootstrapの出力条件であり、Documentへ保存せず、
  Compositionのaspectを変更しない
- fixture Compositionは`Composition::new_v1()`またはaspect専用の値から作る。
  `FrameDesc.width / height`を`Composition::try_new`のaspect引数へ渡さず、window寸法から
  どちらも導出しない
- window resizeとDPI変更はegui上の表示矩形だけを変える。静止fixtureの出力texture、
  Document、評価時刻を変更しない
- minimize/hide中は新しいDocument評価、GPU copy、native texture登録を行わない
- restore後は同じdisplay slotと同じ`TextureId`を再投影する。device lost時の再構築は
  U1a-1で暗黙実装せず、既存`GpuRuntimeError`を保持して停止する
- display slotは、それをsampleするegui Appより先にdropしてはならない。
  `GpuCtx`のruntime health stateもAppの生存中は保持する

自動試験は、resize、scale-factor変更、minimize、restoreを表すshell lifecycle入力列に対し、
Document serialize、display slot identity、native registration count、render/copy countが
不変であることを検査する。この試験はadapterの反応だけを証明し、OS event配送を証明したと
扱わない。windowを作れる開発環境では、製品binary自身を実際にresize、minimize、restoreし、
復帰後も同じslot/registrationでpaint/presentが再開したことをraw log付きで確認する。
見た目の良否は主張せず、少なくともrestore後のpresent回数増加と同じTextureIdのpaintを
記録する。CIでdisplayが無い場合は構造試験をskipせず実行し、window smokeだけを依存欠如
として明示skipする。

実monitor間のDPI/scale-factor変更は複数scaleのmonitorを要するため、U1a-1の完了を
hardware偶然へ依存させない。U1a-1はscale-factor eventでDocument、出力texture、slot、
register回数を変えないadapter不変条件までを担い、実monitor移動は別window/monitorを扱う
U1eのplatform acceptanceで必須化する。[egui採用判断](2026-07-18-m3-egui-selection.md)の
Apple M4 / Metal証拠は方式選定の証拠として継承するが、製品shellのOS smokeを代替しない。

## 5. threadと待機

- Document graph build、`render_graph_cached`、display copyを`eframe::App::ui`、
  winit/egui event callback、repaint callbackから呼ばない
- U1a-1はone-shot静止frameなので、`run_native`開始前にsetup workerの完了を待ってよい。
  event loop開始後のjoin、channel受信待ち、`device.poll(Wait)`は禁止する
- 起動後のrequest mailbox、render worker常駐、generation、連続seekはU1b-1/2で実装する。
  U1a-1のone-shot setupを常駐workerの代用品として公開しない
- setup失敗、thread spawn失敗、panic、size/format不一致は型付きerrorにし、公開入口から
  panicしない

## 6. register-onceの正確な意味

仕様中の「pool生成時に登録」は、texture生成とrenderer登録が同じ時点という意味ではない。

1. display slot生成時にtextureと安定viewを一度作る
2. `eframe::CreationContext`で既存deviceに対応するrendererを得た最初の一回だけ登録する
3. `App::ui`、resize、DPI変更、minimize/restoreで再登録しない
4. U1a-1の単一slotはApp終了まで保持し、途中解除・差し替えを行わない。終了時はeframe
   renderer registryとslotが同じshell teardownで破棄されるため、rendererへの途中
   `free_texture`入口を発明しない。slot差し替えを導入する後続ticketで解除順を別途固定する

`OnceLock<TextureId>`だけを合格根拠にせず、registerを呼べる関数がframe paint経路から
到達不能であることをsource構造試験でも確認する。

## 7. bootstrap shellと視覚境界

React現行参照からU1a-1へ移すのは`surface.stage-viewport`の中央Stageという役割だけである。
JSX、CSS px、色、radius、spacing、文言、DOM階層を移さない。

- Browser / Inspector / Timeline / statusをU1a-1へplaceholderとして置かない
- 五面の組み込み既定preset、panelのsplit、tab、resize、hide、restore、resetと
  `egui_tiles`投影はU1a-2
- layoutの所有層、保存寿命、version付き形式はU1a-3
- 製品theme token、icon、component state、具体色・spacingはU0e-3
- U1a-1のStage shellは操作、Document値の編集、selection、diagnostic、保存状態を持たない
- `egui::Panel` ID、logical size、egui memoryをDocument、User settings、workspace形式へ出さない

したがってU1a-1の完成画像はG0-6Hの視覚審判対象ではなく、製品visual完成を主張しない。

## 8. 公開境界と依存

- egui / eframe / egui-wgpu / egui-winit / winit / egui_tiles / wgpuの型を
  `motolii-ui`外の製品crate、Document、plugin、render/evalの公開APIへ追加しない
- U1a-1がRust公開面へ追加してよいのは、toolkit型を含まないshell起動errorと引数なしの
  shell起動関数だけ。製品binaryはその関数を呼ぶ。fixture Document、display slot、
  lifecycle model、TextureId、自動終了制御はprivate
- `motolii-ui`から既存の`motolii-core`、`motolii-doc`、`motolii-gpu`、
  `motolii-render`、`motolii-eval`、`motolii-plugins-firstparty`を利用してよいが、
  逆向き依存を追加しない
- `UiDeviceParts`、`required_features()`、`check_minimum_limits()`、
  `build_document_frame_graph`、`render_graph_cached`、`RenderSession`、
  `first_party_runtime()`を再実装しない

Rerun source、crate、assetはU1a-1の根拠・依存・移植に使わない。今回の合否はMotoliiの
fixtureと既存境界だけで閉じる。

## 9. 必須負例

次は自動拒否またはsource監査へ固定する。

- UI/event-loop callback内のDocument render、join、blocking receive、sync readback
- second device、CPU pixel upload bridge、preview専用render関数
- `RenderedFrame.texture`をdisplay slotとして直接使い回すこと
- `App::ui`またはlifecycle eventごとのnative texture登録
- resize/DPI値からComposition aspect、Document、評価時刻、出力texture寸法を変更
- minimize/hide中の再render、copy、register
- size/format不一致の`assert!`/panic
- `egui_tiles::Tree`、`TileId`、egui memory、panel px値の保存
- Browser / Inspector / Timeline / status placeholder、Document編集、再生、seek、mailbox、
  generationの追加
- theme値、独自icon、golden期待値の更新
- toolkit型を含む公開signatureまたは`motolii-ui`外へのtoolkit依存

## 10. U1a-1完了条件

1. 固定fixture Documentのcanonical graph/render結果が独立display slotへGPU copyされる
2. 同じprivate preparation入口を使うheadless統合oracleで、2つの異fixtureが各期待画素と
   byte一致し、互いには異なる。別々のrender/copy試験で代用しない
3. UiShared経路のCPU readback / `poll(Wait)`が型付き拒否される
4. product shellが`WgpuSetup::Existing`で同じdeviceを使い、第二deviceを作らない
5. registerはCreationContextで一度だけ、paint/lifecycle列で回数とTextureIdが不変
6. event loop開始後のrender/join/blocking receiveがsource構造上存在しない
7. lifecycle入力列でDocument serialize、slot identity、render/copy/register countが不変。
   window利用可能開発環境では製品binaryの実resize/minimize/restore smokeとraw logも成功。
   実monitor DPI移動はU1eへ送ったことを未証明として明記する
8. bootstrap shellは中央Stageだけで、U1a-2/U1a-3/U0e-3の意味を持たない
9. toolkit依存方向、Document serde面、plugin契約に差分がない。Rust公開面の差分は
   toolkit-freeなshell起動関数とerrorだけ
10. `cargo fmt --all -- --check`、`./scripts/check-docs.sh`、
    `./scripts/check-ui-toolkit-deps.sh`、`cargo clippy --workspace --all-targets -- -D warnings`、
    `cargo test --workspace`が通る

これを満たした時だけU1a-1を完了とし、次にU1a-2を単独実行する。
