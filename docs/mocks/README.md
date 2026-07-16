# M3 UIモック

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
