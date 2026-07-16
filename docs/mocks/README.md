# M3 UIモック

## 基準: 高密度メインUI v1

[インタラクティブHTML](m3-main-ui-v1.html)をM3の視覚構成基盤とする。設定画面からのライト/ダーク切替、preview canvas、波形、driver scopeをブラウザ内で実際に描画する。

このHTMLは2026-07-11にClaude Desktopが一時scratchpadへ生成したモックを、2026-07-14に回収し、テーマ設定要件・role名token・Dark既定・contrast修正を反映したもの。静止画はHTML改訂のたびにheadless Chrome(`#dark` / `#light` hashで固定入場)から再生成し、HTMLと乖離させない。

![M3 main UI dark](m3-main-ui-v1-dark.png)

- [ライト静止画](m3-main-ui-v1-light.png)
- [ダーク静止画](m3-main-ui-v1-dark.png)
- ステータス: **視覚構成の基準モック**。Document意味論や未決機能を確定するものではない
- 色: Ableton Liveを先例にしたflat surface、細罫線、色=機能。装飾gradientなし
- 密度: asset browser、preview、property、effect stack、driver、波形、階層timeline、easing popupを同一画面へ常設
- 説明: 画面内badgeと下部対応表で、確定事項・方向確定・未決の出所を分離

## 比較案: グリッド基調 v2

[m3-main-ui-v2.html](m3-main-ui-v2.html)は、v1と同一fixture・同一token roleのまま「余白は分離の手段にしない」規約(2026-07-14追記)で組み直した比較案。G0-6の構成審判はv1とv2を同じviewport・同じ情報量で比較して行う。

![M3 main UI v2 dark](m3-main-ui-v2-dark.png)

- [v2ダーク静止画](m3-main-ui-v2-dark.png) / [v2ライト静止画](m3-main-ui-v2-light.png)
- 全spacing/行高/radiusを`--sp-*`/`--row-*`/`--radius` tokenから取得し、raw値の場当たり指定を排除(色と同じ機械検査に載る)
- 領域分離は罫線+明度のみ。カード・影・空白分離を廃止(asset browserはカード型→22px行型)
- timeline 10行常視。text / image / meshの項目種別色を追加し、item roleの全種を1画面へ収載
- 下端にBlender式文脈ヘルプのstatus bar(hover対象・操作・shortcut・実行時stat)を常設

## 基盤として固定するもの

- 3-pane + 高密度timelineの大区画
- 波形とBPM gridを含むtimeline overview
- property / effect stack / driverを一覧できる右panel
- 選択、keyframe、data mapping、bakeを別の意味色で示すこと
- context説明を右下/status領域へ追加できる構造。Blenderは文脈ヘルプだけの参考で、全体UIは模倣しない
- ライト/ダーク/custom themeとも同じsemantic token schemaを参照すること
- 設定画面で組み込みDark/Lightを選択でき、初回既定はDarkであること(土台dark neutralの規約)
- tokenはrole名のみとし、文字用途の意味色はcontrast 4.5:1以上を保つこと(具体hex値は固定しない)

## 固定しないもの

- HTML内の具体色値、panel寸法、icon、font(現在のglyphはemoji/文字のplaceholderで、icon仕様の先取りではない)
- 既知のhue近接(solo黄とwarning琥珀、domain-path緑とstate-active緑、domain-pixel紫とitem-mesh紫、accentと選択青)。roleは分離済みで、hueの再配置はG0-6のCVD測定で決める
- 組み込み2テーマ以外の配布テーマ内容。custom themeを追加できる契約だけを固定する
- 未決と表示された音楽同期emission等の機能意味論
- plugin custom UI、3D gizmo、任意track色の永続化
- HTML/CSS/Canvasという実装方式。製品UIはM3仕様どおりSlint + wgpuを使う

## 次の改訂

1. ~~右下/status領域へ短いcontext説明を追加する~~ → v2のstatus barで収載済み（Blenderはこの機能だけの参考）
2. ~~timelineをさらに高くした比較案を同じfixtureで作る~~ → v2で作成済み。G0-6でv1/v2を同一viewport比較する
3. light/dark、grayscale、CVD、125/150/200% scaleで所在認知を比較する
4. hover/focus/drag/trim/easingを操作できるprototypeへ進める

---

# 過去・比較モックの台帳

## timeline v0

![M3 timeline visual mock](m3-timeline-v0.png)

- [SVG source](m3-timeline-v0.svg)
- ステータス: **比較用の静的モック**。G0-6のtoken決定、製品UI、goldenではない
- 対象: asset browser、preview、inspector、一般的なtrack型timelineを同一画面へ置いた時の密度と視覚認知
- 表現済み: video/audio/shape/text/group、選択、keyframe、mute、warning、playhead
- 未表現: hover/focus、drag/trim、zoom、easing popup、keyboard操作、IME、reduce motion、別monitor/DPI

色値はこのSVG内だけの仮値であり、DTCG tokenへ転記しない。採択するのは値ではなく、同じfixtureで比較して合格した役割と階層だけとする。

## このモックで答える問い

1. labelを読まず、項目種別・選択項目・mute・keyframe・warningを識別できるか
2. timelineが主作業面として十分な高さと一覧性を持つか
3. preview/inspector/asset browserがtimelineより強く見えすぎないか
4. 意味色が多すぎず、AE型の文字依存へも戻っていないか
5. 新規componentだけが別製品のように浮いていないか

## interaction v0(状態step送り)

- [HTML mock](m3-interaction-v0.html)(ローカルでブラウザ表示。例: `python3 -m http.server --directory docs/mocks`)
- ステータス: **比較用の状態モック**。token決定、製品UI、goldenではない
- 対象: timeline v0が未表現とした操作状態を、autoplayアニメーションではなく**scene切替(motion 0)**で表現する。各sceneはそのまま静止fixtureとしてG0-6のgrayscale/CVD/5秒審判に転用できる
- scene構成: 平常時 / hover・Info / keyboard focus / drag・trim中HUD / Relative drag(HUD+motion path ghost) / easing popup(区間選択→popup) / 接続valid(カーソル追従説明+仮線+輪郭◇) / 接続invalid(型付き拒否理由) / disabled+段階診断(Brief→Context)
- 未表現: zoom、IME、DPI/別monitor、Stage側の全scene、Effect共有のconnection gutter(G0-6画面5)

アニメーションを主表現にしない理由: 視覚言語は「motionを0にしても状態変化が判別できること」を合格条件とするため、動きが説明の主役になるモックは審判手段として使えない。順序が意味を持つ操作(Discover→Target→Preview→Commit)はstep送りで表現する。

## 次の比較案

- v0-A: 現在案。previewとtimelineをほぼ同格
- v0-B: timelineをさらに高くし、previewを縮小
- v0-C: inspectorを必要時だけ展開し、timelineの横幅を増やす

同じfixtureとviewportでA/B/Cを比較し、印象だけでpanel比率を決めない。

## UI dynamics v1（力学検証）

- [HTML mock](m3-ui-dynamics-v1.html)（ローカルでブラウザ表示。例: `python3 -m http.server --directory docs/mocks`）
- ステータス: **UI力学の比較用モック**。既存のinteraction v0を改版したものではなく、2026-07-16時点のUI操作言語から別に構成した。token決定、製品UI、goldenではない
- 対象: 選択のStage/Timeline/Inspector同期、説明付き接続、Relative Move、Camera/Handの所有差、共有Effectの常時接続線、Brief/Context/Inspectの段階診断
- 共通状態: `Discover → Target → Preview → Commit / Cancel → Inspect → Undo`
- 操作: 上部で力学を選び、「次へ」で状態を送る。「自動」は状態間の連続性を見る補助であり、各状態は停止して単独でも読める。`Motion 0`で動きを無効化できる

### このモックで答える問い

1. 操作の途中で、対象・期待型・確定結果・Cancel時の不変条件を同じ画面から説明できるか
2. Commit後も、選択、semantic badge、connection gutter、Inspectorに因果が残るか
3. Camera操作とHand/Stage View操作のうち、どちらがDocumentとUndoを変えるかを動き以外でも識別できるか
4. Relative Moveが通常dragと異なり、現在値ではなくmotion path全体へ作用することをHUDとghostから識別できるか
5. invalid/disabledを色やdimだけで終えず、expected/actualと次の一手へ段階的に到達できるか
6. motionを0にしても上記が成立し、自動再生を止めた任意の状態をreference fixtureとして比較できるか

このモックのanimationは装飾ではない。対象・preview・commit結果の空間的連続性を観察するためにだけ使い、意味の唯一の手掛かりにはしない。
