# Timeline ordinary UX ledger

- 制定: 2026-08-26
- 機械可読正本: `next/reference/timeline-ordinary-ux.tsv`
- 対象: 機能名より下の、pointer入力からpreview・commit・cancelまでの操作契約
- 判定: `PASS` / `PARTIAL` / `FAIL` / `HOLD` / `ORACLE_GAP`

## 外部根拠

基底契約は [W3C Pointer Events](https://www.w3.org/TR/pointerevents/) のpointer capture・
`pointercancel`・`lostpointercapture`、[WCAG 2.2](https://www.w3.org/TR/WCAG22/) の
2.5.2/2.5.7/2.5.8、[Apple Drag and Drop](https://developer.apple.com/design/human-interface-guidelines/drag-and-drop)
を使う。編集ソフト固有の収束は次を使う。

- Adobe: [After Effects UI scroll/zoom](https://helpx.adobe.com/after-effects/desktop/get-started/get-familiar-with-the-interface/general-user-interface-items.html)、[After Effects layer arrange/trim](https://helpx.adobe.com/after-effects/desktop/work-with-layers/select-and-arrange-layers/selecting-arranging-layers.html)
- Premiere: [Tools panel](https://helpx.adobe.com/premiere/desktop/get-started/tour-the-workspace/tools-panel-and-options-panel.html)、[Snap clips](https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/snap-clips.html)、[Move clips](https://helpx.adobe.com/premiere/desktop/edit-projects/change-clip-sequence/different-ways-to-move-clips.html)
- Resolve: [Resolve 18 Editor's Guide](https://documents.blackmagicdesign.com/UserManuals/DaVinci-Resolve-18-Editors-Guide.pdf)
- CapCut: [How to use CapCut](https://www.capcut.com/resource/how-to-use-capcut)
- Apple: [Mac trackpad gestures](https://support.apple.com/guide/macbook-pro/trackpad-apdbb563a1bc/2026/mac/26)、[Trackpad settings](https://support.apple.com/en-ie/guide/mac-help/-mchlp1226/mac)、[NSEvent momentumPhase](https://developer.apple.com/documentation/appkit/nsevent/momentumphase)、[precise scrolling delta](https://developer.apple.com/documentation/appkit/nsevent/scrollingdeltax)

外部資料が説明しないdrag threshold・snap px・no-op undoは、正解を捏造せず
`ORACLE_GAP`とする。zoom anchorはAEがcursor/centerを選べ、Resolveがplayhead中心なので
`HOLD`である。

## 結論

30契約の現在判定は、共有意味核が `PASS 17 / PARTIAL 13 / FAIL 0`、Makepadホストが
`PASS 9 / PARTIAL 10 / FAIL 11`。製品としては共有核が実装済みでも、現在の正式ホスト候補から
到達できなければ適合とはしないため、総合は `PASS 7 / PARTIAL 12 / FAIL 10 / HOLD 1`
（FAILのうち1件は外部資料が正解を定めない`ORACLE_GAP`由来）。

Makepad側の最初の穴は機能追加ではなく入力分解である。現在は時間面全体がscrub、rail全体が
lane reorderに先取りされるため、clip body/trim edge/key/blank/playhead/lane selectionをdown時に
分類できない。加えて`FingerScroll`のx/yを単一量へ畳み、`ScrollPhase`も見ていないため、wheelと
二本指pan・pinch・momentumを区別できない。次の順序は `hit ownership → selection → cancel →
gesture classification/phase → axis ownership → scroll/zoom modifier → snap preview → drag alternatives/target size`
とする。

## Motoliiで既に成立している下層契約

- down時にbar body/edgeを固定し、originから絶対値でpreviewを再計算する。
- preview中はDocumentを書かず、releaseで`apply_all`を1回だけ行う。
- no-opはundoを増やさず、複数対象は相対間隔を保って一括clampする。
- snap候補、画面px閾値、修飾キーによる一時無効化、key選択修飾が意味核にある。
- Makepadでもpointer capture、lane drop preview、release時のsemantic restack、x-only zoom、
  tick再計算、境界clamp、gesture identity固定は成立している。

## Makepadホストで未適合の普通動作

1. clip/key/blank/playheadのhit ownershipが無く、時間面クリックが全部scrubになる。
2. lane click単独選択、Cmd toggle、Shift rangeが無い。
3. Escape・pointercancel・lost capture・focus lostのcancelが無い。
4. drag thresholdが無く、微小ぶれと意図したdragを区別しない。
5. clip/keyのsnap、snap guide、gesture中のsnap反転が無い。
6. modifier無しwheelまでzoomに奪われ、通常scrollとの役割分担が無い。
7. 小さいM/S/L等のtargetと、dragを使えない場合の代替入口が不足する。

## トラックパッド固有の未適合

静的実装済み・実機未検収:

1. 修飾無し二本指horizontal panと、Option-scroll time zoomを別動詞にした。
2. `Began/Touched/Changed/Ended/Momentum/MomentumEnded`を状態機械へ入力する。
3. momentumのowner継続、新しいtouchによるcatch/停止を行う。
4. dominant axisと動詞をgesture開始後に固定する。
5. platform deltaを再反転せず、精密deltaと段階wheelを別に正規化する。

platform producer未実装:

1. Windows precision touchpadとLinux Waylandのnative transform producer。共通gesture契約と
   macOS producerは実装済みで、未対応platformはOption/Alt-scroll fallbackを維持する。

`ScrollPhase`とOS momentumはMakepadのイベント層に既に存在するためホストで実装した。pinchは
Makepad forkへ意味を持たないtransform sample（phase/centroid/translation/scale/rotation/device）を
追加し、Motoliiの薄いadapterだけが独自gesture sampleへ変換する。Timeline policyはその下流に置く。
8契約は純関数試験が通った段階なので`PARTIAL`とし、実機操作後に`PASS`へ上げる。

## 更新規則

- `core_status`はIced画面の存在ではなく、`Shell/Document`へ委託できる意味契約を表す。
- `makepad_status`はMakepadの実入力からその契約へ到達できるかを表す。
- `overall`は両方の弱い方。型・純関数だけなら実窓検収済みとは書かない。
- 行を`PASS`へ上げるには、TSVの`acceptance`を自動試験または窓操作で満たし、`evidence`を更新する。
