# probe皮の縫い目表 — 繋ぐだけレーンの発注原簿(2026-08-29)

状態: **運転台帳**(利用者方針: 「ただ繋ぐだけ。1本~10分、並列可。現場に任せると自作し出すので
繋ぐ先を先に名指しする」)。皮コンセプトの動機 = 過去にRerun/旧世界の在庫を無視した再発明事故。

各レーンの発注書はこの表の1行を引用し、**繋ぐ先の室名を1行+「出たか」の検査**で渡す
(全読みさせない)。「委託可否」が既に裁定済みの項目は
[rerun-technical-delegation.tsv](../../next/reference/generated/rerun-technical-delegation.tsv) が正本。

## 縫い目表

| # | 皮の受け口(motolii/probe) | 繋ぐ先の室(在庫) | 繋ぎの一行 | 自作禁止(現場がやりがちな事故) | 粒 |
|---|---|---|---|---|---|
| S1 | `stage_widget.rs`(TestTriangle撤去) | `motolii_engine::Engine::with_device` + `render_frame_to_texture`(app/engine/motolii-engine/src/render.rs:424) | probeの共有device/queueでEngineを建て、Clock時刻の`view`を渡してtexture受取→既存のResource表示経路へ | 矩形合成の自作・`rectangles.rs`直叩き・`render_frame`(Vec\<u8\> readback)経路の流用。合成器は既にある | 10分 |
| S2 | 層クリック=選択(timeline_widget/層ヘッダ) | `motolii-shell-state`(app/ui)の`Session`/`KeySelector` | probeの選択状態をSessionへ寄せ、Inspector identが読む | 独自selection構造体の新設 | 10分 |
| S3 | ~~M/S/L点灯~~ **済**(Document実値+SetAttrs) | `Intent::SetAttrs{hidden,solo,locked}`(motolii-store) | M=hidden/S=solo/L=locked 1:1。書きはIntent channel1本(単一書き手の柵) | Documentフィールド直書き・点灯だけの飾りのまま放置 | 10分 |
| S4 | Inspector値(起動時固定→追随) | `StoreView::value_at`(使用中の口) | 選択層+Clock時刻で毎フレーム再読 | 値キャッシュ層の自作 | 5分 |
| S5 | Inspector値セルのdragスクラブ | motolii-storeのproperty書きIntent(document.rsの既存variantを写す) | drag量→値→Intent apply→再読。hint行「Drag to scrub」を本物に | 補間・clamp・undoの自作(全部store側にある) | 10分 |
| S6 | Timeline行モデル(CanvasRow自作中) | `motolii-timeline-projection`(app/ui)+`TimelineFoldState` | 行の投影(グループ畳み含む)を既製projectionへ差し替え | CanvasRowへのグループ/畳み機能の増築 | 10分 |
| S7 | Browserサムネ・メタ(色ブロック仮) | `Engine::media_frames`/`media_duration`(FFmpegはmotolii-media) | 素材メタ表示+1フレームデコードでサムネ | 画像デコーダ/プローブの自作・新規crate | 10分 |
| S8 | 再生の滑らかさ計測(オラクル) | `motolii-compositor::RenderTiming::total_us` | 再生中のフレーム時間をログ(release)。カクつき=不合格の器具 | 独自プロファイラ | 5分 |
| S9 | EFFECTS節(「No shared FX」固定) | `view.effects()`+compositor `effects/isf/`(`motolii.isf_bloom`配線済み)+旧`fx_stack.rs`の意味 | 選択層のeffect実データを節に出す | effect UIの再発明(旧fx_stackの語彙を写す) | 10分 |
| S10 | File menu → 書き出し | `motolii-export`(app/engine) | メニュー1項目→export呼び出しの疎通のみ | exportパイプの再設計 | 10分 |
| S11 | ~~Lottie .json読み込み~~ | **縫い目から除外(2026-08-29利用者裁定)**: importerは設計しない — Lottieは審判であって交換形式でない(decision-index「Lottie importer 設計しない」行)。在庫調査も0件でこれを裏付け | — | importer自作(恒久禁止) | — |
| S12 | ~~窓タイトル~~ | **済**(f9ca6e8c: `WindowAttributes::default().with_title`をlaunch_cfg configへ) | — | — | 済 |
| S13 | Stage毎paintのencoder警告 | `Compositor::render_into`(presentable.rs:118-120)のbegin_frame/submit契約 vs 毎paint呼び | 警告の根(re_renderer context.rs:480)を特定し、正しい呼び順 or フォーク側の口を1本 | 警告のlog抑制で誤魔化す | 15分(調査) |

## レーン共通の柵(発注書に毎回転記)

- ビルドは`motolii/`workspaceのみ・targetは共有1つ(motolii/AGENTS.md)。UI確認はdx serve --hotpatchのwarm窓
- 新しい型/traitを定義する瞬間に「既にやっている物」を名指せ。この表にない自作は停止して差し戻し
- 委託可否で迷ったらrerun-technical-delegation.tsvを引く(blend/mask/text/transform補間=自作継続が**確定済み**、逆方向の再委託調査も禁止)
- コメントは制約だけ。発注は落ちるテスト or 「出たか」検査を先に書く

## チュートリアル(通すべき手順書) — campaignの検収台本

2026-08-29利用者裁定: **このチュートリアルが通ることがゴール。** 動詞→パネルの写像は
「入れる・取り出す=Browser / 時間・調整=Timeline / 値の調節・変更=Inspector」。
全機能はユーザがどこかのパネルから到達できなければ存在しない扱い
(fixtureに直接生やしたテキスト層は到達不能=未完、の実例)。

> **mp4を入れ、その上に四角(パスシェイプ)と文字を作り、positionキーフレームで動かし、取り出す。**

| # | 操作 | 入口 | 縫い目 |
|---|---|---|---|
| 1 | mp4を入れる(D&D/読み込み→admit) | Browser(Media) | S14 |
| 2 | mp4を層にする(配置) | Browser→Timeline | S15 |
| 3 | 四角を作る | Browser(Create) | S16 |
| 4 | 文字を作る | Browser(Create) | S16 |
| 5 | 文字の内容を変更 | Inspector | S2+S4+S5 |
| 6 | playheadを掴む(スクラブ) | Timeline(ルーラー) | S17 |
| 7 | Positionにキーを打つ | Inspector(Key列◇) | S5 |
| 8 | 2点目のキー(時刻+値) | Timeline+Inspector | S5+S17 |
| 9 | 再生して確認 | Transport ▶ | 済(S1) |
| 10 | 取り出す | **Browser**(File menuでなく — 利用者裁定) | S10 |

### 追加の縫い目

| # | 皮の受け口 | 繋ぐ先の室(在庫) | 繋ぎの一行 | 自作禁止 | 粒 |
|---|---|---|---|---|---|
| S14 | Browser(Media)へのD&D/読み込み | store `AssetLedger::admit`+`AssetDraft::from_probed_source`(asset.rs:169)+Engine::media_duration(裁定276: frontはengine越し) | ファイル→admit→Media一覧 | probe/デコーダ自作・motolii-audio直依存 | 10分 |
| S15 | Browser→Timelineの配置gesture | `Intent::AddLayer`+`SetMeta{source: Media,…}`+`LayerTiming::place`(store lib.rs:381) | 素材→層1本 | 配置規則の発明(placeが既にある) | 10分 |
| S16 | Browser Createタブ(Text/Rectangle/Solid) | Text=fixtureレーンで実証済みのIntent列 / Shape=**在庫調査中**(view.shapes→engineのshape texture)。**2Dシェイプはパス前提**(2026-08-29利用者裁定: 四角=4頂点の閉パス。パラメトリック矩形を別型で持たない — Lottieに近い側を採る。`Value::Path`の頂点補間とも整合) | Createの行クリック→層が生まれ選択される | Shape描画の自作・矩形プリミティブ型の新設 | Text 10分/Shape 調査待ち |
| S17 | ~~ルーラースクラブ~~ **済**(Clock::seek) | probe `Clock`(seek口を1本足す)+timeline_widgetのPointerDown(ルーラー域) | クリック/ドラッグ=Clock.seek | 第二のclock・transport状態の複製 | 10分 |

## 責任分割(モジュール地図)

並列の本質は速度でなく**責任の分割**(2026-08-29利用者)。現状はmain.rs 1枚岩が全レーンの
衝突点なので、S1着地直後にsupervisorが機械分割する。以後の発注書は「あなたの家は◯◯.rs」
の1行で write-set が決まり、「触るな」の列挙が要らなくなる。

| ファイル | 責任 | 住む縫い目 |
|---|---|---|
| `main.rs` | 起動・launch_cfgだけ | S12 |
| `app.rs` | レイアウトgrid・スプリッタ・スケール操作 | — |
| `session.rs` | **共有状態の家**: `Arc<Mutex<Document>>`・Clock・UiScale・選択・**Intent送り口(単一書き手の柵はこの1枚)** | S2, S3の柵 |
| `fixture.rs` | load_fixture/UiData構築 | — |
| `browser.rs` | Browserパネル | S7 |
| `inspector.rs` | Inspectorパネル | S4, S5, S9 |
| `timeline_shell.rs` | transport帯・層列・M/S/L UI | S3のUI |
| `timeline_widget.rs` | canvas描画・zoom・ドラッグ | S6 |
| `stage_widget.rs` | Stage表示 | S1 |
| `tokens.rs` | 意匠トークン | — |

**交差点の専権**: `session.rs`と`tokens.rs`への書きはsupervisorのみ(レーンは読むだけ。
変えたい時はFINDINGで返す)。ビルドはtarget1つの柵により直列のまま — 編集は並列、
`cargo check`は順番待ち。ビルド並列化の道具は
[parallel-lane-build-mechanisms](2026-08-29-parallel-lane-build-mechanisms.md)を引く。

## 並列の割り方(write-set互いに素)

- 波1: S1(stage_widget) ∥ S3+S4(inspector/msl) ∥ S7(browser) ∥ S12(main.rsのconfig1行)
- 波2: S2+S5(選択と書き、S3の柵の上) ∥ S6(timeline) ∥ S8(器具) ∥ S9
- 波3: S10(S11は裁定により除外)
