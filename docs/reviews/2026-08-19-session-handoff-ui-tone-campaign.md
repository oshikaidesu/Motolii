# セッション引き継ぎ — UIトンマナ統一 campaign(第5区切り)

日付: 2026-08-19
状態: **引き継ぎ**(観察。決定は各裁定文書)
前区切り: [第4区切り引き継ぎ](2026-08-19-session-handoff-timeline-port-and-instruments.md)

## 現在地(機械的事実)

campaign 正本 = [UIトンマナ統一campaign](2026-08-19-ui-tone-unification-campaign.md)(裁定4本の追記込み)。
第3波設計 = [フラット文法の正本改定と定数化](2026-08-19-flat-grammar-canon-revision.md)(定数表が正本)。

本区切りで main に入った列(主要のみ):

| commit | 中身 |
|---|---|
| `bbdb6f7d` | campaign 文書(目標更新裁定 D1〜D8) |
| `a87a172c` | レーンA merge: 拒否理由表示のテスト固定(11経路、配管は既修復と判明) |
| `e75be5b8`/`d3a6672e` | レーンC/B merge: 回帰の柵2本 / chrome統一(ベージュ退治・legend文法・type_scale/space新設) |
| `eea0aaad` | レーンD merge: timeline/palette.rs 新設(クリップ8色=Browserモック設計値・param色統一・timecode amber) |
| `757496a9`/`734a5923` | レーンE/F merge: ヘッダ帯新設+下帯1行化 / Timeline縦密度(行20・transport24・ruler20・overview14・rail196・M/S/L 16×16) |
| `7d38b62f` | 能力台帳再計測(**有32/部分2/無14**、危険候補3/4解消) |
| `ad3370ad` | 第3波設計(定数表) |

検収は全 merge 後に main checkout の target で `cargo test -p motolii-shell-iced --no-fail-fast` を
完全ログで実行(第1波後 186 passed、第2波後 192 passed、いずれも 0 failed・31バイナリ)。
full workspace gate は区切り冒頭に1回(fail-fast 挙動+末尾クリーンで green 判定。
完全集計は次回 `--locked --no-fail-fast` を1パイプ tee で)。

## 利用者裁定(本区切り、時系列)

1. **目標更新**: Ableton のようなトンマナ・一目で多情報・普通に使えるソフト。現UIは「野暮ったい」
2. **iced エコシステムを検索**(→ [採掘2段](2026-08-19-iced-ecosystem-mining.md)。COSMIC 深掘りも指示)
3. **第2波**: 「タイムラインから下がどデカい(skia/egui参考)」「Undo/Export はヘッダへ」「ログは下へ」
4. **第3波**: 「ここから先は**正本改定を土台**とし、Ableton系の**余白を作らないフラットUI**を調査」
5. **ゴール**: 「**評価軸の定数化**を行えるようになるまで。一面の情報量は多く保つ」
6. **UIスケール%**: Ableton の全体%調整が好み → 定数化(第3波)→スケール%(第4波)の順序を確定

## 実測資産(本区切りで作った判断根拠)

- [Ableton密度実測](2026-08-19-ableton-density-measurements.md): 行高20px・transport30・本文11px帯(公式マニュアルPNG、2x確認済み)
- Abletonフラット文法画素実測(要点は[第3波設計](2026-08-19-flat-grammar-canon-revision.md)に転記): **隣接面は同色・分離は1〜2pxの線のみ・gap0・行内padding≈2px・角丸0**
- モック余白の器具実測(同転記): モックは既に大半フラット。問題は分散(面色 3/8〜9/6段バラバラ、文字基準 9/8/11 バラバラ)
- [icedエコシステム採掘](2026-08-19-iced-ecosystem-mining.md): cosmic-theme α ladder(hover0.1/pressed0.2/disabled0.5)コード流用可、Liana(BSD-3)= per-widget theme の許諾実例、GPL系参照不可リスト

## 走行中・キュー

- **走行中: レーンG**(モックcss改定 — 第3波設計の定数表へ。css_metrics_oracle の pin 追随込みで green 返し契約)
- キュー: **レーンI**(iced追随: browser_pane colors/font_size・型scale{11,9,8}+title12・spacing{2,4,6,8}への全面置換)→ **レーンH**(トンマナoracle群=ゴール: css-metrics検査+ソース走査fence+contrast計算+行高種類数)
- 第4波: UIスケール%(TaskList #7。iced fork の scale_factor API 実機確認から)
- タスクチップ発行済み: Cmd+A が fold 状態を見ない不整合(pane.rs:394、台帳再計測の新発見)
- 未処理の残差: 下帯でログとヒントが密着(gap無しでclip)/ Stage のオリーブ面(Rerun側・据え置き)/ iced が Document の envelope.color を読まない両shell乖離(レーンD報告)

## 運転知見(本区切りの実測。次セッションは必ず読む)

1. **共有 CARGO_TARGET_DIR(/private/tmp/motolii-lane-target)は並走時に本物の汚染を起こす**
   (lock 待ちではなく): 別 worktree のパスを含む失敗値・存在しないテストの実行・**ソース変更が
   無視され stale バイナリでテストが走る**(supervisor 自身でも再現)。レーン3本+supervisorの
   計4回実測。**合否判定は必ず main checkout の target で行う**。レーン内の red は隔離再実行
   なしに信じない。green も「テスト名が手元のソースと一致するか」を見る
2. Edit 直後の cargo は mtime 同期遅延で stale fingerprint を掴むことがある(レーンF実測)。
   `touch` してから回すと確実
3. subagent は背景 cargo を待って中間停止する(3レーンで再現)。**capsule に「cargo は前景・
   timeout 600000」を最初から書く**(第2波以降は明記して解消)
4. cargo 出力の集計はパイプ1本で tee 保存し、exit code を直接 echo する(grep|tail は先頭が欠ける)
5. Ableton 公式マニュアルの画像は 144dpi メタデータ付き=2x確定で、画素実測の一次資料として使える

## 状態の正本

campaign 文書(裁定群)+ 第3波設計(定数表)+ 本引き継ぎ + docs/CANON.md + 能力台帳(再計測追記)。
メモリ: normal-editor-campaign-playbook(運転の型)/ ui-color-direction-ableton / ui-hand-feel-direction。
