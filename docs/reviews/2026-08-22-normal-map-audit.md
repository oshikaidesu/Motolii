# normal-map 全面精査(MA、9レンズ) — 2026-08-22

読み取り専用の精査レーン。**normal-map.tsv 本体は触っていない**。以下は全て提案(確定は supervisor/利用者)。

## 0. 対象ファイルについての注記(重要・最初に読むこと)

このレーンは worktree(`agent-a497cb664dffaae73`)に隔離されている。`git` を使う相対操作は共有チェックアウトへ
またがれない制約があるため、**読み取りは共有チェックアウト(`/Users/member_ottoto/rust_ae/Motolii/`)の現物を使った**
— worktree 側の `next/reference/normal-map.tsv` は commit `7bcd1b7f`(MB-0 着地・消化132/1,551)時点で止まっており、
裁定173/174・H1・SS 台帳など本精査が前提とする最新状態を欠く。共有チェックアウト側は 1,552 行(1行増: 差分未確認だが
既決事項が反映済み、例: map 957 は既に裁定174 で不採用へ更新済み)。**本監査は共有チェックアウトの現物を正として実施**。
worktree の map が古いままなら、supervisor 側で最新 map をこの worktree へ同期してから本監査の提案を適用すること。

## 1. 全体像

| 項目 | 数値 |
|---|---|
| 総行数 | 1,552(ヘッダ除き1,551) |
| 採用済 | 124 |
| 採用予定 | 1,162 |
| 不採用 | 265 |
| freq≥2(採用予定内) | 29 |
| freq分布(採用予定) | freq1=1,134 / freq2=21+β / freq3=7 / freq4=1(概算、下記L4節で束ね後の再計算あり) |

カテゴリ別(採用予定の内訳、上位):misc 138 / clip_edit 120 / playback 92 / tool 65 / layer_transform 65 /
preferences 58 / timeline 56 / panel_window 52 / workspace 50 / view_display 49 / audio 49 / mask 42 /
effects_animation 40 / text 38 / blend_mode 31 / camera_3d 30 / edit_basic 27 / color 25 / import_export 24 /
marker 23 / project 22 / help 20 / label_color 19 / export_render 13 / caption 13 / ai_feature 1

## 2. 分類の型 — カテゴリ単位の機械束ね

指示どおり、機械的に流せる行はカテゴリ単位で束ねた。個別に争点のある行は §4〜§7 で行 id 単位に分解している。

| カテゴリ(採用予定 N) | 既定4値 | レンズ札 | 一行理由 |
|---|---|---|---|
| blend_mode(31) | 今 | L1/L5 | BL3(分離可能11)/BL4(非分離4)が裁定161完了で発注可能。Normal/Add 以外は個別 vism 実装行としてそのまま消化対象 |
| camera_3d(30) | 今/一部継ぎ目 | L1/L8 | カメラ束(裁定116)は済。gizmo 系(141-144 Position/Rotation/Scale/Universal gizmo)はギズモ拡張が裁定124止まり(未完節9)なので継ぎ目。View切替・Track Camera 等は今 |
| effects_animation(40) | 今 | L4/L1 | keyframe 操作(追加/削除/選択/移動/補間)は store 実装済みの Intent の薄い顔。個別行は UI 結線の消化対象。L4: Easy Ease 系6行(485-490)は「区間イージング適用+in/out限定」の1機構2引数に縮約可 |
| mask(42) | 今 | L2/L8 | MK1/MK2(裁定108/133)で基盤(被覆代数・ラスタ)着地済み。残りは Feather/Expansion/Free Transform 等の UI 結線。基盤は既払い、顔は個別に軽い |
| text(38 残り) | 今 | L8/L4 | TextRange/TextRangeSelector 基盤(裁定121/122)が着地済みで大半 採用済。残る行(Character Offset/Value・Line Anchor・Wiggly・Blur 等)は同一基盤上の属性追加=薄い顔。カーソル/選択操作5行は§6で縮約 |
| marker(23) | 今 | L4 | 基盤(裁定111/120/149)着地済み。In/Out/Clear/Flag/Nav 個別UIが残り。§6で7行縮約の型見本 |
| project(22) | 今 | L1 | New/Save/Quit/Auto-Save 等は GOALS M1/M11 の直接範囲。Media & Disk Cache(1236)は§4クラスタへ合流注記 |
| import_export(24) | 今 | L1 | AAF/XML/EDL 相互運用・footage 差替・proxy 設定は GOALS M2/M9 の周辺で妥当 |
| export_render(13) | 今/一部対象外 | L1 | Quick Export・Current Frame as Still 等は今。Bin/Collect Files/Media Management は Resolve 版元管理機能で領域境界要再確認(争点束へ) |
| help(20) | 今 | L1 | 574 の既存reasonどおり、競合製品名(DaVinci Resolve Reference Manual 等)は「一般的なヘルプ資源の型」への写像として妥当(§8注記) |
| label_color(19) | 今 | L1 | AE型ラベル色12種+Labels…設定は ρ(裁定済み label_color index)の直接消化対象 |
| audio(49) | 今 | L1 | A2(実時間再生)着地済みの上に乗る音声UI個別行(gain・mute/solo・波形帯等)。Apply Batch Fades(3/23)は§8スクリプト/Intent節参照 |
| color(25) | 今/一部対象外 | L1 | 基本カラー調整は今。Tracker/Fairlight/Dolby Vision 系(417/424等)は既に不採用済みで正しい(カラーグレーディングスイート級は領域外) |
| clip_edit(120) | 今/一部不採用候補 | L1/L3 | 大半はGOALS M5-M7直接範囲。trim family(ripple/insert/overwrite/lift/extract)は既に系統的に不採用済みで正しいが、**441 Paste Insert が同族なのに採用予定のまま**(§4) |
| playback(92) | 今 | L1 | Space再生・scrub・loop等はA2/M8の直接範囲。**1054 Extract/1080 Lift が clip_edit 側の不採用と矛盾**(§4) |
| tool(65) | 今/一部継ぎ目 | L1/L8/L9 | 選択・変形ツール等は今。Paint系(Brush/Clone Stamp/Eraser、§7)は継ぎ目候補。Puppet(1375)は既に正しく不採用 |
| layer_transform(65) | 今 | L1 | Anchor/Scale/Rotation/Position 等 GOALS標準の直接範囲(バック優先順序4番目) |
| timeline(56) | 今 | L1 | レーンバー・ナビゲータ等 Timeline第2波の直接消化対象 |
| panel_window(52) | 今/一部継ぎ目 | L1/L8 | 各パネルの開閉は今。Paint/Brushes パネル(989/1023/1519/1523)はtool側paint継ぎ目判定と連動 |
| workspace(50) | 今 | L1 | レイアウトプリセット・パネル開閉束。980/981/982/983 が freq上位(§6) |
| view_display(49) | 今 | L1 | Zoom In/Out・Wireframe等。**1451 Wireframe は844-846クラスタと別物(問題なし)** |
| edit_basic(27) | 今 | L1 | Copy/Paste/Find/Edit Original 等GOALS標準直接範囲 |
| preferences(58) | 今/要精査クラスタ | L2 | 大半は今。**メモリ/ディスクキャッシュ系(1147/1148/1159/1183/1194/1195/1197/1198/1199/1207/1236)がSS台帳項目1の構造解決(ゼロコピーGPU再生)と重複** — §4で一括検討 |
| misc(138) | 今/一部要精査 | L1/L2 | 大半は今(Active Window束・Effect関連・star/polygon形状操作等)。**844/845/846 Fast Previews Draft系が裁定21の実測と矛盾**(§4) |
| ai_feature(1) | 既決 | — | Auto reframe(#1)は解析→keyframe駆動として既に妥当な採用予定。#2 Image to videoは既に正しく不採用 |

## 3. 909/910(Pre-compose)再判定

SS台帳は「H1着地前に不採用確定=verdictが実装を先回りした例」と指摘した。現状を再確認:

- **H1(親変換合成)は着地済み**(検収合格・merge `74da6c42`)。`StoreView::world_affine` が数値証明つきで「親が動くと子も動く」を実証、`LayerSource::Group` マーカーも着地
- **H2(Timelineツリー行)も着地済み**(merge `016b29e3`)
- **G1(グループ化動詞 ⌘G/⌘⇧G の実UI)はまだ未着手** — 裁定174 の設計(oracle型まで規定済み)はあるが、lane-board の「走行中」「完了」いずれの表にも G1 の実装行が無い。457/468-470行(Group/Ungroup系)はまだ消化されていない

**提案**: 909/910 の verdict(不採用)は維持が妥当 — D5(Expression)のケースと違い、G1 に開いた設計上の未決点(pick-whipの任意プロパティ参照のような網羅性の疑問)は無く、oracle型まで規定済みの「実装待ちの機械的な残り」でしかない。H1 が最大のリスク(再帰合成のアルゴリズム)を実証済みなことが D5 との違いを分ける。ただし **理由テキストが古い**(現在「GOALS要らないもの…(裁定119)」を引用しているが、正しい引用は裁定173/174であるべき)。理由テキストの更新のみ提案:

> 909/910 理由 → 「プリコンポの『まとめる』用法は裁定173(H1着地済み・親変換合成が数値証明あり)+裁定174(G1グループ化動詞、oracle型規定済み・実装待ち)で置換。isolate/freeze用途は別裁定(未完節9)で対象外」

## 4. verdict整合の修正提案(確定リスト)

| id(s) | 現verdict | 提案 | 根拠 |
|---|---|---|---|
| **1450** Fast Previews>Adaptive Resolution | 採用予定 | **採用済へ** | μ裁定163・merge `3737d5c4` で Auto/½/¼ cap が実装済み(SS台帳項目7、実装済みのverdict更新漏れ) |
| **696/697/705** Pre-render/RAM Preview/Save RAM Preview | 採用予定 | **不採用候補**(確定は保留・下記注記) | SS台帳項目1: ゼロコピーGPU再生(裁定166/170/171)がプレビューの遅さを構造解決。RAM上に事前レンダーしてキャッシュする機構自体が前提を失う。**ただし境界: 巨大comp・重エフェクト構成での実測はまだ**(M4検収記録の正直な境界)ので、断定不採用でなく「継ぎ目(重い構成での再測定待ち)」が安全側 |
| **1148/1194/1207/1236**(+関連1147/1159/1183/1195/1197/1198/1199) Memory&Disk Cache/Purge系 | 採用予定 | **不採用候補**(同上、確定は保留) | 同じ根 — 明示キャッシュ管理UIが必要な前提(readback遅延・CPUキャッシュ肥大)がゼロコピー経路では成立しない。境界: 市松ONはCPU合成フォールバック(readback発生)が生きているため**完全に不要とは言い切れない** |
| **844/845/846** Fast Previews>Draft/Fast Draft/Off | 採用予定 | **不採用候補** | 裁定21実測「解像度を落としても速くならない(帯域律速)」がこれらの存在根拠(画質を落として速度を稼ぐ)を直接否定。SS台帳のL2最重要指摘対象 |
| **441** Paste Insert | 採用予定(理由欄空欄) | **不採用へ** | clip_edit の trim family(ripple/insert/overwrite/lift/extract、拘束1・2026-08-19裁定)と同族。Insert編集=後続clipを押し出すripple挿入で、Motoliiの自由配置(gapless packing非前提)と機構衝突。263番(Q/W=Insert/Overwrite)は既に同じ理由で不採用済みなのに441だけ漏れている |
| **1054/1080** Extract/Lift(category=playback) | 採用予定(理由欄空欄) | **不採用へ**(208-210/226-228と統合) | clip_edit側の同名概念(Extract=208/209/210、Lift=226/227/228)は既に「trim family不採用—拘束1」で不採用済み。playbackカテゴリに二重採録され判定が矛盾している。カテゴリを跨いだ重複行の典型例 |
| **740** Ripple Timeline Markers | 採用予定 | 争点束へ(§8) | ripple編集自体が拘束1で不採用のため、この設定の存在根拠(ripple中にマーカーを追従させるか)が空洞化。ただしマーカー基盤への影響は軽微で断定しづらく、争点束行き |
| **471/475/492/493/1173/1209** Expression系6行 | 不採用 | **保留(D5完了まで)へ** | SS台帳の自己矛盾(c類)指摘どおり: GOALS D5(型付きlinkでExpression全数カバー)が「未・カバレッジ表に穴」のまま不採用を確定させている。pick-whipの任意プロパティ参照等、型付きlinkが本当に全用途を覆うかが未検証のうちは verdict確定でなく保留が正しい |
| 957 Show or hide Parent column | (共有チェックアウトで既に不採用) | 変更不要(確認のみ) | 裁定174で既に正しく反映済み。worktree側map更新の際にこの行が含まれているか確認すること |

## 5. 縮約(L4)と freq 再計算

同一意図の別機構行を意図動詞へ縮約した例(行は消さない、意図写像案のみ):

| 縮約先の意図動詞 | 対象行(id) | 元の行数 | 縮約後 |
|---|---|---|---|
| `set_work_area{in?, out?}` / `clear_work_area{in?, out?}` | 719,720,721,724,725,726,727(marker) | 7 | 1(2引数の組み合わせで7UIを表現) |
| `zoom(direction)` | 1441,1442(view_display) | 2 | 1 |
| `apply_transition(kind: video|audio)` | 164,165(clip_edit) | 2 | 1 |
| `extend_text_selection(unit: char|word|line, dir)` | 1267,1268,1269,1270,1271(text) | 5 | 1 |
| `move_text_cursor(target)` | 1281,1282,1283(text) | 3 | 1 |
| `set_line_break_mode` | 1276,1277,1278,1307(text、既に同束と自己記載あり) | 4 | 1 |
| `apply_ease(scope: selected|all, side: in|out|both)` | 485,486,487,488,489,490(effects_animation) | 6 | 1(既に1機構+スコープ引数) |
| `serialize_intent_bundle`(preset)の薄い顔 | 574,809,815,918,938(help/misc) | 5 | 1入出力面(save/apply/browse/recent は同一機構の4動詞) |
| `apply_to_selection(op: batch_fade)` | 3,23(audio) | 2 | 1 |

**freq再計算の影響**: freq≥2の優先キュー(現29件)のうち、marker 7行(freq各2)が1意図へ縮約されると重複計上が-6、
zoom 2行→1で-1、transition 2行→1で-1。**優先キューの実質的な独立意図数は 29 → 21 まで圧縮される**
(marker/zoom/transitionの3クラスタが縮約対象であるため)。text選択系(freq1)・preset系(freq複数だが分散)は
freq≥2キューの外なので優先度への影響は間接的(縮約後にfreq再計算すればキュー入りする可能性はあるが本監査では未実施 — EVIDENCE_GAP)。

## 6. L8/L9 型見本(指示どおり実施)

**ペイント族**(基盤/顔分解+島度):
- 基盤: パス編集(mask MK1/MK2で既払い — Path→Shape橋・coverage合成が着地済み)
- 顔: Brush/Clone Stamp/Eraser(1374,1427,1428,1429,1430,1468 他)は「筆致をラスタとして層に焼き込む」という**別内容**で、mask基盤を再利用できない(maskは選択範囲の被覆、paintは永続ラスタコンテンツ)
- 島度: **出力辺=ゼロに近い**。paint strokeの出力(ラスタ画素)はmask/transform/keyframe/他layerの合成へ流れない — 自層のpixelとして完結するのみ。**継ぎ目候補**(L9)。傍証(利用者実観察メモ): AMにもペイントはあったが皆パスを使った、という記録と整合

**maskとの対比**(同じ基盤を使うが逆の島度):
- mask自体はvector path(パス)が入力で、出力(coverage)がcompositorの合成・matte・他エフェクトのstencil入力へ流れる=**出力辺が太い**。基盤も顔も**コア**(L8)

**テキストアニメータ族**(基盤共有・高整合):
- 基盤: TextRange/TextRangeSelector(裁定121/122)が着地済み・大半採用済(text_range_fill_color等)
- 顔: Character Offset/Value(1256/1257)・Line Anchor(1279)・Wiggly(1313)・Blur(1255)は同一TextRangeSelector機構上に新しいproperty target/selector種別を足すだけ
- 島度: セレクタの出力(選択された文字集合)はrender pipelineへ直接流れる=**出力辺が太い**。基盤既払いにつき**追加コストが低い="今"**(paintと対照的な結論)

**トラッカー×スタビライズ族**(基盤共有の発見的列挙、高整合):
- Motion Tracking(1038パネル/1515workspace)・Keyframe Animation and Motion Tracking(494)・Warp Stabilizer VFX(974)は「フレーム解析→keyframe/補正transformを生成」という共通機構(Auto reframe #1の「Bake/Analysis provider経路」と同型)
- 出力(トラックされた点・補正transform)はlayerのtransform系やnullの位置keyへ直接流れる=**出力辺が太い**。**継ぎ目ではなく"今/後"**が妥当。ただし実装コストは高い(解析アルゴリズム自体が無い)ため優先度は低め、時機レンズ(L7)では"後"寄り

## 7. スクリプト系(Intent背骨)の扱い

ExtendScript語彙(Run Script File・Enable JavaScript Debugger・Interrupt running script・Open Script Editor・
Warn User When Executing Files・Allow Scripts to Write Files等)は**既に系統的に不採用済み**で正しい —
SS台帳supervisor追記の「Intent背骨=スクリプト命令集合」の帰結どおり、独立したスクリプティング言語/デバッガ/実行許可
UIを別途作る必要が無い(JSON-Intent+query API+undo束ね規約の薄い顔で足りる)。

一方、**Animation Preset族**(574 Help/809 Apply/815 Browse/918 Recent/938 Save)は「名前つきIntentバンドルの
保存・適用・閲覧・履歴」であり、これも同じIntent背骨の薄い顔(§5でL4縮約済み)。Expression(式)とはっきり区別が要る:
Expressionは**per-property reactive**(D5、型付きlinkで置換予定・未完)であるのに対し、Preset/Batchは**one-shot適用**
(apply_allで1回書き込んで終わり)で、既に着地済みのIntent機構がそのままカバーする。両者を同じ「スクリプト」枠で
混同しないことが§4の保留提案(Expression系)と§5の縮約提案(Preset系)の判定を分けている根拠。

## 8. 争点束(30行以内)

レンズ間で判定が割れる・教義解釈が要る・利用者の実観察が要る行のみ。

| id | canonical | 現verdict | 争点 |
|---|---|---|---|
| 696/697/705 | Pre-render / RAM Preview / Save RAM Preview | 採用予定 | L2: ゼロコピーGPUで構造解決済みだが巨大comp実測なし。不採用/継ぎ目のどちらかは実測データ待ち |
| 1148/1194/1207/1236/1147/1159/1183/1195/1197/1198/1199 | Memory&Disk Cache/Purge/Multiprocessing系(preferences 11行) | 採用予定 | L2: 同上。市松ONのCPUフォールバックが生きているため完全不要と言い切れない境界あり |
| 844/845/846 | Fast Previews>Draft/Fast Draft/Off | 採用予定 | L2/L3: 裁定21実測が存在根拠を否定。不採用が本命だが、UIとして「品質を落とす」ボタン自体の需要(バッテリー/熱等の別理由)が別途あり得るかは利用者判断が必要 |
| 441 | Paste Insert | 採用予定(理由欄空欄) | L3: trim family(拘束1)との整合。同族の263は既に不採用。機械的には不採用が妥当だが、"Paste"の一般利便性(挿入でなく単純上書きpasteとして再定義する道)も検討可 |
| 1054/1080 | Extract/Lift(playback) | 採用予定(理由欄空欄) | L3: clip_edit側の同名行(208-210/226-228)と矛盾。重複統合(不採用化)が機械的に妥当 |
| 740 | Ripple Timeline Markers | 採用予定 | L3: ripple編集自体が不採用のため設定の意味が空洞化。影響軽微につき判断保留 |
| 471/475/492/493/1173/1209 | Expression系6行 | 不採用 | L2: D5未完のため自己矛盾。保留への変更提案(§4)、利用者が「型付きlinkで本当に全用途をカバーする意図か」を最終確認すべき |
| 909/910 | Pre-compose系 | 不採用 | L1: 維持提案(§3)だが、G1未着手のうちに「置換済み」と言い切れるかは利用者感覚の確認価値あり |
| 141-144 | Position/Rotation/Scale/Universal gizmo | 採用予定 | L5/L8: ギズモ拡張は裁定124でスクラッチ方針決定のみ・実装は静止描画止まり(未完節9番)。"今"と"継ぎ目"の境界線上 |
| 1374/1427-1430/1468/989/1023/1519/1523 | Brush/Clone Stamp/paint系(9行) | 採用予定 | L8/L9: §6の型見本どおり継ぎ目候補。GOALS要らないものに明記は無いが低島度。次のvism優先順とどちらを先に発注するかは意図優先(L1)の判断が要る |
| 574/809/815/918/938 | Animation Preset族(5行) | 採用予定 | L4/L7: 縮約後は1機構4面。時機(L7)は"今"寄り(Intent apply_all既存)だが、UIとしてのpresetブラウザ自体は後回し可 |
| 3/23 | Apply Batch Fades/Batch Fade Settings | 採用予定 | L4: 縮約後1機構。バス/コンソール不要な単純機能という既存reason自体は妥当 |
| 1266 | Enable Per-character 3D | 不採用 | L3: 2.5Dモデル(裁定113/115)との衝突は明記済みで妥当だが、D12(3D差別化)の将来スコープ次第で再訪の可能性がある — 現状維持でよいが根拠の再訪条件が未記載 |

## RETURN 集計

- **4値件数**(カテゴリ束ね+個別判定の合算、概算): 今=約1,050 / 後=約20(トラッカー族等の高コスト機構) /
  継ぎ目=約20(paint族9行+gizmo4行+Pre-render系候補7行等) / 不採用候補(新規提案)=約10(441/1054/1080/844-846)
- **verdict修正提案**: 確定提案2件(1450→採用済、441・1054/1080→不採用)、条件付き提案(継ぎ目)3クラスタ(696/697/705、
  Memory&Diskキャッシュ11行、844-846)、保留提案1クラスタ(Expression系6行→保留)、理由テキスト更新のみ1件(909/910)
- **縮約で消える行数**: L4縮約9クラスタで 7+2+2+5+3+4+6+5+2 = 36行 → 9意図(縮約差分 -27行相当、行自体は削除しない)
- **freq再計算の影響**: freq≥2優先キュー 29→21(marker/zoom/transitionクラスタの重複計上解消)
- **争点束**: 13行(id単位では約35行相当だが表としては30行以内の粒度に圧縮)

## EVIDENCE_GAP

- worktree側 `next/reference/normal-map.tsv` が共有チェックアウトより古い(commit `7bcd1b7f` 止まり)。本監査は
  共有チェックアウトを正として実施したが、**両者の完全差分(957以外の他の相違有無)は未確認**
- text選択系・preset系縮約クラスタのfreq再計算後の優先キュー入り判定は未実施(§5末尾に明記)
- 1,162行の**全数個別判定は実施していない**。カテゴリ単位の機械束ね(§2)+名指しされた検証対象(§3-8)に絞った。
  個別に争点化していない行の中に見落としがある可能性は残る(カテゴリ既定値からの逸脱チェックは代表サンプルのみ)
- export_render の Bin/Collect Files/Media Management(Resolve版元管理機能)が領域内か領域外かは未確定のまま
  §8争点束に入れず「一部対象外」注記に留めた — 精査時間の制約による省略
- 740(Ripple Timeline Markers)・141-144(gizmo)の最終判定は利用者/supervisor裁定待ちとして争点束止まり
