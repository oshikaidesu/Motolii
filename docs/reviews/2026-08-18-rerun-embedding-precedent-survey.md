# Rerun埋め込みの前例調査 — 誰がこの道を通ったか

作成日: 2026-08-18

状態: **調査**(観察。決定を含まない)

対象: [Rerun合成基盤裁定](2026-08-18-rerun-as-composition-foundation.md)が前提とする「Rerun viewerを自製品へ埋め込む」形態に、公式サポートと実在の前例がどれだけあるかの外部調査。fork運用のリスク評定は[fork seam台帳](2026-08-18-rerun-fork-seam-ledger.md)、非eguiホストへの搭載実測は[iced再評価調査](2026-08-18-iced-reentry-survey.md)が正本。

## 1. 公式サポート状況

- Rerun公式docsに「Implement custom visualizations (Rust only)」があり、`re_viewer` crateをeframeアプリへ埋め込む形態を**公式に文書化**している。ただし同ページは「The interfaces for extending the Viewer are not yet stable. Expect code implementing custom extensions to break with every release of Rerun.」と明記する(docs 0.36.0時点)。
- 公式exampleは3系統(`extend_viewer_ui` / `custom_callback` / `viewer_callbacks`)が**本体リポジトリ内でCI維持**されている。埋め込みは「野良ハック」ではなく、上流が毎リリース動作を保つ想定線の内側にある。
- Web版のカスタムビルドは非サポート(issue #2337、2023-06起票で現在もopen)。
- 上流自身の拡張性自己評価は「GUI extensions: Level 0」(issue #3087)。

## 2. 実在前例(全数)

ネイティブ埋め込み(`re_viewer`+eframe)の実在例として見つかった全数:

1. **rewire-run/viewer** — 最近接の前例。ROS 2用カスタムビューアで、`re_viewer` 0.36+`re_ui`+eframeの構成。0.34→0.36への製品追従実績あり。2026-03発足の小規模プロジェクト。https://github.com/rewire-run/viewer
2. **luxonis/depthai-viewer** — rerunを丸ごとforkした製品。2026-01-06にarchive化された。**fork追従を怠った末路**の実例として読める。https://github.com/luxonis/depthai-viewer
3. 個人プロジェクト数件。

crates.io上の`re_viewer`逆依存はrerun本体のみ1件(2026-08-18確認)。つまり公開エコシステムに埋め込み利用者はほぼ存在しない。

**Web埋め込み(弱い前例・別枠)**: Hugging Face LeRobot dataset visualizer、gradio-rerun、`@rerun-io/web-viewer`(SDKとviewerのlockstep制約を明記)。これらはviewerを改変せずiframe/wasmで貼る形態で、本件のfork+ネイティブ埋め込みとは別カテゴリ。

## 3. APIチャーン

- リリースはほぼ月次: 0.24.0(2025-07-17)→0.36.0(2026-08-10)で12 minor。
- viewer系crateにsemver保証なし。1.0の気配なし。
- 緩和材料: 公式exampleが毎リリース維持されていること、Rewireが0.34→0.36を追従できた実績。

## 4. 評定

**「公式に舗装されているが通行者がほぼいない道」**である。

- eframe埋め込み自体は公式文書+CI維持example付きで、上流の想定線の内側。
- offscreenで非eguiホスト(iced)に載せる形態は**前例ゼロの単独走**。ただし公式想定線から外れるのはホスト層1枚のみで、その1枚(wgpu offscreen+入力ブリッジ)は[iced再評価調査](2026-08-18-iced-reentry-survey.md)で実測済みの平凡な部品である。
- rev固定+seam台帳([fork seam台帳](2026-08-18-rerun-fork-seam-ledger.md))は、この「月次破壊的リリース+semver保証なし」という状況への合理的な唯一の防御である。depthai-viewerのarchive化は追従を怠った場合の末路を示すが、Rewireは小規模でも追従できることを示している。
- **無謀と判定する材料は出なかった。**

## 5. 再観測トリガー

- Rerun上流に`SpatialStage`類似の埋め込みAPIが出現した時
- `re_viewer`にsemver公約が出た時
- Rewire viewer(最近接前例)の動向 — 追従停止・archive化は警報、継続はこの道の生存証拠

## 6. 出典

- [Implement custom visualizations (Rust only) — Rerun docs](https://rerun.io/docs/howto/visualization/extend-ui)
- [Support custom web builds · rerun-io/rerun#2337](https://github.com/rerun-io/rerun/issues/2337)
- [Extensibility self-assessment · rerun-io/rerun#3087](https://github.com/rerun-io/rerun/issues/3087)
- [rewire-run/viewer](https://github.com/rewire-run/viewer)
- [luxonis/depthai-viewer](https://github.com/luxonis/depthai-viewer)
