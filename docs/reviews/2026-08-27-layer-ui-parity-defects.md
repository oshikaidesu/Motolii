# レイヤーUI の当たり前が欠けている箇所(2026-08-27)

判定語: **採用**(発注中)。2026-08-27 の並列監査5レーンが `app/motolii`(当時
`next/probes/r7-makepad-panel`)から挙げた27件のうち、**AE なら30年前からできること**に
絞った台帳。

## なぜこれが本題か

Motolii の賭けは「**レイヤーは UI であって存在論ではない**」(裁定270 の文脈)。
AE = レイヤーUI + 平面画素モデル、Nuke/Blender = 強いモデル + 高い UI に対して、
Motolii は**勝った方の UI を取り、天井の無いモデル(Rerun の空間合成・Lottie の
ベクタ)を下に置く**。

この賭けは**レイヤーUI が実際に強い場合にしか成立しない**。以下はその強さの中身が
まだ無い箇所であり、AE との距離を詰めるための最低条件である。「シンプルさはプロ仕様と
同じ線」は、プロ仕様の線に届いて初めて美点になる。届いていないシンプルさは不足である。

## 分類

- **虚報** — 操作できると表示しながら、操作が存在しない。信頼を直接壊すので最優先
- **欠落** — 普通なら必ずある操作・反応が無い
- **幾何充填** — 利用者の持ち物であるべき値がレイアウトの従属変数(またはその逆)
- **状態設計** — 単一の状態から導出されていない

## レーン A/B: Timeline (`app/motolii/src/timeline_surface.rs`)

| # | 類 | 欠陥 | 実測 |
|---|---|---|---|
| A1 | 欠落 | **クリップにトリムハンドルが無い**。`TimelineGesture` は `None`/`Playhead`/`Lane` の3状態のみで、`draw_lane_clip` は棒を描くだけ。clip の `start`/`duration` を書き換える経路がコード全体に存在しない | `:214` `:966` |
| A2 | 虚報 | **クリップ上で常に `EwResize` カーソル**が出る。`time_rect()` がルーラーだけでなく本体領域を含むため。伸縮機構は A1 のとおり存在しない | `:747` |
| A3 | 欠落 | **無修飾の縦スクロールが常に `0.0`**。コメントは「垂直入力はレーンスクロール用に予約」と書くが、対応する処理はファイル内に無い | `:331` |
| A4 | 欠落 | **レーン並べ替えで掴んだ行が動かない**。色が `playhead_color` に変わるのと、挿入先の2px線だけ。掴んだ物が静止したままドラッグする | `:857` `:1306` |
| A5 | 欠落 | **クリック選択の経路が無い**。`TimelineLane.selected` は描画側が読んで色を変えるが、`FingerDown` 分岐は M/S/L グリフ・ルーラー・レール開始の3系統だけで、選択を切り替える action が無い | `:1242` |
| B1 | 幾何充填 | **`fitted_lane_height`** — 行高 = ペイン高 ÷ レーン数、かつ `type_ratio: 0.53` で文字が行高に比例。**ペインのリサイズがタイムライン全体の画像ズームになる**。553行に "There is intentionally no vertical scale." とある | `:50` `:606` |
| B2 | 欠落 | **プロパティ(キーフレーム)行を折りたためない**。`visual_rows()` が無条件に全展開し、開閉状態も三角も無い | `:619` `:996` |

**B1 は A3 と対**である。普通のソフトは「行高は固定(利用者が選ぶ)+ 入り切らなければ
縦スクロール」。B1 を直すと A3 が必要になり、A3 だけでは B1 が邪魔をする。

**注意**: `lane_height_fits_all_lanes_and_keeps_property_height_fixed` という**テストが
B1 を仕様として固定していた**(2026-08-27 に削除済み)。緑の柵が逆を向いていた例。

## レーン C: Browser (`app/motolii/src/browser_surface.rs`)

| # | 類 | 欠陥 | 実測 |
|---|---|---|---|
| C1 | 虚報 | **検索欄が `InkLabel`**。プレースホルダ文字列を貼った「入力欄に見える面」で、キー入力を受け付ける構造が無い | `:214` |
| C2 | 幾何充填 | **カードグリッドが手書き2列**。`col_a` に奇数・`col_b` に偶数をコードで振り分けている。件数が変わっても再配分されず、幅を変えても列数が再計算されず、スクロールコンテナも無い | `:270` |
| C3 | 状態設計 | **rail 選択・`kind_filter`/`tag_filter`/`clear_filters` がカタログと配線されていない**。カタログが8枚直書きなので、フィルタ UI は構造上一覧へ反映され得ない | `:230` |

## レーン D: Inspector (`app/motolii/src/inspector_surface.rs`)

| # | 類 | 欠陥 | 実測 |
|---|---|---|---|
| D1 | 虚報 | **数値 `vx`/`vy`/`vz` が全て `InkLabel`** で、ドラッグ/クリック検出がどこにも無い。にもかかわらずフッターが `"drag to scrub · click to type · Esc to cancel"` と操作方法を明示している。**Q0「触れそうで触れない物は不合格」への直撃** | `:41` `:98` |
| D2 | 虚報 | **折りたたみ三角 `▼`/`▶` と FX の `ON` が無反応**。どちらも `InkLabel`/`SolidView` で `on_click` も開閉状態も無い | `:25` `:88` |

## レーン E: Chrome parts (`app/motolii/src/chrome/parts/`)

| # | 類 | 欠陥 | 実測 |
|---|---|---|---|
| E1 | 虚報 | **stepper の値表示が `Label`**。`+`/`−` ボタンでしか変えられず、クリックでタイプもドラッグスクラブも構造的に不可能。色が `#xc9c9c9` なので編集可能に見える | `stepper.rs:57` |
| E2 | 欠落 | **タイムコードの半分だけ編集可能**。`frames` は `TextInputFlat`、`seconds` は `ChromeInk`(表示専用)。普通は1個のアトミックな欄 | `parts/transport.rs:188` |
| E3 | 欠落 | **ツリー行の hover 反応がゼロ**。コメントが「hover の shader merge と animator は eval を落とした(frozen vec / self 不在の実測)ため置かない」と自己申告。**単純に戻すと窓が落ちる可能性がある — 実窓で確認しながら進めること** | `fold.rs:69` |
| E4 | 虚報 | **ロックボタンが押下中しか青くならない**。`color_down` はマウスを押している間だけの状態で、離すと見た目が解除に戻る。永続 active が無い | `toggle.rs:83` |
| E5 | 虚報 | **色見本が `SolidView`**。クリックの受け口が無く hover 色も無い。ピッカーを開けない | `color.rs:18` |

## レーン F: Shell / Stage (`app/motolii/src/main.rs`, `stage_chrome.rs`, `export_surface.rs`)

| # | 類 | 欠陥 | 実測 |
|---|---|---|---|
| F1 | 虚報 | **パネル切替/設定ボタンが `ui.status.set_text()` を呼ぶだけ**。アイコンは panels.svg / filter.svg で、パネル開閉・設定遷移に見えるが何も起きない。`SettingsPane` は別途実在するのに導線が無い | `main.rs:208` |
| F2 | 自作機構 | **テキスト入力中でも Space が再生になる**。`handle_key_down` が `ui.handle_event` より前に無条件で横取りしており、フォーカスガードが無い | `main.rs:1096` |
| F3 | 状態設計 | **ズーム % が2箇所に独立保持**。`stage_band.stage_mode` はタブ切替時に `"USER VIEW · 62%"` をハードコードで上書きし、`zoom_well.zoom` は `home_zoom` でのみ `"100%"` になる。Home を押しても片方が古いまま | `stage_chrome.rs:158` `:169` `:229` |
| F4 | 幾何充填 | **進捗バーの塗りが固定 120px**。`width: Fill` のトラック内で定数(ダミーと自己申告)。配線時に踏襲されると問題化する | `export_surface.rs:72` |

## 検収の作法

**テストを書かない。** 2026-08-27 の裁定270 のとおり、バック/フロントの段差は構造で
消えており、**見える物は窓で見る**。

```bash
# 窓は1回だけ起動(既に走っているならこれは不要)
cargo run --locked --manifest-path app/Cargo.toml -p motolii -- --hot --remote > /tmp/motolii.log 2>&1 &
P=$(grep -o 'listening on 127.0.0.1:[0-9]*' /tmp/motolii.log | grep -o '[0-9]*$')

curl -s "http://127.0.0.1:$P/snap?q=<id>"          # 対象の矩形を探す
curl -s "http://127.0.0.1:$P/click?x=..&y=..&wait=1"
curl -s "http://127.0.0.1:$P/k?t=hello"            # 文字を打つ
curl -s "http://127.0.0.1:$P/g"                    # 絵を撮って目で見る
curl -s "http://127.0.0.1:$P/log?n=30"             # eval エラーが出ていないか
```

**各欠陥の合否は「窓を叩いて、期待した事が起きたか」**。実装しただけで通ったことに
しない。落ちたら `BLOCKED` と報告する。

## 参照

- [注意の失敗と、世界の分断](2026-08-27-attention-failures-and-the-partition.md) — 裁定270
- [Timeline 普通の UX 台帳](2026-08-26-timeline-ordinary-ux-ledger.md) — W3C/WCAG/AE 由来の契約30件
- 運転: [AGENTS.md](../../AGENTS.md)「UI修正はホットリロード運転」「製品は `app/` に居る」
