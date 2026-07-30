# SD-02G product Host layout geometry実装決定

- 日付: 2026-07-30
- 状態: **決定**
- `SD-02G`: **DONE**
- 次PRODUCT-ASSET `DO`: **未選定（0件）**

## 1. 目的と結果

通常製品routeの`NativeHostLayout`が組み込みshare定数を直接読み、既存の
toolkit非依存`PanelLayout` / `LayoutAuthority`と別にgeometryを所有していた状態を閉じた。

既決の`taffy 0.12.2`を`motolii-ui`内のprivate rectangle projectionだけへ採用し、
製品`ProductApp`が一つの`LayoutAuthority`を所有して、そのintentからBrowser、Stage、
Inspector、Timelineのlogical rectangleとnative physical viewportを導出する。
taffyへpanel identity、layout intent、selection、Document、window lifecycleを所有させない。

## 2. 境界

変更対象:

- workspace / `motolii-ui`のtaffy依存
- private `layout_geometry`
- `NativeHostLayout`のauthority入力
- `ProductApp`の既存`LayoutAuthority`所有

非目標:

- Window menu、layout操作入力、detach / re-dock、追加panel role
- Workspace profile codec
- React、Document、journal、Undo、plugin契約、公開API
- visual / golden / threshold変更

hidden subtreeはgeometry treeから除外し、残るvisible shareを再正規化する。tab stackは
active visible roleだけを同じrectへ投影する。通常製品routeで必要な4 roleが欠けた場合は
silentなblank layoutにせず、private typed errorを返す。taffy roundingは無効にし、
logicalからphysicalへの変換だけをrounding境界とする。

## 3. 検証とレビュー

- `cargo test -p motolii-ui`（unit 130件と全integration/doc tests）
- `cargo clippy -p motolii-ui --all-targets -- -D warnings`
- `./scripts/check-ui-toolkit-deps.sh`
- `cargo fmt --all -- --check`
- `git diff --check`

Claude Code `claude-opus-5`最終レビューは初回`REJECT`（P0=0 / P1=2）。
hidden roleでHost全体がsilent `None`になる問題と、hidden shareの未再配分を修正し、
hidden/tab/no-gap、mutated authority、typed failure、DPIの負例を追加した。
再レビューは`VERDICT: ACCEPT`、P0=0 / P1=0。

後続P2は、hidden active tab時のfirst-visible fallback、taffy依存監査名の追加、
projection error細分化、極小viewport / 非dyadic share追加試験である。製品layout入力を
接続する粒で意味とconsumerを同時に再照合し、本粒へWindow menuや新ownerを混ぜない。
