# Workspace — Motolii の窓の正本(2026-10-04)

Widgetbook の `Workspace › Workspace › Window`。lab の部品を 1 枚の窓に集めた、**今の正本**。
迷ったら、この窓とこの文書に合わせる。`Archive` の棚(`sets/studies/` と `parts/` の草案)は検討の記録で、正本ではない。

```
┌ Top bar ─────────────────────────────────────────────────────────────┐
├ Dock ─────┬ Stage ───────────────────────────────┬ Inspector ─────────┤
│ rail│shelf│ tools│ viewport (art + HUD + trial bar)│ InspHost(Pop)     │
├───────────┴──────────────────────────────────────┼ Desk ──────────────┤
│ Timeline                                          │ Tools/Ease/Depth  │
└───────────────────────────────────────────────────┴───────────────────┘
```

## 前提(取り違えやすいこと)
- **本物の Stage は Rust が描く**(wgpu / vello / 3DGS)。ここの Stage の絵(`stage_art.dart`)と効果の見た目(`stage_fx.dart`)は代用品。Flutter が担うのは、絵の上に重ねる操作の層(選択枠、ハンドル、定規、HUD、試し表示の帯)だけ。
- **Motolii 本体(`/Users/member_ottoto/rust_ae/Motolii/motolii`)は読むだけ**。手本にしたのは、本体の `ui/lib/panels/browser/`(Effects 棚、フィルター帯)と、`crates/motolii-render/src/compositor/effects/catalog.rs`(shader の静的解析で何がわかるか)。
- 窓の文字は英語。会話と文書は日本語。

## 開き方・確かめ方
```bash
cd /Users/member_ottoto/rust_ae/design-sense-lab/book
F=/Users/member_ottoto/Documents/Motolii-probes/toolchains/flutter/bin
$F/flutter run -d web-server --web-hostname 127.0.0.1 --web-port 8732   # 最初の 1 回は落ちやすい。落ちたらもう一度
# → http://127.0.0.1:8732/#/?path=workspace/workspace/window   (起動時の初期ページ)
$F/flutter analyze lib test
$F/flutter test                                  # Workspace は test/workspace_*_test.dart
$F/dart format -l 160 <自分が触ったファイル>        # glob で全体にかけない
```
- web のデバッグ版は、最初の表示に 20〜40 秒かかる(白いままでも待つ)。
- 実寸の静止画は、テストの中で `RepaintBoundary.toImage(pixelRatio: 2)` で撮る。文字は `FontLoader` で SFNS を `Inter` / `.AppleSystemUIFont`、SFNSMono を `Menlo` として読み込む(`tester.runAsync` の中で)。撮影用のテストは使い終わったら消す。

## ファイルの地図(1 ファイル = 1 つの責任)
| ファイル | 責任 |
|---|---|
| `ws.dart` | **共有の状態 `Ws`** と**窓のトークン `WsT`**。全部の席がこれだけを読む |
| `workspace.dart` | 窓の組み立て(5 枚のペインと初期配置 `_preset`)、縮小(最小 1280×800 を切ったら 1600×960 を縮める)、Enter/Esc、Inspector の席(Stage の値を Inspector に渡す) |
| `panes.dart` | ペインの入れ物 `WsPanes`。pub の `docking` に載せる(タブのドラッグで並べ替え・分割、仕切りでサイズ変更)。タブの記号と見た目(`WsT` から)、棚のタブと `Ws.shelf` の行き来 |
| `top_bar.dart` | ロゴ、プロジェクト名、モード(Edit/Animate/Export)、Render |
| `dock.dart` | 棚 1 枚 = ペイン 1 枚の `WsShelf`(Media / Effects / Fonts / Colors / Create)と棚の一覧 `dockShelves`。見出し、検索欄、Effects のときだけフィルターのボタン |
| `dock_parts.dart` | 棚の共通部品と、棚ごとの記憶 `DockMem`(選択、畳み、星、フィルター、帯の高さ) |
| `dock_effects.dart` | Effects 棚(文字だけの一覧、試し表示、Add) |
| `dock_filters.dart` | フィルター帯 `DockFilterBand`、ボタン、高さを変える `DockResizable` |
| `fx_catalog.dart` | 効果のカタログ `FxDef`(解析で読む値 + 作者が書く `@tags` / `@preview`)。Dock と Stage が読む |
| `dock_files.dart` | Media の棚 = ファイルブラウザー(AEViewer 2 型)。場所とフォルダの木 + 開いているフォルダの中身(サムネイル / リスト)。広いと木が左、狭いと木が上(高さはドラッグ) |
| `dock_media.dart` | 素材の目録 `mediaAssets` とサムネイルの絵(`MediaThumb` / `PopArt`) |
| `dock_fonts.dart` / `dock_make.dart` | Fonts / Colors・Create の棚 |
| `dock_glyphs.dart` | 記号(棚、ペインのタブ、効果の系統、ファイルの種類) |
| `stage.dart` | Stage の席(ヘッダー、道具の列、オプション列、試し表示の帯の置き場) |
| `stage_view.dart` | ビューポート(定規、ガイド、選択、HUD、ナビゲーター) |
| `stage_art.dart` | 絵(`ArtScene` = 1 フレームの純関数)と、当たり判定・箱 `WsArt.boxOf` |
| `stage_fx.dart` | 効果の代用の見た目(系統ごとに 1 つ) |
| `stage_trial.dart` | 試し表示の帯(TRYING … / Cancel Esc / Add Enter) |
| `timeline.dart` / `timeline_paint.dart` | Timeline の席と描画 |
| `desk.dart` / `desk_ease.dart` / `desk_depth.dart` / `desk_kit.dart` | Desk(選択に追従: キー → Ease、カメラ → Depth、他 → Tools) |

Inspector は `../sets/inspector/`(`inspector_parts.dart` が入口、`InspHost` と Pop のトークン)。book 全体の地図は [`../../README.md`](../../README.md)。

## 共有の状態 `Ws`(ws.dart)
席どうしは直接話さない。**`Ws` を読み、`Ws` に書く**。`WsScope.of(context)` で購読、`WsScope.read(context)` で読むだけ。

| 持つもの | API |
|---|---|
| レイヤー(仮の作品 CODA、8 枚) | `layers`, `fps = 30`, `duration = 240` |
| 選択 | `selected`, `layer`, `select(id)` |
| キー | `keys`, `pickKey(k, add:)`, `clearKeys()` |
| 時刻と再生 | `frame`(初期 112), `playing` |
| Dock の棚 | `shelf`(`Media` / `Effects` / `Fonts` / `Colors` / `Create`) |
| Desk | `desk`(追従の結果), `openDesk(name)`, `deskManual` |
| よそのパネルへの誘導 | `route('Browser › Colors' / 'Browser › Fonts' / 'Ease')` |
| 効果 | `fxOf(id)`(確定済み), `trial`(試し中), `canTry`, `tryFx(name)`, `keepFx()`, `dropFx()` |

規則: 操作 1 回につき通知 1 回。**何もしていない間は何も回さない**(再生中だけ進む)。

## トンマナ(Pop)
- 値は `WsT`(窓)と `Pop`(Inspector、`sets/inspector/inspector_parts.dart`)と `Grey` / `T`(`tokens.dart`)から取る。**数値を直書きしない**。
- 箱(席・カード)は四角く、隙間なく並べる(間は `WsT.gutter` の暗い線)。触る物と読む物(欄・チップ・ボタン)は丸い。
- 色は平らで不透明。アクセントはライム 1 色(`#B3C66B`、上の文字は `#15170D`)。再生ヘッドもアクセント。
- レイヤーの種類ごとに色を 1 つ(`wsTone`)。Timeline のバー、Dock、Stage で同じ色を使う。

## カラーテーマ(Dark / Light)
- 上のバーの半円ボタン(key `ws-shade`)で切り替える。`Grey.shade`(`ValueNotifier<Shade>`)を反転して `reassembleApplication()` で窓全体を建て直す。State は残るので編集中の内容は消えない。
- **Workspace と Inspector(`sets/inspector/`)では灰色を `Grey.g07` などで取る。`N.g07` は使わない。** `N` は古い研究ページ用の暗い段で、テーマに追従しない(そのページは Dark のみ)。
- `Grey` は `N` と同じ名前の段を持つ。役割(地・席・カード・欄・線・控えめな文字・文字)は両テーマで同じで、明るさだけが逆になる。Light ではカードが一番明るく、文字が一番暗い。
- テーマで変わる色は `const` にできない。`static const` / トップレベルの `final` / 引数の既定値に `Grey` や `WsT.body` などを置かず、getter か `Color?` + `?? Grey.xx` にする(`final` に入れると最初の色のまま固まる)。
- アクセントは塗りなら `WsT.accent` / `Pop.accent`。**線や文字として地の上に描くときは `accentInk`**(Light ではライムが白に溶けるので濃いオリーブ)。再生ヘッドの線は `accentInk`、頭の札は `accent`。
- 種類の色(`tone*`)と Stage の絵(`ArtInk`)はテーマで変えない。
- `T.name()` などの文字色の既定は `Grey` から取る。暗すぎる・薄すぎる文字の補正(`T._floor`)もテーマごとに向きが逆。

## 決まっていること(利用者の裁定)
- **Media と Files は一つの棚(Media)。** AEViewer 2 型: 左(狭い時は上)に場所(Project / Home / Desktop / Downloads)とフォルダだけの木、右に開いているフォルダの中身をサムネイルかリストで。フォルダのタイルを押すとそのフォルダを開く。パンくずの各段も押せる。検索は今いる場所の中を全部探し、結果に入っているフォルダ名を添える。サムネイルの格子は捨てない。
- **窓はドックパネル。** Motolii 本体と同じ pub の `docking`(本体の `ui/lib/workspace/dock_workspace.dart` を lab 用に書き直したもの)。ブラウザも棚ごとにペインに分けた(rail は無い)。**タブは記号で示す**: 棚のタブは記号だけ(中の見出しが名前を言う、5 枚並んでも細い)、Stage / Inspector / Timeline / Desk は記号 + 名前(席の中の見出しに同じ名前を重ねない)。前に出ている棚が `Ws.shelf`(タブを押すと変わり、`Ws.route` で棚のタブが前に出る)。別窓に出す Detach は native の窓が要るので lab には無い。ペインは閉じられない(開き直す手段がまだ無い)。
- **初期配置**: 棚のタブの列・Inspector・Timeline・Desk は基準の px で始め、Stage が残りを取る。`multi_split_view` は size を weight に直して足し合わせるので、残りを取るペインには weight を付けない。
- **ペインは広げたら得をする。** 列を固定して引き伸ばさない。
  - 格子は `wsCols(幅, セルの幅)` で列数を決め、`wsCell` でセルの幅を割り出す(Create のタイル、Colors の色見本、Desk の道具)。
  - 一覧の区切り(Create の群、Colors の Used / Palettes / Gradients / Swatches、Effects の族)は `DockMasonry` のカードにする。1 列は 260〜300px 以上、最大 4 列、短い列から詰める。
  - Fonts は 440px 以上で見本の大きいカード、狭いと行。Media は 440px 以上で木が左、狭いと上。
  - Effects はフィルター帯を開いていて 560px 以上なら、帯を一覧の横に縦いっぱいで立てる。狭いと上の帯。
  - **Inspector は縦積みのまま**(カードを横に並べない)。カードの中身を、Effects のパラメーターと同じブロックの格子(`Pop.tileMin` ごとに列が増える)にする。Layer の Opacity / Depth / Parent は `LayerTiles`。
- **Inspector は一目でわかること。** フォント・色の一覧・イージングは、それぞれ専用のパネル(Browser › Fonts、Browser › Colors、Ease の desk)が持つ。Inspector は今の値を見せて、そこへ誘導するだけ(`PanelLink` / `LinkLine` → `Ws.route`)。
- **Inspector の値は Stage と一致させる。** 位置・倍率・回転・塗りは `WsArt.boxOf` と絵の色から渡す(`InspHost(values:, strs:)`)。
- **パスの操作は同じものを何度でも重ねられる**(パンク膨張の後にまたパンク膨張)。UI 側で数を絞らない。
- **シードなど普段触らない値も、隠さずに Advanced に置く**(`AdvancedFold`)。
- **テキストは普通に**: 入力欄、その下にサイズ・太さ・色。
- **Effects の一覧にサムネイルは付けない。** 見た目は Stage で確かめる: 行を選ぶ → 選択中のレイヤーに仮で掛かる → Add / Enter で確定、Cancel / Esc で取り消し。
- **効果のタグは 2 種類。** 振る舞い(Seat / Applies to / Time / Inputs / Parameters / Origin)は shader の静的解析が出す。何に見えるか(Look)だけを作者が `@tags(...)` で書く。試し表示だけで伝わらないこと(下の合成を読む、1 フレーム目に何も出ない、など)は、作者の `@preview(...)` か解析結果から帯に一言出す(`FxDef.trialNote`)。
- **フィルター帯は Motolii 本体の草案に合わせる**: グループ同士は AND、グループ内は OR。クリックで 1 つ、Cmd/Ctrl-クリックで追加・解除。残る件数を札に出し、0 件になる札は押せない。件数と Clear は帯の一番下。**帯の高さは利用者が決める**(下端のグリップを上下にドラッグ、ダブルクリックで 168px に戻る)。

## 変えるときの作法
- 席をまたぐ情報は `Ws` に足す。席どうしで import し合わない。例外は二つだけ: Dock と Stage が共有するデータの `fx_catalog.dart`、Inspector に Stage の値を渡すために `stage_art.dart` を読む組み立て役の `workspace.dart`。
- 新しい振る舞いには `test/workspace_*_test.dart` を足す。窓全体の通しは `workspace_window_test.dart`。
- コメントは、コードから読めない制約だけを書く(`contract:` で始まる行がそれ)。
- `lib/sets/inspector_gui_set.dart` は利用者の作業中のファイル。触らず、コミットにも含めない。
- 区切りごとにコミットする(履歴が無いのが一番怖い、が利用者の方針)。

## まだ無いもの(候補。入れるかどうかは利用者が決める)
- 確定した効果を Inspector の効果スタックに名前で出す(今の `InspHost` は効果を件数でしか受け取れない)。
- `Ws` に足す候補: ループと作業範囲、レイヤーごとの表示・ロック・ソロ、マーカー、Ease とレンズの値、レイヤーの奥行き、Blend への誘導、コンポのサイズの定数。
- 本体とはつながっていない(FFI も実データも無い)。
- テーマの選択は保存されない(起動すると Dark)。Widgetbook の他のページには Light が無い。
