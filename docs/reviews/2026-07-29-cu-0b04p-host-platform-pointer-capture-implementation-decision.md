# CU-0B04P Host platform pointer capture実装決定

- 日付: 2026-07-29
- 状態: **実装完了 / DONE**
- 前提: `CU-0B04S`

## 1. 完了した境界

product Browserのtyped startを受けたHostが、WebKit tracking loopの外からもprimary
buttonとpointer位置を観測できるprivate capture seamを実装した。

- toolkit非依存state machineはgeneration、active、押下中の最終位置だけをTransientに
  保持し、`Moved` / `Released` / focus-loss `Cancelled`候補だけを返す。
- macOS adapterは既存windowの`NSWindow`をretainし、AppKitのglobal
  `NSEvent::mouseLocation`をscreen→window→contentへ変換する。
  `mouseLocationOutsideOfEventStream`はWebKit tracking中に古くなり得るため使わない。
- `NSEvent::pressedMouseButtons`のprimary bitだけを読み、button-upを観測したgenerationは
  再開しない。release候補はrelease後の位置でなく、押下中に得た最終位置へラッチする。
- active中はHost repaintを継続し、release / cancel候補を得た時点で本粒のactive intentを
  終える。candidateをStage admission、canonical変換、D2へまだ渡さない。
- `objc2` / `objc2-app-kit`はmacOS target dependencyに限定し、すでにwry / eframeが使う
  `0.6.4` / `0.3.2`系列へ一致させた。OS型をpublic APIへ出さない。

## 2. 負例

二重armは拒否し、idle sample、release後sample、focus-loss後sampleは新しいcandidateを
生まない。tracking loopがrelease前sampleを一度もHostへ返さない場合でも、typed startで
arm済みの最初のbutton-up sampleをrelease候補として失わない。

React terminal / coordinate、HTML5 DataTransfer終端、egui release、default center、
Stage hit-test、Document、D2、Undo、public raw input APIは追加していない。

## 3. 検証

```text
cargo clippy -p motolii-ui --lib --bins -- -D warnings
passed

cargo test -p motolii-ui host_pointer_capture --lib
4 passed

cargo tree -p motolii-ui -i objc2@0.6.4
cargo tree -p motolii-ui -i objc2-app-kit@0.3.2
direct dependencyは既存wry / eframe系列と一致
```

Claude Opus 5のread-only限定reviewは`--effort low`で完了し、global screen point、
押下中位置のlatch、継続repaint、probe/state分離を採用した。Stage hit-testは後続へ
維持した。Cursor Grok 4.5 Highのread-only実diff検収も開始したが、約3分間出力が
なく中断した。別modelへfallbackしていない。

次は`CU-0B04N`で同じtop-level Surface内のnative Stage viewport、最新layout epoch、
hit-test/canonical変換の製品ownerを接続する。visual token consumerはこのgeometry /
input接続の入場条件に戻さない。
