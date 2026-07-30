# CU-108 product connection human acceptance observation

状態: **観察**

## 観察

2026-07-30、固定Macの通常製品windowを人間確認した結果、Browser→typed intent→Host→Place→
Stage / Timeline / Inspector→Undoの配送成立とは別に、次の受け入れ不具合が確認された。

- Browser WebViewの外周に白い余白があり、web標準scrollbarが製品chromeへ露出する。
- HTML5 D&DのDOM drag imageが表示され、配置されるRectangleの形と位置を直接確認できない。
- Stage表示がぼやけ、drag中の座標と配置結果の一致を判断しにくい。
- Timelineはpublished snapshotを投影しているが、以前のnative比較面が持つheader / ruler /
  row rail / key / playheadの情報密度を失い、click結果も次のUI eventまで見えない。
- React panelの初期表示より先に空のnative windowを表示していた。

## コード事実

- `browser-host-screen.css`には`html/body/root`のmargin / overflow resetがなかった。
- Browserの`ElementCard`は標準drag imageを抑制していなかった。
- `static_preview.rs`のcomposition descriptorは`64x36`で、`Quality::DRAFT`後は`32x18`だった。
- AppKit local monitorはmouse-upをinboxへ積むがevent loopを起こさず、clickの排出が次eventへ
  遅延し得た。
- WebKit tracking中のpointer pollは実mouse-upより前の最後のbutton-down位置を保持し、release時に
  その途中点を優先していたため、速いdragほどdrop結果がカーソル後方へずれた。
- 数値ログ導入後、global pressed-button stateがupになってからAppKit local monitorへ実
  `LeftMouseUp`が届くまで約数msの競合を観測した。即時fallbackでは終点に1.6〜3.5 logical pxの
  残差が生じた。
- winitのAppKit content viewがflipped（top-down）である場合にもHostが`height - y`を適用し、
  Stage NDC変換の前にYを二重反転していた。
- Timeline product passは背景とbarだけを描き、既存headless projectionのkeyとprimary selectionを
  表示へ使っていなかった。

## この変更で閉じる範囲

- product Hostだけのviewport reset、scrollbar非表示、native drag imageへの置換。
- composition descriptorを`1920x1080`へ戻し、Draft表示を`960x540`にする。
- Retina logical point→Stage NDCの回帰試験と、canonical `0.2x0.2` Rectangleのnative transient
  overlay。
- Timelineのnative header / rail / ruler / row / bar / key / playheadとprimary selection表示、
  AppKit click wakeの即時配送。
- AppKit local monitorの実`LeftMouseUp` content logical pointをPlace terminalの終端座標にする。
- global button-upだけを観測してもcommitせず、実`LeftMouseUp`を待つ。WebKit trackingにより
  50msを超えて遅れる実測があるため時間fallbackは置かない。待機中はDocument workを発行せず、
  focus loss / Escだけをcancelとして受ける。
- `NSView::isFlipped()`に従ってAppKit pointをtop-down logical pointへ一度だけ正規化する。
- debug製品UIではraw AppKit point / `isFlipped` / logical point / Stage rect / NDC /
  canonical Place中心を同一generationとlayout epochで構造化数値ログへ出す。
- 同じ系列へstartup phase時間、window / DPI / layout、Browser WebView load / IPC / focus、
  Stage render generationと解像度、Document revision、Timeline projection / hit、Inspector
  publish、history、surface recovery / failureを追加し、UI背骨の全境界を実機確認時に読む。
- hidden window中にReact navigationを開始してからwindowを表示する初期化順。

## 非目標・停止線

- Document、journal、plugin契約、公開API、永続形式を変更しない。
- ReactにDocument / selection / Undo正本を追加しない。
- archived mock、Timeline比較spike、fixture stateを製品runtimeへimportしない。
- text renderer、semantic zoom、playhead永続ownerをこの観察から発明しない。
- visual goldenまたはthresholdを変更して合格させない。

## 再現可能な審判

```bash
npm --prefix ui/motolii-web run build:host
npm --prefix ui/motolii-web run check:host
node --test \
  ui/motolii-web/guard-tests/browser-host-codec.test.mjs \
  ui/motolii-web/guard-tests/browser-ownership.test.mjs
cargo test -p motolii-ui
```

座標診断ログはdebug buildで常時有効とし、release buildでは
`MOTOLII_UI_TRACE=1`を設定したdevelopment診断時だけ有効にする。旧座標診断との互換用に
`MOTOLII_UI_NUMERIC_TRACE=1`も受ける。prefixは`[motolii-ui-trace]`で、Document /
journal / 公開APIへ保存しない。

通常の実機確認は次で起動し、生ログを`target/ui-traces/`へ保存してから読む。

```bash
./scripts/run-ui-trace.sh /path/to/project.json
```

固定Mac実機では通常製品windowで、Browser外周に白余白とweb scrollbarがないこと、drag中に
DOM cardではなく配置RectangleがStage上を追従すること、drop中心が追従中心と一致すること、
Timeline clickが追加eventなしでprimary selectionとInspectorへ反映されることを確認する。
