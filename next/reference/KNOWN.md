# 検証済み事実(レーンは再検証しない)

発注書はこのファイルを必読に入れる。**ここにある事実の再検証にツール呼び出しを使わないこと**。
疑わしい場合は黙って再検証せず、終了報告に「KNOWN.md の X を疑う、理由」と書く(supervisor が裁定)。
1行1事実、日付と出典を必ず添える。古くなった行は消さず「(失効 YYYY-MM-DD: 理由)」を付ける。

## 上流・依存(2026-08-20 一次ソース確認済み)
- rerun(pin 483b855・上流とも)に**操作ギズモは無い**。"gizmo" は軸表示の doc comment のみ。別語彙(manipulate/dragger)でも無し
- rerun の**音声対応は未実装**(issue #2852/#5181 とも open)。音声・再生時計は自前
- rerun の再生時計(TimeControl)は wall-clock 駆動で**音声同期に構造的に効かない**
- re_renderer に **mipmap 自動生成は無い**(TODO のまま)→ preview 高速化は素材 proxy 一本(裁定21)
- `re_video::load_mp4_from_reader` は **moov だけ先読みのストリーミング対応**(裁定24 の理由(c)は誤りだった)。encode/mux は無し(理由(d)は真)
- `re_chunk_store::gc` / `EntityDb::drop_time_range` は**既存 API**(store 層、即呼べる)
- alpha 付き書き出しには ViewBuilder への **getter 追加+COPY_SRC 付与の2箇所の fork 改造が必要**(裁定16 の「無改造」は楽観だった)
- fork = 上流ほぼ素+seam 13個(全部旧 egui 埋め込み向け=next/ には死蔵)。pin 後の上流250コミットに関連変更なし
- **iced の Theme/Style は色・境界・影のみ。寸法は持てない**(API 実測)。iced 0.14 の公式ホットリロードは実験的すぎて前提にしない(裁定117)
- iced_test 0.14 は動く。ただし **canvas と slider は Simulator から構造的に不可視**
- `transform-gizmo` crate は 3D 前提で skew 無し。旧 gizmo は Motolii 追加(コミット fd6f54ba で依存追加)であって rerun 由来ではない
- rerun viewer の selection panel には**型別 component editor registry**(re_component_ui)がある — Inspector の型の先例(コードは egui 層、引かない)
- pucker/bloat の極性ラベル「正で bloat」は**正しい**(lottie-web 実コード+AE/Illustrator 3系統一致、裁定110)
- tiny-skia の gradient は stops 空/1本/半径0 で panic しない。**非昇順 stops は自動ソートされない**(構築時ソート実装済み、裁定109)
- Ravel の ShellManipulator は 2D・toolkit 非依存だが**利用者裁定でギズモはスクラッチ**(2026-08-20、Ravel はダサいため。plan B=transform-gizmo+自前カメラ行列)

## レーン運用(実測済み)
- **worktree の base はほぼ必ず stale**。作業前に `git reset --hard claude/motolii-reset-handoff-bda7f3` を無条件で行う(確認に時間を使わない)
- stash 禁止(worktree 間で共有)/ Edit 直後の stale fingerprint は touch / CARGO_TARGET_DIR 共有禁止(後勝ち事故の実測あり)
- 時間予算試験2本(`edit_storm_with_the_real_track_type`・r2 `timeline_projection_fits_a_frame`)は**負荷で落ちるのが既知**。単独実行で緑なら自分の変更と無関係。予算は緩めない
- 一次ソースの取得結果は終了報告に URL/rev を書く(次のレーンが KNOWN 経由で再利用できるように)
