# レビュー文書の規律(2026-07-12制定)

このディレクトリの調査・レビュー・ゲート文書、および以後の調査・仕様レビューに適用する継続規律。制定の経緯: 2026-07-12の先例調査2本([考慮漏れ調査](2026-07-12-prior-art-gap-survey.md)・[成功先例](2026-07-12-success-prior-art.md))がいずれも独立レビュー([反対側レビュー](2026-07-12-prior-art-gap-counter-review.md)・同日の批判レビュー7点)で過剰結論・帰属誤り・審判不一致を指摘され、全面改訂に至ったこと。

## 規律6点

1. **調査文書の結論をそのまま設計根拠にしない**
2. **独立した反対側レビューで再判定する** — 事実(一次資料で確認できるか)・転移条件(同じ失敗条件がこのプロジェクトにあるか)・因果(効果の帰属は正しいか)・より小さい対策(境界を公開しない/ホスト側に閉じる選択はないか)
3. **反例未探索なら「仮説と整合する事例」に留める** — 「裏付けられた」「証明された」を書かない
4. **公約は保証意味を分解し、対応する審判セットと有効化条件が揃うまで外向き化しない**
5. **機能正当性・互換性・供給網信頼・安全性を別々に評価する** — 「機械検証可能」に畳み込まない
6. **元調査と反対側レビューを必ず併読する** — ゲート・仕様へ採用する時は判定語(採用/縮小/延期/棄却)を併記する

## 運用注

- 出典は**再確認可能な公開恒久文書**(公式仕様・RFC・公式ブログ・学会誌・バグトラッカ)に限定する。調査ワークフローの「検証済み」申告や、出典URLの無い歴史詳細を根拠にしない
- 判断が割れたら「**ユーザーデータまたは公開契約へ不可逆に焼くかどうか**」で決める。焼かない選択が可能なら、v1では小さい方を選ぶ(反対側レビューの判定基準)
- LLM能力への言及は日付を添える。能力仮定は契約・スキーマ・ゲートに焼かず、日付+見直しトリガー付きで運用文書にのみ書く

## 登録規則(2026-07-19制定)

制定の経緯: 入口台帳([docs/README.md](../README.md))のファイルマップから36件のreview文書が欠落し、既決事項(例: [AM式高度イージング=区間補間の非破壊差し替え、2026-07-10決定](../concept.md))が後続作業から逆引きできず、モック・仕様に旧仕様が混在した。

1. **新しいreview文書を作ったら、同じ変更で下の全文書索引に1行追加する**。入口台帳のファイルマップは「現役で参照される文書」の抜粋であり、全量はこの索引が正本
2. **ユーザー決定・採否・撤回・未統一を含む文書は、[決定逆引き台帳](../decision-index.md)にも主題キーワードつきで1行登録する**。会話・commit履歴・エージェントセッションにしか残らない決定を作らない
3. 状態語彙は固定集合とする: **決定 / 縮小採用 / 延期 / 棄却 / 撤回 / 未統一 / 観察 / 比較中 / 停止線**。この語彙の外の状態表現を新設しない(必要なら本規則を先に改訂する)
4. `scripts/check-docs.sh` が索引の抜け・入口台帳の重複掲載・ローカルリンク切れ・状態語彙を機械検証する。docsを触る変更では実行してから終える

## 全文書索引

各文書の1行要旨と状態は文書冒頭が正本。ここはファイル名と表題のみ(抜け検出用の全量索引)。

| ファイル | 表題 |
|---|---|
| [2026-08-19-normal-timeline-prior-art.md](2026-08-19-normal-timeline-prior-art.md) | **普通のタイムラインが持つ操作の先例調査**(AE/Premiere/Resolve/CapCut/AviUtl/Ableton/Reaper 7製品、観察)。必須判定12件は既にMotolii既決またはコード実装済み。決定的な欠落は個別操作ではなく「track/laneがgapless packingか自由配置か」という土台の未選択で、trim family(ripple/roll/slip/slide)はtrack-based系だけが持つ操作族と判明。プリコンポ回避(AE痛点)はグループ化+foldで裏付け済み、A/V link-unlinkはMotoliiの単一soundtrackモデルと前提が異なる可能性 |
| [2026-08-19-timeline-packing-model-decision.md](2026-08-19-timeline-packing-model-decision.md) | **利用者裁定**: Timeline の土台は AE 型の自由な絶対時間配置。gapless packing 前提の trim family(ripple/roll/slip/slide/insert/overwrite/lift/extract/sync lock)は**設計上の除外**で漏れではない。「以降を押し出す」は便利機能として先送り | **決定**(2026-08-19) |
| [2026-08-17-composition-resolution-and-aspect-fit-decision.md](2026-08-17-composition-resolution-and-aspect-fit-decision.md) | **出力解像度はCompositionが所有し、素材はfit(contain)で受ける**。現行exportの「最初のvideo asset解像度」導出はM1名残で、縦動画が拒否される。`Composition.resolution: Option`(None=互換)+新Command+既定1920x1080+letterbox。blur-fillはv1.x粒2 |
| [2026-08-17-edit-during-playback-decision.md](2026-08-17-edit-during-playback-decision.md) | **再生中の編集は再生を止めない(Ableton型)**。音はsoundtrackのみ由来で編集は影響せず、絵は毎フレームsnapshot読み直しのため停止が不要。playhead手動移動とSetSoundtrackだけsessionを開き直す。oracleは再生継続テストを操作family毎に1本 |
| [2026-08-18-session-handoff-normal-editor-campaign.md](2026-08-18-session-handoff-normal-editor-campaign.md) | **「普通の動画編集ソフト」campaign第1区切りの引き継ぎ**(セッションID記載)。merge済み17レーン、機械検証と人間未検証の線引き、既知欠陥・残タスクの正本、運転規約。remote未push |
| [2026-08-18-cli-gui-driver-seat.md](2026-08-18-cli-gui-driver-seat.md) | **CLI→GUI運転席の決定**。ShellTranscript(言う場所は1つ・stderr黙殺全廃)+ScriptedPrompts(dialog台本化)+DrivenShell(egui_kittestでheadless駆動)。合格条件はred先行のdrive_tests/shell_error_fenceに固定 | **決定**(2026-08-18) |
| [2026-08-18-first-real-run-observations.md](2026-08-18-first-real-run-observations.md) | 実素材での初通し実走(CLI鎖+GUI screenshot)。placeがclip長をcomp長にする・exportの報告(300frames)と現物(178)の不一致・Stage斜め視点の再確認・thumbnail失敗のstderr黙殺を発見 | **観察**(2026-08-18) |
| [2026-08-18-rerun-as-composition-foundation.md](2026-08-18-rerun-as-composition-foundation.md) | **利用者裁定: Rerunは合成のメイン基盤**。AEカメラレイヤー相当をdocument cameraとしてRerunへ外注し、ビューとエクスポートを同一シーンで撮る。成立条件はE0 probe(offscreen/カメラ注入/遮蔽)の実測待ち | **決定**(2026-08-18) |
| [2026-08-18-iced-reentry-survey.md](2026-08-18-iced-reentry-survey.md) | iced 0.14の実態調査(段差ゼロ軸)。replayは公式不変量(iced_test/time-travel)で軸上は本物、だがwgpu版不一致でRerun基盤と衝突・AccessKit未統合・NLE先人ゼロ。繋がり3分類の現状マップ=「繋がっていない」0件 | **観察**(2026-08-18) |
| [2026-08-18-user-first-touch-observations.md](2026-08-18-user-first-touch-observations.md) | 利用者の初回タッチ観察。Browserダブルクリック=処理不在の無反応(Q0違反)、画像はadmission拡張子リストで入口拒否(台本と乖離)、透明パネル=空コンポの正対プレート。「繋がっていない」は引き続き0件。修正レーン2本発注 | **観察**(2026-08-18) |
| [2026-08-18-external-ux-diagnosis.md](2026-08-18-external-ux-diagnosis.md) | **外部LLM診断(検収済み・10/10 CONFIRMED)**。B無反応=F-02死んだ3面/F-03 M・S見た目だけ、C入口=F-01再起動で続きが開かない、D黙殺=F-07〜F-10(timeline status・Stage GPU失敗・thumbnail・export spawn)、E台本乖離=F-04矢印/F-05 Export | **観察**(2026-08-18) |
| [2026-08-18-log-and-structure-enforcement.md](2026-08-18-log-and-structure-enforcement.md) | **利用者裁定: ログと構造の強制が組めればeguiはicedになれる**。UiIntent journal(原因のログ)+replay oracle常設+単一ゲートウェイ+フェンスで、icedのElm性質を自前強制する。診断D類=ログ欠け・B類=構造欠けとして修正waveを束ねる | **決定**(2026-08-18) |
| [2026-08-18-iced-host-migration-decision.md](2026-08-18-iced-host-migration-decision.md) | **利用者裁定: ホストをicedへ乗り換える**(DX実測342vs1,415行・draw内書換0vs97が決定打)。絞め殺し方式M-0〜M-5、UiIntent背骨とRerun島は持ち越し、egui shellは並走→`--legacy`。fork 2本体制のコストを明記 | **決定**(2026-08-18) |
| [2026-08-18-session-handoff-ux-driver-seat-and-iced-migration.md](2026-08-18-session-handoff-ux-driver-seat-and-iced-migration.md) | **本セッションの引き継ぎ**(ID記載)。運転席→実走→Rerun基盤→診断10件全着地→iced移行M-0完了+M-1走行中。fork 2本push済み・Motolii repo未push・検証の実態と残作業の正本 | **引き継ぎ**(2026-08-18) |
| [2026-08-18-session-handoff-iced-four-pane-campaign.md](2026-08-18-session-handoff-iced-four-pane-campaign.md) | **第3区切りの引き継ぎ**(ID記載)。main一本化→vism発掘回収→theme/M-2/widgets着地・Inspector/Browser/Timeline検収済み未merge・統合第1弾(sonnet)走行中。full gateは1c76140e以降未通過・要裁定4件・レーンはsonnet明示の規約 | **引き継ぎ**(2026-08-18) |
| [2026-08-18-iced-fork-seam-ledger.md](2026-08-18-iced-fork-seam-ledger.md) | iced fork(branch `motolii/host-seams`、上流 pin `3de45144`)が上流とどこで乖離しているかの台帳。seam 2件=`web-sys` 完全一致釘打ちの解除(Rerun `re_renderer` の `js-sys` と衝突する)と、`iced_wgpu::device_limits`(`max_bind_groups` の床。既定は上流と同じ 2)。上流fileへの実質差分は +12/-3。再適用手順・上流PR候補である旨・**seam 2 の効きを見る oracle がまだ無いこと**(M-2 の受け入れ条件)まで書いてある | **観察**(2026-08-18) |
| [2026-08-18-rerun-embedding-precedent-survey.md](2026-08-18-rerun-embedding-precedent-survey.md) | Rerun埋め込みの前例外部調査。eframe埋め込みは公式文書化+example 3系統CI維持だが「毎リリース壊れる」明記・semver保証なし(月次12 minor)。実在例=rewire-run/viewer(0.34→0.36追従実績)とdepthai-viewer(fork追従を怠りarchive)、`re_viewer`逆依存は本体のみ。評定=「舗装されているが通行者がほぼいない道」、offscreen非eguiホストは前例ゼロだが外れるのはホスト層1枚のみ。rev固定+seam台帳が合理的な唯一の防御 | **調査**(2026-08-18) |
| [2026-08-18-iced-track-record-survey.md](2026-08-18-iced-track-record-survey.md) | icedの出荷実績・維持体制の外部調査。COSMIC半年運用(pop-os fork上)・Kraken商用・Sniffnet/Halloy追従、離脱ポストモーテムは発見できず=「forkして留まる」が観測パターン。1.0は遠い(作者明言)・AccessKit外部PRを作者が同日クローズ・IMEは0.14初出荷でWindows #3189放置・focus未整備・バス係数1。評定=迂回を要求する発見ゼロ、不安は全て「fork内で持つコードが増える」種でseam台帳運用の延長。ただし小forkは小のままでは済まない(Windows IME/focus系) | **調査**(2026-08-18) |
| [2026-08-18-stage-interaction-concept-map.md](2026-08-18-stage-interaction-concept-map.md) | Stage対話の概念地図(M-2実測後の裁定材料)。Rerunにauthoring概念(ギズモ・スナップ・書き出し枠・undo)は来ない=文書側(UiIntent→Document)に置きStageは投影に徹する。ギズモ3部品=入力所有権(ブリッジが握る)/hit数学(`canonical_drop_from_ndc`既存)/絵とドラッグ意味論(新規、transform-gizmo crate候補)。柵=ギズモをforkの機能にしない(seamのみ)。2台カメラ=視点カメラ(camera seat)/書き出しカメラ(document概念、export経路実行)は全DCCが解いた標準パターン。提案(未決)=M-2受入条件に入力調停3状態 | **観察**(2026-08-18) |
| [2026-08-19-egui-timeline-capability-ledger.md](2026-08-19-egui-timeline-capability-ledger.md) | egui Timeline(`timeline_editor/`9,059行)を利用者の1操作単位で48行の台帳にし、iced側(`timeline/`+`shortcuts.rs`)の有無と突き合わせた。無28件・部分4件・有16件。危険候補1位=ロック中clipのドラッグがicedでは無警告で動いて見え release後に無言で戻る(D2はロックを検査しないためUI層のhit_test/mouse_interactionが漏れている)、2位=拒否理由(`take_rejections`)がiced側のどこにも表示されない | **観察**(2026-08-19) |
| [2026-08-17-soundtrack-command-decision.md](2026-08-17-soundtrack-command-decision.md) | `Command::SetSoundtrack { old, new }`を`Document.soundtrack`の唯一の書き込み経路として決定。apply時UnknownAssetId拒否、singleton merge key、v3-only journal decode。N-SOUNDTRACK-WRITEを閉じる |
| [2026-08-15-blitz-ui-runtime-adoption-proposal.md](2026-08-15-blitz-ui-runtime-adoption-proposal.md) | UI基盤をBlitz(HTML/CSS)+テクスチャ合成へ移す起案。処分すべき既決3件(RN再基線・Skia ADOPT・Web窓)と未了6件を固定。比較中・裁定未記入 |
| [2026-08-15-blitz-ui-runtime-probe.md](2026-08-15-blitz-ui-runtime-probe.md) | Blitz(HTML/CSS)をUI基盤候補として実測。自前wgpu29テクスチャへの描画はPASS、キーはフォーム要素にしか届かない等の制約と版の罠を記録。比較中・裁定未了 |
| [2026-08-16-daw-playhead-follow-prior-art.md](2026-08-16-daw-playhead-follow-prior-art.md) | **再生中に窓をどう動かすかを、Ardour(GPL)・LMMS(GPL)・Ableton Live 11の実装で確認した観察**。追従の型は「ページ送り」と「中央固定」の2つだけで、**自作した「相対位置を保つ」は先例に無い**(窓が止まる時間がゼロになる)。ページ送りは**ページの内側では1pxも動かない**(Ardour `editing_context.cc:3746`)、前進時の着地は**playheadが新しい左端**(次の送りまでが最長)、瞬間移動は避けるのではなく**600msかけて見せる**(LMMS `SongEditor.cpp:727`)、ドラッグ中は追従しない(同`:1570`)。GPLのためコードは持ち込まずPATTERNのみ。**実装は同日に撤廃** — 利用者の違和感の対象は追従ではなく目盛の明暗で、Abletonの窓の挙動は調査前の実装と同じだった。症状から原因を推測して調査に走った失敗の記録でもある |
| [2026-08-16-timeline-runtime-reselection-to-egui.md](2026-08-16-timeline-runtime-reselection-to-egui.md) | **Timelineの実行時基盤をeguiとし、同日の`timeline_blitz`正本決定を撤回**(実測つき)。HTML/CSSモックはUXの台本、Blitzはビルド時のコンパイラへ役割変更。DOM仮想化の天井=可視2,000ノード、Blitz betaの制約4件(z-index/resolve2回/taffy丸め/transition)、egui+`egui_taffy`で構造成立・hidpiはタダ・フォント同梱必須、`mock_tokens`で定数129本と可変60件を分離。`timeline_egui`961行は`input.rs`とテスト339行だけ回収。**同日に`document_edit_runtime`と`timeline_intent_adapter`が削除され、C2の繋ぎ先も作り直しになった**(両端は無傷、`21cb8204^`から読める) |
| [2026-08-16-blitz-html-css-authoring-and-validation-decision.md](2026-08-16-blitz-html-css-authoring-and-validation-decision.md) | Blitz panelは通常のHTML/CSSで設計し、browser previewで視覚設計、固定crate版dumpで製品採用を確認する決定。JS・入力・Document/D2・dock・Stageは移さない |
| [2026-08-16-blitz-timeline-authority.md](2026-08-16-blitz-timeline-authority.md) | **Timelineの正本は`timeline_blitz`**(利用者裁定)。Skiaは意味/hit/oracle源として残置(呼び手なし)。ホストがeguiなのはRerun Viewerがeguiウィジェットである構造上の帰結で、ドッキングをCSSへ移す案は成立しない。clip/keyはcustom widget 1ノード(C3完了)、行高20px固定（2026-08-16: **実行時基盤はeguiへ再選定され撤回**。P8実測とドッキング=eguiの帰結は残る） |
| [2026-08-16-web-window-and-rn-product-fold.md](2026-08-16-web-window-and-rn-product-fold.md) | Web窓(wry)とRN製品面を畳む決定。約40,657行撤去。2026-08-14 Web窓projectionを撤回。`ui/motolii-rn/src`のTSは移植元として残す。**Timeline座席は空席・製品は不在**になった |
| [2026-08-16-skia-timeline-authority-correction.md](2026-08-16-skia-timeline-authority-correction.md) | 製品Timelineの正本はSkia(`timeline_skia_raster`)。eguiは製品から1箇所も呼ばれていない残骸で、旧eguiアプリごと3,338行撤去。参照グラフで判定 |
| [2026-08-15-egui-timeline-engine-authority.md](2026-08-15-egui-timeline-engine-authority.md) | 製品Timeline engineはMotolii egui。Godot PORT操作、egui-keyframeはPATTERNのみ。Rerun Time Panel相乗りは撤回（2026-08-16: engine正本はSkiaへ訂正。Rerun非相乗りは維持） |
| [2026-08-14-rerun-body-skin-meaning-decision.md](2026-08-14-rerun-body-skin-meaning-decision.md) | Timelineに限る。Rerun Time Panelのスキン＋Skiaのlayer／clip／key意味（2026-08-15: engine相乗りは撤回注記） |
| [2026-08-14-web-window-product-reflection-authority.md](2026-08-14-web-window-product-reflection-authority.md) | Web窓／macOS窓／Windows窓を同じ製品coreとsnapshotのprojectionとし、platform adapterを薄く保つ決定（2026-08-16: **撤回**。wry経路ごと撤去） |
| [2026-08-13-godot-editing-port-handoff.md](2026-08-13-godot-editing-port-handoff.md) | Godot編集系PORTのdirty実装状態、Release確認入口、未接続の`[` `]`、Debug誤起動を次sessionへ渡す観察 |
| [2026-08-13-godot-editing-system-adoption.md](2026-08-13-godot-editing-system-adoption.md) | Timeline／key／Inspector操作をGodot MIT editorからPORTし、トンマナは現行Motoliiのまま維持する決定 |
| [2026-08-13-inspector-key-add-ux-decision.md](2026-08-13-inspector-key-add-ux-decision.md) | Riveのproperty key buttonを既存Document/D2 intentへ縮小採択し、Vec2一button、snapshot-owned tri-state、同一値行、reject非遷移を固定する決定 |
| [2026-08-10-creator-translation-working-draft-pr-integration-decision.md](2026-08-10-creator-translation-working-draft-pr-integration-decision.md) | Motoliiをクリエイター意図の翻訳Hostとし、理論を通す叩き台、良い塊のPR、機械的conflict許容とsemantic conflict停止を定める決定 |
| [2026-08-10-m5-glow-multipass-hdr-transient-proof.md](2026-08-10-m5-glow-multipass-hdr-transient-proof.md) | M5-R0でFP16 bright-pass、separable blur、additive composite、Host所有transient再利用とalpha／extent負例を確認する縮小採用 |
| [2026-08-10-m5-feedback-trail-host-ping-pong-proof.md](2026-08-10-m5-feedback-trail-host-ping-pong-proof.md) | 再帰FeedbackをStatefulFilterにせずHost所有2 textureの明示clear／ping-pong／fresh replayで成立確認し、SCR-4のWAITを維持する決定 |
| [2026-08-10-m5-datamosh-codec-domain-private-proof.md](2026-08-10-m5-datamosh-codec-domain-private-proof.md) | FFmpeg標準packet dropで固定MP4のkey packetだけを除去し、再生可能出力、決定性、元asset不変を確認したcodec-domain private proof |
| [2026-08-10-m5-planar-gradient-path-clip-mask-rerun-proof.md](2026-08-10-m5-planar-gradient-path-clip-mask-rerun-proof.md) | 2D平面gradientをPath coverageで切り抜く融合passをクリッピングマスク最小例としてRerun確認し、中間mask textureとhalftoneの再入場条件を分ける決定 |
| [2026-08-10-lottie-path-modifier-candidates-and-rerun-proof.md](2026-08-10-lottie-path-modifier-candidates-and-rerun-proof.md) | 固定lottie-web実コードからAE由来Path modifier候補を分類し、Pucker/Bloatのcubic handle式訂正とZig Zag burstのRerun実画面proofを記録する縮小採用 |
| [2026-08-10-m5-path2d-rerun-custom-visualizer-probe-and-dispatch-route.md](2026-08-10-m5-path2d-rerun-custom-visualizer-probe-and-dispatch-route.md) | M5最初の可視成果をz=0 Path2DのRect／Circle／source-overとし、Rerun custom visualizer proof、probe限定事項、次のRN Stage seat compile発注capsuleを固定する決定 |
| [2026-08-10-m5-rerun-spatial-viewer-adoption-reclosure-decision.md](2026-08-10-m5-rerun-spatial-viewer-adoption-reclosure-decision.md) | 固定Rerun実コードのView／camera／visualizer／wgpu／picking閉包を確認し、M5 spatial主部を機構別PATTERNからSpatial Viewer subsystemのADOPT／WRAPへ再締結する決定 |
| [2026-08-11-m3-m5-stage-hero-projection-root-decision.md](2026-08-11-m3-m5-stage-hero-projection-root-decision.md) | M5由来のRerun StageをHero consumerとし、Document／D2 authorityを維持したままaccepted snapshotからStage、Timeline、Inspectorへ意味を回収するPR前の根本決定 |
| [2026-08-10-m3-map-node-state-measurement.md](2026-08-10-m3-map-node-state-measurement.md) | M3実行地図の全54 nodeをcurrent codeへ照合し、状態語の乖離8件を特定して更新した実測。WIRED 5／BUILT_UNWIRED 19／PARTIAL 16／ABSENT 10／EXTERNAL 4 |
| [2026-08-09-supervisor-handoff-pr-operation-first-wave.md](2026-08-09-supervisor-handoff-pr-operation-first-wave.md) | PR運用の初merge 2件とclose 1件、4 lane並列waveの実測コスト、「前提を疑えるか」を軸とした総監督席のClaude移管理由とCodexとの役割分離、未記録だった読み口分離線とGPU群抽出、intent 2種の欠落だけがR1を止めている実測を次のsupervisorとCodexへ渡す引継ぎ |
| [2026-08-09-chain-gate-results-and-audio-path.md](2026-08-09-chain-gate-results-and-audio-path.md) | 仮コード未通過6区間へ鎖のgateを掛けて全区間NEEDS_REVISIONを確定し、完成条件の音声経路が書き出し側・取り込み側とも実装済みで未配線(BUILT_UNWIRED)であることと、完成条件を塞ぐ8件を特定した観察 |
| [2026-08-10-stage-hit-test-missing-group-transform.md](2026-08-10-stage-hit-test-missing-group-transform.md) | Stageのhit-test経路の world にグループ変形継承が入っておらず、変形を持つグループ内の子で描画位置とhit領域がずれるfinding。該当testなし。未処分 |
| [2026-08-10-wire-carries-layer-identity-only.md](2026-08-10-wire-carries-layer-identity-only.md) | RN wireがlayer粒度のidentityしか運ばず、keyframe/effect/paramを指すintentはhost実装が正しくても撃てないfinding。R2編集4 intent中3本が該当。未処分 |
| [2026-08-10-group-transform-bounds-draft2.md](2026-08-10-group-transform-bounds-draft2.md) | グループのtransform boundsを子孫幾何のcanonical union と定め、エフェクトは広げず、非表示の子はvisibility:hidden側として寄与させる第2案。W3C SVG2仕様とコード実測に基づく。未採択 |
| [2026-08-10-group-transform-bounds-draft.md](2026-08-10-group-transform-bounds-draft.md) | グループのtransform handleが囲む矩形をcomposition寸法ではなく子の合成範囲とし、Unknownは全域fallback、空グループは全域へ倒さないとする起草。未採択・反対側レビュー未実施 |
| [2026-08-10-group-drag-call-site-sketch.md](2026-08-10-group-drag-call-site-sketch.md) | グループrootを掴んで動かす鎖の仮コード。???8件のうち本当に未決は group bounds契約と group root選択の2件で、前者が pivot・handle・snap・dirty領域を含む6依存の要石であることを示した |
| [2026-08-09-m3-r0-product-runtime-seat-acceptance.md](2026-08-09-m3-r0-product-runtime-seat-acceptance.md) | R0-HOST／MAC-SEAT／STAGE-LIFECYCLEを責任別に再照合し、通常RN Release artifactのread-only起動でR0-ACCEPTをDONEとした受入 |
| [2026-08-10-m3-collaborative-bringup-decision.md](2026-08-10-m3-collaborative-bringup-decision.md) | R0後のM3を全surface一括統合待ちから、起動可能なRN製品artifactへStage／Timeline／Browser／Inspectorを継続統合する共同開発bring-upへ切り替える決定 |
| [2026-08-09-unified-parallel-start-baseline-decision.md](2026-08-09-unified-parallel-start-baseline-decision.md) | 製品main、現行authority、直列核、UI配置逃げ道、仮コード調査、未commit設計資料を一つの開始履歴へ収束し、INTEGRATED／CANDIDATE／WAIT／REJECTEDを混同しない全体並列開始baseline決定 |
| [2026-08-09-cold-replaceable-supervision-failure-containment-decision.md](2026-08-09-cold-replaceable-supervision-failure-containment-decision.md) | authorityを一つのtop seatへ保ったままfresh sessionへcold replacementし、総監督停止、二重権威、base drift、衝突、reviewer mutation、user STOPを非LLM failure injectionで封じ込める決定 |
| [2026-08-09-stage-pointer-miss-clears-primary-oracle.md](2026-08-09-stage-pointer-miss-clears-primary-oracle.md) | layer単位の特異transformをUnavailable化した結果到達したStage空き領域clickのprimary解除を、実装済み未固定の挙動から明示oracle 2本へ昇格した決定 |
| [2026-08-09-single-writer-guard-test-exemption-decision.md](2026-08-09-single-writer-guard-test-exemption-decision.md) | `&mut Document` deny走査を製品moduleでは無条件に保ちつつ、`#[cfg(test)]` fixtureだけ理由必須の同一行マーカーで除外し、除外件数を常に出力する精密化決定 |
| [2026-08-08-m4-m5-call-site-connection-sketch.md](2026-08-08-m4-m5-call-site-connection-sketch.md) | M4／M5全機構をM3の共有render背骨へ仮接続し、最小Core＋private sandbox境界、authority段8 lane／foundation段12 lane、共有ownerによる4列publication、既存STOPごとのsafe parallel edgeを抽出した観察 |
| [2026-08-08-depth-rail-selection-focus-decision.md](2026-08-08-depth-rail-selection-focus-decision.md) | z=0既定群の灰色統合と「個別化=逸脱」新規則、選択フォーカスでdragして初めて視差が生まれるDepth Rail設計の利用者裁定、7案却下の経緯と10方向実声調査の返却 |
| [2026-08-08-supervisor-handoff-timeline-design-and-return-to-codex.md](2026-08-08-supervisor-handoff-timeline-design-and-return-to-codex.md) | Timeline設計12件と完成条件の鎖、skia fixture 7本、起動準備だけ済ませた外部発注2件、Codex復帰による代理supervisor席の返上を次のsupervisorへ渡す引継ぎ |
| [2026-08-08-timeline-design-decisions-and-skia-fixtures.md](2026-08-08-timeline-design-decisions-and-skia-fixtures.md) | キーframe方式維持(値域が確定しないため)、行高固定・最小(縦が情報を持たないため)、object bar読み取り専用(誤爆コストの非対称)、畳み＝射影、逸脱時のみ表示、glyphは形で示す、をskia fixtureで確認した設計決定とAbleton実測 |
| [2026-08-08-completion-condition-call-site-sketch.md](2026-08-08-completion-condition-call-site-sketch.md) | 完成条件(3〜5分・音楽同期・音声mux)を1本の鎖として書き、音声mux実装済みに対して楽曲bedのSoundtrack編集とAsset登録を同一製品操作へ閉じるatomic境界が無いこと(`N-SOUNDTRACK-WRITE`)を検出した観察 |
| [2026-08-08-mascot-and-pet-decision.md](2026-08-08-mascot-and-pet-decision.md) | マスコットを8×8・2色のカササギへ確定し、ライト背景では体色を反転、動きは立つ／沈む／浮くの3フレームで賄う決定と、マスコット的装飾規律との衝突によるペット機能の延期 |
| [2026-08-07-m3-integration-zone-value-update.md](2026-08-07-m3-integration-zone-value-update.md) | M3を「UIを作る工程」から「先に作った資産を接続する統合ゾーン」へ読み直し、既定推定を`BUILT_UNWIRED`、sort keyをconcept.mdの完成条件、記録↔コードdriftを一級riskとする価値観更新 |
| [2026-08-08-source-shortfall-ask-before-remap-decision.md](2026-08-08-source-shortfall-ask-before-remap-decision.md) | 素材不足時の`OverrunMode`既定をFreezeのまま維持し、引き伸ばし／Loopは利用者へ問うて明示選択時のみ有効とする決定 |
| [2026-08-08-group-bake-chain-and-gap3-root.md](2026-08-08-group-bake-chain-and-gap3-root.md) | Group Bake(プリコンポ代替)の仮コードでM4着地先が一つも実在しないことを確認し、阻害の根がGAP-3(同一性format未締結)であると特定した観察 |
| [2026-08-08-call-site-sketch-artifacts.md](2026-08-08-call-site-sketch-artifacts.md) | 仮コード成果物の保全(非authority/非compile)。背骨はgate通過・修正済み、他6区間はgate未通過のまま |
| [2026-08-08-gate-effectiveness-measurement.md](2026-08-08-gate-effectiveness-measurement.md) | 鎖のgateは1回で12件(うち施工不能なseam 4件)を検出し回収する一方、capsuleのgateはv1 12件→v2 9件/再指摘7件で1周では収束しないという実測 |
| [2026-08-08-skia-reject-to-adopt-authority-reconciliation.md](2026-08-08-skia-reject-to-adopt-authority-reconciliation.md) | 2026-07-21/27のSkia`REJECT`と2026-08-07再基線の衝突を裁定し、Vello退役により前提が消滅したとして旧`REJECT`を撤回、alpha・色の懸念は維持する |
| [2026-08-08-n-overlay-dependency-gate.md](2026-08-08-n-overlay-dependency-gate.md) | rust-skia overlay renderer導入の依存ゲート7段通過記録と、`references.md`にskia項目が存在しない記録層欠落の指摘 |
| [2026-08-08-gizmo-known-implementation-preflight.md](2026-08-08-gizmo-known-implementation-preflight.md) | gizmo機構の既知実装調査と製品先例、`transform-gizmo`系候補の一次資料事実、未確認4件により`BUILD JUSTIFICATION`未確定で実装発注不可とするpreflight |
| [2026-08-08-supervisor-handoff-integration-map-and-instrument.md](2026-08-08-supervisor-handoff-integration-map-and-instrument.md) | 統合地図の実測再構築、M3価値観更新、仮コード器具の3段運用、Stage×M5判定、リポジトリ外資産による判定訂正を次のsupervisorへ渡す引継ぎ |
| [2026-08-08-out-of-repository-asset-inventory.md](2026-08-08-out-of-repository-asset-inventory.md) | node surveyがリポジトリ内しか見ない限界と、リポジトリ外の隔離probe実在による`N-OVERLAY`判定訂正(`ABSENT`→`PROBE_ONLY`)、`ABSENT`判定前の外部確認手順 |
| [2026-08-08-call-site-sketch-seams-and-stage-m5-verdict.md](2026-08-08-call-site-sketch-seams-and-stage-m5-verdict.md) | 仮コード7区間を1本へ統合して検出した継ぎ目の合成失敗9件と、Stage×M5迎え入れ判定(絶対規律2/6は成立、Provider席は実在、不足はC0-Schema1つ) |
| [2026-08-07-call-site-sketch-composition-failures.md](2026-08-07-call-site-sketch-composition-failures.md) | 仮コード合成テストで検出した決定間の合成失敗14件と、作者・配布枝7件が同一の欠落(runtime identity/installation path不在)へ収束する観察 |
| [2026-08-07-supervisor-handoff-map-rebuild-and-spine.md](2026-08-07-supervisor-handoff-map-rebuild-and-spine.md) | 62項目の実測による地図再構築、背骨4粒のlocal branch到達、新規node 3件、仮コード器具の初回運用実績を次のsupervisorへ渡す引継ぎ |
| [2026-08-07-call-site-sketch-findings-return.md](2026-08-07-call-site-sketch-findings-return.md) | 仮コード照合で出たReadOnlyNewer admission不成立とUndoがasset登録を巻き戻さない疑いを、修理許可でないfindingとして前ownerへ返却する |
| [2026-08-07-provisional-call-site-sketch-instrument-decision.md](2026-08-07-provisional-call-site-sketch-instrument-decision.md) | 背骨を呼び出し側から先に書き、実名で埋まらない箇所を欠落として露出させる非compile器具の規約と、survey `ABSENT`との相互検証条件、並列化の上限 |
| [2026-08-07-r2-rn-transient-time-seat-decision.md](2026-08-07-r2-rn-transient-time-seat-decision.md) | RN Hostへ背骨に必要な最小のtransient評価時刻席だけを置き、`R2-FOCUS-PLAYHEAD-AUTHORITY`と旧`EditorPlayhead`流用は未決のまま維持する縮小採用 |
| [2026-08-07-r2-stage-geometry-read-projection-decision.md](2026-08-07-r2-stage-geometry-read-projection-decision.md) | Stage幾何を`TARGET_MISSING`から`REMAP`へ訂正し、AABBでなく(正準局所rect + world/camera Affine2D)をRect source限定・可視時刻限定で投影する縮小採用 |
| [2026-08-07-m3-supervisor-handoff-stage-to-gizmo.md](2026-08-07-m3-supervisor-handoff-stage-to-gizmo.md) | Stage初回pixelsのlocal main到達、gizmo直行を止めたidentity gap、Browser回り道候補の未採用処分を次のfresh supervisorへ渡す引継ぎ |
| [2026-08-07-terra-grok-composer-role-reallocation-decision.md](2026-08-07-terra-grok-composer-role-reallocation-decision.md) | Terraをbounded order compile、Grok 4.6 medium／high／xhighを通常・複雑・長時間agenticなbounded施工へ再配置し、Composer 2.5を明示理由のある代替施工に保つ決定 |
| [2026-08-07-codex-spark-cli-smoke-observation.md](2026-08-07-codex-spark-cli-smoke-observation.md) | `gpt-5.3-codex-spark`の現行Codex CLI起動、JSONL、usage、4.864秒完了と小context向け運用境界を記録する観察 |
| [2026-08-07-outcome-order-compilation-and-research-return-loop.md](2026-08-07-outcome-order-compilation-and-research-return-loop.md) | 利用者成果からclosed orderをcompileし、実装／調査返却後にcurrent codeから次edgeを再選定する横断発注ループ |
| [2026-08-07-m3-baseline-required-autonomy-checkpoint.md](2026-08-07-m3-baseline-required-autonomy-checkpoint.md) | M3 baseline必要性の自動承認、非OpenAI抽出、別family challenge、Codexの整理・写像・採否責任、Web調査再入場条件を分離するcheckpoint決定 |
| [2026-08-07-m3-react-native-rust-skia-runtime-rebaseline.md](2026-08-07-m3-react-native-rust-skia-runtime-rebaseline.md) | M3標準UIをReact Native shell、rust-skia Timeline／Curve、wgpu + rust-skia Stageへ再基線化する決定 |
| [2026-08-09-ui-placement-deferral-staging-surface-decision.md](2026-08-09-ui-placement-deferral-staging-surface-decision.md) | UI配置だけが未決のclosed controlを既存routeのまま一時配置し、並列接続を止めないHost-owned staging surface決定 |
| [2026-08-08-serial-core-known-contracts-decision.md](2026-08-08-serial-core-known-contracts-decision.md) | M3/M4/M5直列核 — Asset lifecycle、source/recipe identity、resource/artifact/job、mutation/invalidationの既知実装採択決定 |
| [2026-08-03-p12-c1-document-lifecycle-adoption-decision.md](2026-08-03-p12-c1-document-lifecycle-adoption-decision.md) | Desktop文書ライフサイクル採択決定: P12-C1で既知セーブ意味論を仕様化し、ReadOnlyNewer/Close/Save As/rfdの未接続境界を固定 |
| [2026-08-02-m4-known-implementation-survey.md](2026-08-02-m4-known-implementation-survey.md) | M4のcache、resource、disk artifact、区間、background job、proxy、SVGを具体APIと非証明範囲へ割り当てる既知実装比較 |
| [2026-08-02-m4-disk-artifact-store-resurvey.md](2026-08-02-m4-disk-artifact-store-resurvey.md) | cacache棄却後のdisk候補を再検索し、global CASを現行authority外としてverified recipe artifact storeへ縮小採用 |
| [2026-08-02-m5-known-implementation-survey.md](2026-08-02-m5-known-implementation-survey.md) | M5の3D math、glTF、wgpu depth、bounds、text、deterministic identityを既存ownerへ接続する既知実装比較 |
| [2026-08-02-vism-entrance-parallelization-root-map.md](2026-08-02-vism-entrance-parallelization-root-map.md) | Vismの意味の根と並列施工の根を分け、Filter／Source／Data／Path／Text／Instance／Simulation／Automation等の入口を既存owner、route、STOPへ接続する§8.1全体レビュー |
| [2026-07-26-third-party-sustainable-economy-decision.md](2026-07-26-third-party-sustainable-economy-decision.md) | 無料／有料、OSS／proprietary等を作者が選べる持続可能な経済圏を、Motoliiがmarketplaceを所有せず支える理由、責任境界、停止線 |
| [2026-07-26-vism-malware-containment-handoff.md](2026-07-26-vism-malware-containment-handoff.md) | Vism悪性コード封じ込めをcatalogから分離し、現行事実、攻撃面、必須負例、合格条件、STOPを他LLMへ渡すread-only締結依頼 |
| [2026-07-26-vism-malware-containment-contract-decision.md](2026-07-26-vism-malware-containment-contract-decision.md) | 悪性Vismのambient authority 0、hard budget、typed failure、bounded recovery、atomic install、13負例とclaim boundaryを締結し、runtime／schema実装はSTOPする意味論決定 |
| [2026-07-27-vism-authoring-journey-decision.md](2026-07-27-vism-authoring-journey-decision.md) | 推奨pass shape／標準operation候補と独自WGSL、自己完結shader closure、source authority／backend派生物、v1 source fork／v2 local Vism、Kit selection closureを分ける作者journey比較 |
| [2026-07-28-vj-multistream-video-prior-art-observation.md](2026-07-28-vj-multistream-video-prior-art-observation.md) | 40動画＋合成後Glowをdecode／surface／coverage／post effectへ分け、VJのGPU-native media、hardware decode、YUV直結、Glow pyramidをMotoliiへ移す比較と停止線 |
| [2026-07-29-decode-to-composite-premise-audit.md](2026-07-29-decode-to-composite-premise-audit.md) | compressed packetからcompositionまでのcopy／materialize／共有／seek前提を分解し、library選定前の原因分離fixtureと停止線を固定する比較 |
| [2026-07-29-aviutl-completed-plugin-stack-performance-observation.md](2026-07-29-aviutl-completed-plugin-stack-performance-observation.md) | AviUtlを入力、handle reuse、patch、LuaJIT、RAM preview、波形／編集補助で完成するsoftwareとして再定義し、責任分解、音MAD高編集密度fixture、MotoliiのHost／plugin境界を記録 |
| [2026-07-29-aviutl2-low-spec-migration-performance-gate.md](2026-07-29-aviutl2-low-spec-migration-performance-gate.md) | 旧AviUtl完成拡張スタックとAviUtl2を別々の移行基準旗とし、L0-M音MAD映像を含む日常編集と負荷後の粘りを分ける最低スペック比較台帳 |
| [2026-07-29-external-authoring-bridge-seat-decision.md](2026-07-29-external-authoring-bridge-seat-decision.md) | 外部制作toolの選択をtyped proposalからAuthoring Toolへ渡す第三者Bridge席、app非列挙、single-writer正本化、商流／権限分離、能力別再入場gateの決定 |
| [2026-07-29-vism-cross-culture-expression-stress-test-observation.md](2026-07-29-vism-cross-culture-expression-stress-test-observation.md) | AviUtl／Blender／TouchDesigner／Cavalry／GitHubを横断し、Path→Path、補助typed出力、Feedback、Data→Data、Surface／Materialの空席と候補を分離する観察 |
| [2026-07-31-authoring-continuity-capsule-goal-contract.md](2026-07-31-authoring-continuity-capsule-goal-contract.md) | 作者経路の行き止まり禁止、一回一変更カプセル、Host所有oracle、宣言typed capability、初心者には一つの作者面・内部artifactはHost導出と段階開示、製品所有Inspect／Fork／atomic adoptionと後戻り不能な負例を固定 |
| [2026-08-01-motolii-semantic-sdk-aviutl-community-comparison.md](2026-08-01-motolii-semantic-sdk-aviutl-community-comparison.md) | AviUtl 1.x成熟拡張環境／AviUtl2とMotolii意味SDKを比較し、一作者面、自動parameter、通常Vism、fork、反復、Preset／Kit、分散配布、高度化をcontinuity floorへ縮小採用 |
| [2026-08-01-vism-authoring-language-boundary-decision.md](2026-08-01-vism-authoring-language-boundary-decision.md) | 一般creator-authorがprogramを書く段のTypeScript source、MTS-1 compatibility profile、WGSL／Rust責任、engine／package非決定、F0/F1と停止線を固定 |
| [2026-08-01-vism-inspector-source-automation-boundary-decision.md](2026-08-01-vism-inspector-source-automation-boundary-decision.md) | Vismを通常の製品単位、Inspectorを意味の第一面、TypeScriptを外部IDEへ段階開示する作者source、Automationを別の将来席として固定 |
| [2026-08-01-vism-semantic-sdk-cavalry-translation-decision.md](2026-08-01-vism-semantic-sdk-cavalry-translation-decision.md) | CavalryのPath／Mesh／Context／Deformer／Particle先例を、言語より長寿命な意味型、純粋operation、明示capability、Host責任へ翻訳するVism意味SDK決定 |
| [2026-08-01-sdk-s0-path2d-semantic-fixture-spec.md](2026-08-01-sdk-s0-path2d-semantic-fixture-spec.md) | SDK-S0S — 既存M2 PathOpをnative oracleにする`Path2D → Path2D`意味fixture責任仕様 |
| [2026-08-01-vsm-a4s-external-crate-author-scaffold-spec.md](2026-08-01-vsm-a4s-external-crate-author-scaffold-spec.md) | VSM-A4S — 外部crate作者scaffold責任仕様 |
| [2026-08-02-m5-a0s-decision-recovery.md](2026-08-02-m5-a0s-decision-recovery.md) | M5-A0S — 後発3D import／Render Contribution資料7 blobの縮小採用・観察・棄却と、runtime前の停止線 |
| [2026-08-02-m5-c0-observation-preflight.md](2026-08-02-m5-c0-observation-preflight.md) | M5-C0 — Planar／Spatial Observationの実在target、未決公開境界、M4 K1a依存、仕様化前の停止線 |
| [2026-08-02-m5-c0-observation-contract-decision.md](2026-08-02-m5-c0-observation-contract-decision.md) | M5-C0 — 初期projective Observationの意味、Host／Provider責任、拒否・換装oracle、schema/runtime分割 |
| [2026-08-02-m5-c0-schema-preflight.md](2026-08-02-m5-c0-schema-preflight.md) | M5-C0 — 公開型・serde／wire・Document version・provider identityの実在target再照合と仕様化停止線 |
| [2026-08-02-m5-pause-until-m3-semantic-release.md](2026-08-02-m5-pause-until-m3-semantic-release.md) | M5をM3意味開放まで全面休止した旧契約。2026-08-10に循環を確認して撤回し、第二writer／別world／別Preview・Export禁止の負例だけを継承 |
| [M5-C0 private Observation semantics receipt](evidence/m5-known-implementation/M5-C0/README.md) | M5-C0 — `glam` private semantic fixtureの射影、typed refusal、provider換装oracle（5/5） |
| [2026-07-09-R1-export-review.md](2026-07-09-R1-export-review.md) | コードレビュー所見 2026-07-09 (R1/Quality・export・cli周辺) |
| [2026-07-09-R3-datatrack-review.md](2026-07-09-R3-datatrack-review.md) | コードレビュー所見 2026-07-09 (R3/DataTrack統合) |
| [2026-07-10-M1-plugin-boundary-review.md](2026-07-10-M1-plugin-boundary-review.md) | 設計レビュー所見 2026-07-10 (M1完了後・プラグイン境界の凍結前監査) |
| [2026-07-10-R8-vello-review.md](2026-07-10-R8-vello-review.md) | 軽量レビュー 2026-07-10 (R8/Vello採否スパイク) |
| [2026-07-10-R9-real-material-checklist.md](2026-07-10-R9-real-material-checklist.md) | R9 実素材検証チェックリスト (T11) |
| [2026-07-10-freeze-gate-declaration.md](2026-07-10-freeze-gate-declaration.md) | 凍結ゲート宣言(2026-07-10) |
| [2026-07-10-freeze-gate-remaining.md](2026-07-10-freeze-gate-remaining.md) | 凍結ゲート残件(2026-07-10 監査) |
| [2026-07-11-INF-7g-llm-plugin-demo.md](2026-07-11-INF-7g-llm-plugin-demo.md) | INF-7g: LLMプラグイン実演記録(2026-07-11) |
| [2026-07-23-historical-llm-plugin-demo-lineage-recovery.md](2026-07-23-historical-llm-plugin-demo-lineage-recovery.md) | Unit 9E — INF-7g LLM Opacity実演の証明範囲と現行停止線 |
| [2026-07-11-M2-entry-gate.md](2026-07-11-M2-entry-gate.md) | M2入場条件(2026-07-11。同日改訂: ゲート運用レビュー7点を反映) |
| [2026-07-11-code-audit-pre-m2.md](2026-07-11-code-audit-pre-m2.md) | 実コード監査: M2並列解禁前に詰めるべき設計箇所(2026-07-11) |
| [2026-07-12-M2E-2-ruleset-activation.md](2026-07-12-M2E-2-ruleset-activation.md) | M2E-2 ruleset 有効化ログ |
| [2026-07-12-M2E-7-render-ctx-thaw.md](2026-07-12-M2E-7-render-ctx-thaw.md) | M2E-7 解凍手続き: Filter/Compositeへ`RenderCtx`を導入する |
| [2026-07-12-M3-M4-gate-ledger.md](2026-07-12-M3-M4-gate-ledger.md) | 次フェーズ入場条件の候補台帳: M3/M4(2026-07-12) |
| [2026-07-12-code-audit-2nd-d1.md](2026-07-12-code-audit-2nd-d1.md) | 第二実コード監査の裏取りと台帳化: D1系スキーマ・評価・永続(2026-07-12) |
| [2026-07-12-d1-spec-holes-prior-art.md](2026-07-12-d1-spec-holes-prior-art.md) | D1スキーマ未決点の先例調査メモ(2026-07-12) |
| [2026-07-12-m2-permanence-prevention.md](2026-07-12-m2-permanence-prevention.md) | M2恒久焼き込みの予防(2026-07-12) |
| [2026-07-28-g0-6h-v1p-capture-prerequisite-selection.md](2026-07-28-g0-6h-v1p-capture-prerequisite-selection.md) | G0-6H-V1P capture前提の再選定 |
| [2026-07-23-historical-permanence-prevention-lineage-recovery.md](2026-07-23-historical-permanence-prevention-lineage-recovery.md) | Unit 4B — GR-PV予防5手全9版とstale branch回帰の処分 |
| [2026-07-23-historical-d1-spec-holes-lineage-recovery.md](2026-07-23-historical-d1-spec-holes-lineage-recovery.md) | Unit 4C — D1仕様穴・TimeMap・Generator先例全12版の処分 |
| [2026-07-12-pathop-ae-cavalry-comparison.md](2026-07-12-pathop-ae-cavalry-comparison.md) | PathOp語彙比較: AE/Lottie × Cavalry(2026-07-12) |
| [2026-07-12-plugin-ui-v1-boundary.md](2026-07-12-plugin-ui-v1-boundary.md) | 決定: v1プラグインUI境界は`NodeDesc`自動生成のみ(2026-07-12) |
| [2026-07-12-prior-art-gap-counter-review.md](2026-07-12-prior-art-gap-counter-review.md) | 反対側レビュー: M3/プラグイン生態系の先例所見を最小化する(2026-07-12) |
| [2026-07-12-prior-art-gap-survey.md](2026-07-12-prior-art-gap-survey.md) | 先例調査: M3/プラグイン生態系の考慮漏れ(2026-07-12) |
| [2026-07-12-rework-prior-art.md](2026-07-12-rework-prior-art.md) | 出戻り: 先人の失敗後対応と、その反面(予防)(2026-07-12) |
| [2026-07-12-success-prior-art.md](2026-07-12-success-prior-art.md) | 先例調査: 成功先例からの仮説メモ(2026-07-12) |
| [2026-07-12-vertical-text-prior-art-counter-review.md](2026-07-12-vertical-text-prior-art-counter-review.md) | 反対側レビュー: 縦書き先例調査の再判定(2026-07-12) |
| [2026-07-12-vertical-text-prior-art.md](2026-07-12-vertical-text-prior-art.md) | 先例調査: 縦書き(日本語縦組み)テキストレイアウトの既存実装分解(2026-07-12) |
| [2026-07-13-decision-pack-adoption.md](2026-07-13-decision-pack-adoption.md) | 決定パック採択(2026-07-13ユーザー承認) |
| [2026-07-13-readback-pipelining-prior-art.md](2026-07-13-readback-pipelining-prior-art.md) | 先例調査: GPU→CPUリードバック重畳とcold shader compileの解決例(2026-07-13) |
| [2026-07-13-undecided-critical-path-confirm.md](2026-07-13-undecided-critical-path-confirm.md) | 友人レビュー確認: 未決事項とクリティカルパス(2026-07-13) |
| [2026-07-13-wgpu-challenges-counter-review.md](2026-07-13-wgpu-challenges-counter-review.md) | 反対側レビュー: Rust+wgpu技術的課題調査の二重補正(2026-07-13) |
| [2026-07-14-3d-depth-boundary-prior-art.md](2026-07-14-3d-depth-boundary-prior-art.md) | 先例調査: 「2Dレイヤー順合成×3D深度合成」の境界の切り方(2026-07-14) |
| [2026-07-14-3d-depth-scope-design.md](2026-07-14-3d-depth-scope-design.md) | 2Dレイヤー順と3D深度合成の境界設計(2026-07-14) |
| [2026-07-14-audio-generalization-design.md](2026-07-14-audio-generalization-design.md) | 音声を「楽曲1本」から一般メディアへ拡張する設計(2026-07-14) |
| [2026-07-14-color-conversion-prior-art.md](2026-07-14-color-conversion-prior-art.md) | 色変換(プレビュー/書き出し不一致)の既知解調査メモ(2026-07-14) |
| [2026-07-14-d5-transport-prior-art.md](2026-07-14-d5-transport-prior-art.md) | 先例調査: D5 Transport低速時戦略(2026-07-14) |
| [2026-07-14-m2-core-closure.md](2026-07-14-m2-core-closure.md) | M2コア締結宣言(撤回済み) |
| [2026-07-14-m2-exit-param-pipeline-disposition.md](2026-07-14-m2-exit-param-pipeline-disposition.md) | M2終了前判定 — Param Pipelineと操作単純化の持ち越し境界 |
| [2026-07-14-m3-ui-boundary-counter-review.md](2026-07-14-m3-ui-boundary-counter-review.md) | 反対側レビュー: M3 UI境界規約を実装可能な最小形へ縮小する(2026-07-14) |
| [2026-07-14-m3-ui-boundary-prevention.md](2026-07-14-m3-ui-boundary-prevention.md) | M3 UI境界汚染の予防(2026-07-14) |
| [2026-07-14-motion-foundation-known-tech-disposition.md](2026-07-14-motion-foundation-known-tech-disposition.md) | Motion基盤候補の既知技術による処分決定(2026-07-14) |
| [2026-07-14-motion-tools-praise-diy-gap-audit.md](2026-07-14-motion-tools-praise-diy-gap-audit.md) | モーショングラフィック4ツール 称賛・日曜大工・根本ギャップ監査 |
| [2026-07-14-recent-concept-propagation-audit.md](2026-07-14-recent-concept-propagation-audit.md) | 直近コンセプトの全層反映監査(2026-07-14) |
| [2026-07-14-repeated-wheel-standardization-audit.md](2026-07-14-repeated-wheel-standardization-audit.md) | AE反復再発明プラグイン標準化監査(2026-07-14) |
| [2026-07-14-unified-stage-camera-design.md](2026-07-14-unified-stage-camera-design.md) | Stage / Output Frame / 統一カメラ設計(2026-07-14) |
| [2026-07-15-d1l-copylocal-remint-counter-review.md](2026-07-15-d1l-copylocal-remint-counter-review.md) | D1l Copy Local内部ID契約 — 反対側レビューと採否 |
| [2026-07-15-d1l-journal-revert-boundary-counter-review.md](2026-07-15-d1l-journal-revert-boundary-counter-review.md) | D1l journal/Undo/Writer追補 — 反対側レビューと採否 |
| [2026-07-15-d1l-journal-revert-boundary-decision.md](2026-07-15-d1l-journal-revert-boundary-decision.md) | D1l journal互換・Undo等価・Writer採番境界 — 追補決定 |
| [2026-07-15-implementation-readiness-ledger.md](2026-07-15-implementation-readiness-ledger.md) | Relative / Stage / Shared Effect / Bounds / Duplicator 実装準備台帳(2026-07-15) |
| [2026-07-15-m2-foundation-reclosure-counter-review.md](2026-07-15-m2-foundation-reclosure-counter-review.md) | M2基盤再締結ゲート 反対側レビュー(2026-07-15) |
| [2026-07-15-m2-foundation-reclosure-gate.md](2026-07-15-m2-foundation-reclosure-gate.md) | M2基盤再締結ゲート(2026-07-15) |
| [2026-07-15-p5-generative-pattern-disposition.md](2026-07-15-p5-generative-pattern-disposition.md) | p5.js系ジェネラティブ表現の分類とMotoliiへの配置 |
| [2026-07-15-prior-art-complaint-boundary-audit.md](2026-07-15-prior-art-complaint-boundary-audit.md) | 先例収束 / 日曜大工境界監査(2026-07-15) |
| [2026-07-15-relative-scope-duplicator-decision.md](2026-07-15-relative-scope-duplicator-decision.md) | Relative Move / Timeline Effect Link / Duplicator決定(2026-07-15) |
| [2026-07-15-shared-effect-lifecycle-decision.md](2026-07-15-shared-effect-lifecycle-decision.md) | Shared Effect lifecycle決定(GAP-14 / D1l実装ゲート) |
| [2026-07-16-ae-layer-system-disposition.md](2026-07-16-ae-layer-system-disposition.md) | AEレイヤー方式への処置台帳と出戻り一次声調査 |
| [2026-07-16-d1l-current-document-constructor-counter-review.md](2026-07-16-d1l-current-document-constructor-counter-review.md) | D1l新規Document v4生成契約 — 反対側レビューと採否 |
| [2026-07-16-d1l-current-document-constructor-decision.md](2026-07-16-d1l-current-document-constructor-decision.md) | D1l新規Documentのv4到達境界 — 追補決定 |
| [2026-07-16-d1l-new-v1-lint-conflict-decision.md](2026-07-16-d1l-new-v1-lint-conflict-decision.md) | D1l `new_v1` lintとprotected semantic testの矛盾解消決定(2026-07-16) |
| [2026-07-16-m2-comp-camera-decision.md](2026-07-16-m2-comp-camera-decision.md) | M2 CompCamera決定 — planar v1、空間は追加的拡張(2026-07-16) |
| [2026-07-16-m2-param-element-constraint-disposition.md](2026-07-16-m2-param-element-constraint-disposition.md) | M2 Param Pipeline / Element Domain / Constraint Graph処分(2026-07-16) |
| [2026-07-16-m2-project-sidecar-session-decision.md](2026-07-16-m2-project-sidecar-session-decision.md) | M2 project sidecar identity / session所有決定(2026-07-16) |
| [2026-07-23-historical-d1m-sidecar-session-lineage-recovery.md](2026-07-23-historical-d1m-sidecar-session-lineage-recovery.md) | Unit 4A — D1m sidecar/session全6版、D1n分岐、A0S追補、legacy診断の処分 |
| [2026-07-23-historical-d1-code-audit-lineage-recovery.md](2026-07-23-historical-d1-code-audit-lineage-recovery.md) | Unit 4C-2 — 第二D1コード監査全4版を現行コードで再判定し、DataTrack identityとOTIO loss reportを再回収 |
| [2026-07-23-historical-first-code-audit-lineage-recovery.md](2026-07-23-historical-first-code-audit-lineage-recovery.md) | Unit 4C-3 — M2前第一コード監査全2版を再判定し、実装済み群と公開runtime／M4／M5残件を分離 |
| [2026-07-23-historical-render-ctx-thaw-lineage-recovery.md](2026-07-23-historical-render-ctx-thaw-lineage-recovery.md) | Unit 4D — RenderCtx解凍全2版、Quality製品配線追補、予約fieldの非証明範囲を処分 |
| [2026-07-23-historical-test-oracle-ruleset-recovery.md](2026-07-23-historical-test-oracle-ruleset-recovery.md) | Unit 4E — M2E-2 ruleset有効化ログをlive設定へ再照合し、oracle保護の責任分離を固定 |
| [2026-07-23-historical-m2-entry-gate-lineage-recovery.md](2026-07-23-historical-m2-entry-gate-lineage-recovery.md) | Unit 4F — M2入口ゲート全43版の限定gate、A→B→C順序、棄却案、完了再開、歴史的達成範囲を処分 |
| [2026-07-23-historical-m2-reclosure-gate-lineage-recovery.md](2026-07-23-historical-m2-reclosure-gate-lineage-recovery.md) | Unit 4G — M2基盤再締結全14版のA/B/C証明、D1n分岐、解除とM3入場の責任分離を処分 |
| [2026-07-23-historical-m2-supplementary-review-lineage-recovery.md](2026-07-23-historical-m2-supplementary-review-lineage-recovery.md) | Unit 4H — M2独立追補レビュー全3版の初回P1、修復再審査、証拠増分、P2現行処分を回収 |
| [2026-07-23-historical-m2-camera-contract-lineage-recovery.md](2026-07-23-historical-m2-camera-contract-lineage-recovery.md) | Unit 4I — planar camera決定＋runtime解凍全5版のsemantic core／runtime／実UI分離とSpatial再入場条件を処分 |
| [2026-07-23-historical-shared-effect-lifecycle-lineage-recovery.md](2026-07-23-historical-shared-effect-lifecycle-lineage-recovery.md) | Unit 4J — Shared Effect全3版のlifecycle、内部ID再採番、予約区間、Undo watermark、UI分離を処分 |
| [2026-07-23-historical-d1l-counter-review-evidence-recovery.md](2026-07-23-historical-d1l-counter-review-evidence-recovery.md) | Unit 4K — D1l反対側レビュー3本の反例、修復、再審査と、timeout／非実在pathを証拠へ数えない規律を処分 |
| [2026-07-23-historical-d1l-constructor-lint-lineage-recovery.md](2026-07-23-historical-d1l-constructor-lint-lineage-recovery.md) | Unit 4L — current constructor＋legacy lint全4版を処分し、Document意味完成とdoc-hidden／suppression実装driftを分離 |
| [2026-07-23-historical-d1l-journal-undo-lineage-recovery.md](2026-07-23-historical-d1l-journal-undo-lineage-recovery.md) | Unit 4M — journal／Undo／Writer全2版のEffect実装、Position Add Key未実装追補、snapshot fallback driftを処分 |
| [2026-07-23-historical-param-element-constraint-lineage-recovery.md](2026-07-23-historical-param-element-constraint-lineage-recovery.md) | Unit 4N — Param Pipeline／Element Domain／Constraint Graph全2版のsingle-source維持、三解凍gate、task ID衝突を処分 |
| [2026-07-23-historical-semantic-oracle-boundary-recovery.md](2026-07-23-historical-semantic-oracle-boundary-recovery.md) | Unit 4O — D1i-4 semantic oracle訂正全1版のoracle／harness分離、段階移行、gate自己保護不足を処分 |
| [2026-07-23-historical-reclosure-counter-review-evidence-recovery.md](2026-07-23-historical-reclosure-counter-review-evidence-recovery.md) | Unit 4P — M2再締結gate反対側レビュー全1版の事前検収、authority確認、証拠段階分離、timeout非証拠を処分 |
| [2026-07-23-historical-unified-stage-camera-ui-lineage-recovery.md](2026-07-23-historical-unified-stage-camera-ui-lineage-recovery.md) | Unit 4Q — 統一Stage／Camera UI全2版の旧schema分別、操作owner、off-frame同一world、分類軸の直交を処分 |
| [2026-07-23-historical-r1-export-gpu-safety-lineage-recovery.md](2026-07-23-historical-r1-export-gpu-safety-lineage-recovery.md) | Unit 5A — R1 export／GPU safety全5版の実装修復、監査漏れ、未到達G1〜G8、GPU health driftを処分 |
| [2026-07-23-historical-audio-generalization-lineage-recovery.md](2026-07-23-historical-audio-generalization-lineage-recovery.md) | Unit 5B — 音声一般化全6版の恒久意味、進捗表示、D5訂正、mixer coreと製品Transport／UI未到達を処分 |
| [2026-07-23-historical-wgpu-readback-cold-compile-lineage-recovery.md](2026-07-23-historical-wgpu-readback-cold-compile-lineage-recovery.md) | Unit 5C — wgpu課題／先例全4版の計測前優先度訂正、同期readback、product cold pipeline gapを処分 |
| [2026-07-23-historical-d5-transport-lineage-recovery.md](2026-07-23-historical-d5-transport-lineage-recovery.md) | Unit 5D — D5 Transport全4版のaudio clock主、video drop、DRS縮退、device wait／D4-FU境界を処分 |
| [2026-07-23-historical-color-export-lineage-recovery.md](2026-07-23-historical-color-export-lineage-recovery.md) | Unit 5E — 色変換／GPU export先例1版を処分し、GAP-31、TRC、readbackの責任を分離 |
| [2026-07-23-historical-media-portability-gpu-resurvey-plan-recovery.md](2026-07-23-historical-media-portability-gpu-resurvey-plan-recovery.md) | Unit 5F — 可搬性／GPUベンダ差の未実施再調査計画1版を狭い再入場gateへ処分 |
| [2026-07-23-historical-vello-adoption-lineage-recovery.md](2026-07-23-historical-vello-adoption-lineage-recovery.md) | Unit 5G — Vello採否レビュー／spike結果2版の局所renderer採択、単一premul境界、製品未統合を処分 |
| [2026-07-23-historical-r9-real-material-export-acceptance-lineage-recovery.md](2026-07-23-historical-r9-real-material-export-acceptance-lineage-recovery.md) | Unit 5H — R9実素材／B-4書き出し受入4版の歴史sign-offと現行release受入を分離 |
| [2026-07-23-historical-s2-decode-pipeline-lineage-recovery.md](2026-07-23-historical-s2-decode-pipeline-lineage-recovery.md) | Unit 5I — M0-S2 decode 6版の採択済み自前pipe／CFR seekと未成立VFR／process lifecycleを分離 |
| [2026-07-23-historical-m4-cache-analysis-spec-lineage-recovery.md](2026-07-23-historical-m4-cache-analysis-spec-lineage-recovery.md) | Unit 5J — M4 cache／analysis仕様20版のHost専権、完全key、StateTrack、敗北枝、未実装境界を再締結 |
| [2026-07-23-historical-performance-model-lineage-recovery.md](2026-07-23-historical-performance-model-lineage-recovery.md) | Unit 5K — performance model 21版の帯域規律、liveness-aware target pool、性能仮説／実装境界を再締結 |
| [2026-07-23-historical-memory-model-lineage-recovery.md](2026-07-23-historical-memory-model-lineage-recovery.md) | Unit 5L — memory model 6版の階層責任、hard budget、capacity／deadline、K1／K7／K8未実装境界を再締結 |
| [2026-07-23-historical-r3-datatrack-export-correctness-lineage-recovery.md](2026-07-23-historical-r3-datatrack-export-correctness-lineage-recovery.md) | Unit 5M — R3/DataTrack統合review 3版のfail-closed、export長、fallback、helper driftを現行再判定 |
| [2026-07-16-m3-preflight-decisions.md](2026-07-16-m3-preflight-decisions.md) | M3着手前決定 — 操作の意味を固定し、見た目の実値は測って決める |
| [2026-07-16-m3-ui-concept-to-tickets.md](2026-07-16-m3-ui-concept-to-tickets.md) | M3 UIコンセプトから実装チケットへの分解 |
| [2026-07-16-m3-ui-gap-survey.md](2026-07-16-m3-ui-gap-survey.md) | M3前UIギャップ調査: U1〜U8に席が無いUI要素とコア側前提の欠落(2026-07-16) |
| [2026-07-16-m3-ui-rapid-acceptance-prior-art.md](2026-07-16-m3-ui-rapid-acceptance-prior-art.md) | 先例調査: すぐに受け入れられたUI(2026-07-16) |
| [2026-07-16-media-portability-gpu-resurvey-plan.md](2026-07-16-media-portability-gpu-resurvey-plan.md) | 再調査ラウンド起案: メディア可搬性(GAP-3/7)とGPUベンダ差(INF-3)(2026-07-16) |
| [2026-07-16-ui-update-forensics.md](2026-07-16-ui-update-forensics.md) | UIアップデート考古学 — 改善履歴から潜在的な失敗を読む |
| [2026-07-17-aviutl2-comment-voices.md](2026-07-17-aviutl2-comment-voices.md) | AviUtl2動画コメント欄 — 統一できない利用者の声 |
| [2026-07-17-d1i4-semantic-oracle-boundary-decision.md](2026-07-17-d1i4-semantic-oracle-boundary-decision.md) | D1i-4 / S16: semantic oracle 保護境界の訂正 |
| [2026-07-17-extensible-core-prior-art-translation.md](2026-07-17-extensible-core-prior-art-translation.md) | 個体性・介入・上限・縮退・遊びの先例翻訳(2026-07-17) |
| [2026-07-17-non-video-workspace-asset-ui-prior-art.md](2026-07-17-non-video-workspace-asset-ui-prior-art.md) | 動画ソフト外から引き直すWorkspace・素材探索・視線設計 |
| [2026-07-17-vism-a0-plugin-boundary-inventory.md](2026-07-17-vism-a0-plugin-boundary-inventory.md) | VSM-A0 — 現行plugin境界inventory |
| [2026-07-17-vism-a0d-contract-migration-ownership-decision.md](2026-07-17-vism-a0d-contract-migration-ownership-decision.md) | VSM-A0D — plugin契約とmigrationの所有決定 |
| [2026-07-17-vism-a0s-contract-catalog-spec.md](2026-07-17-vism-a0s-contract-catalog-spec.md) | VSM-A0S — Contract Catalogとprepared plugin解決仕様 |
| [2026-07-23-historical-vism-foundation-contract-lineage-recovery.md](2026-07-23-historical-vism-foundation-contract-lineage-recovery.md) | Unit 9C — Vism-ready反対側レビュー、A0D/A0S、A2、A7の全版処分とD1m時点補正 |
| [2026-07-17-vism-a1-public-crate-boundary-spec.md](2026-07-17-vism-a1-public-crate-boundary-spec.md) | VSM-A1S — Opacity外部crate化の公開境界仕様 |
| [2026-07-17-vism-a2-legacy-project-migration-decision.md](2026-07-17-vism-a2-legacy-project-migration-decision.md) | VSM-A2S — 旧CLI ProjectV1 migration処分 |
| [2026-07-17-vism-a7-bpm-datatrack-spike.md](2026-07-17-vism-a7-bpm-datatrack-spike.md) | VSM-A7 — BPMから既存DataTrackへの意味spike |
| [2026-07-17-vism-implementation-plan.md](2026-07-17-vism-implementation-plan.md) | Vism実装計画 — 公開境界の反証から配布へ |
| [2026-07-17-vism-ready-counter-review-disposition.md](2026-07-17-vism-ready-counter-review-disposition.md) | Vism-ready化提案の反対側レビュー採否 |
| [2026-07-18-d1k-runtime-camera-thaw-spec.md](2026-07-18-d1k-runtime-camera-thaw-spec.md) | D1k-S CQ-5 解凍記録: runtime planar `CompCamera`と必須camera-bearing render signature(2026-07-18) |
| [2026-07-18-m2-foundation-supplementary-code-review.md](2026-07-18-m2-foundation-supplementary-code-review.md) | M2基盤再締結・独立追補実コードレビュー(2026-07-18) |
| [2026-07-18-m3-egui-selection.md](2026-07-18-m3-egui-selection.md) | M3 UI基盤 egui採用判断(2026-07-18) |
| [2026-07-18-m3-gpu-preview-viewport-prior-art.md](2026-07-18-m3-gpu-preview-viewport-prior-art.md) | M3 GPU Preview / Viewport先例調査 |
| [2026-07-18-vism-a3-external-expression-survey.md](2026-07-18-vism-a3-external-expression-survey.md) | VSM-A3R — 外部表現・Expression・Add-onの責任分類 |
| [2026-07-18-vism-a3d-radial-repeater-decision.md](2026-07-18-vism-a3d-radial-repeater-decision.md) | VSM-A3D — 決定論的 2D Radial Repeater LayerSource 採用決定 |
| [2026-07-18-vism-a3s-layersource-lowering-spec.md](2026-07-18-vism-a3s-layersource-lowering-spec.md) | VSM-A3S — 一般 LayerSource lowering 仕様 |
| [2026-07-23-historical-vism-a3-expression-layersource-lineage-recovery.md](2026-07-23-historical-vism-a3-expression-layersource-lineage-recovery.md) | Unit 9D — 外部表現責任分類、Radial Repeater採択、LayerSource lowering全版処分 |
| [2026-07-19-am-keyframe-graph-observation.md](2026-07-19-am-keyframe-graph-observation.md) | Alight Motionキーフレームグラフ観察台帳(AM実機確認。`codex/m3-mock-components`側から回収) |
| [2026-07-19-graph-view-reference-decision.md](2026-07-19-graph-view-reference-decision.md) | multi-key Graph ViewのReact比較記録。製品採択・M3 task化は未決 |
| [2026-07-19-m3-interaction-prototype-decision-ledger.md](2026-07-19-m3-interaction-prototype-decision-ledger.md) | M3操作prototype未決パラメータ台帳(2026-07-19。`codex/m3-mock-components`側から回収) |
| [2026-07-19-lyric-motion-text-sequence-comparison.md](2026-07-19-lyric-motion-text-sequence-comparison.md) | リリックモーション: Text Sequence / Materialize 比較台帳(2026-07-19) |
| [2026-07-19-m3-text-motion-task-translation.md](2026-07-19-m3-text-motion-task-translation.md) | M3タスク翻訳: Text Motion(Live Text)縦切り第1弾(2026-07-19) |
| [2026-07-20-rerun-prior-art-survey.md](2026-07-20-rerun-prior-art-survey.md) | Rerun先例調査と歴史的方向決定: 主要製品先例は継続、egui固有転移はG0-9待ち |
| [2026-07-20-rerun-learning-transfer-plan.md](2026-07-20-rerun-learning-transfer-plan.md) | Rerun → Motolii学習・転移計画: RR-0〜9、資産分類、M3/M5接続、停止線 |
| [2026-07-20-rerun-source-asset-inventory.md](2026-07-20-rerun-source-asset-inventory.md) | Rerun固定commitの139 package全量と重点source資産の観察inventory |
| [2026-07-20-rerun-re-ui-module-inventory.md](2026-07-20-rerun-re-ui-module-inventory.md) | Rerun `re_ui` module inventory: React安定ID・M3 task・CJK・転移候補のfile-level照合 |
| [2026-07-20-m3-rerun-late-discovery-premortem.md](2026-07-20-m3-rerun-late-discovery-premortem.md) | M3/Rerun実装後半発覚プレモーテム: fixture正本、GPU表示寿命、stable identity、semantic zoom、転移粒度の先行処分 |
| [2026-07-20-perceptual-expression-translation-decision.md](2026-07-20-perceptual-expression-translation-decision.md) | 知覚表現の翻訳 — Motolii Hostの役割 |
| [2026-07-20-local-worktree-publication-audit.md](2026-07-20-local-worktree-publication-audit.md) | ローカルworktreeの公開・WIP保全・吸収済み・旧契約差分を分類した外部再開地図 |
| [2026-07-21-m3-react-webview-runtime-reconsideration.md](2026-07-21-m3-react-webview-runtime-reconsideration.md) | M3 React / WebView UI runtime再選定（2026-07-21） |
| [2026-07-29-cu-0a09b-browser-standalone-mount-implementation-decision.md](2026-07-29-cu-0a09b-browser-standalone-mount-implementation-decision.md) | R6 Browser standalone mount実装決定 |
| [2026-07-21-native-stage-gizmo-ownership.md](2026-07-21-native-stage-gizmo-ownership.md) | Native Stage gizmo所有境界: wgpu overlay / CPU picking / Web controls |
| [2026-07-21-native-stage-gizmo-counter-review.md](2026-07-21-native-stage-gizmo-counter-review.md) | Native Stage gizmo案の反対側レビューと縮小採用 |
| [2026-07-21-native-surface-renderer-reselection.md](2026-07-21-native-surface-renderer-reselection.md) | React複合下のnative Stage/Timeline renderer再選定とFableレビュー入口 |
| [2026-07-21-native-surface-renderer-extended-search.md](2026-07-21-native-surface-renderer-extended-search.md) | native surface renderer拡張サーチ(egui以外の追加候補・先例・支援基盤) |
| [2026-07-21-native-surface-renderer-counter-review.md](2026-07-21-native-surface-renderer-counter-review.md) | native surface renderer反対側レビュー(Fable回答・11問) |
| [2026-07-21-native-surface-renderer-growth-review.md](2026-07-21-native-surface-renderer-growth-review.md) | native surface renderer伸長レビュー(Fable回答・機会と優先順位) |
| [2026-07-21-ui-surface-topology-decision.md](2026-07-21-ui-surface-topology-decision.md) | 1 top-level wgpu Surface + Stage/Timeline viewport + opaque child WebView islandsのtopology決定 |
| [2026-07-21-m3-product-mock-recovery-plan.md](2026-07-21-m3-product-mock-recovery-plan.md) | Rectangle製品縦切り・Timeline・複数Surface・隔離・OS受入の一括回収計画と停止線 |
| [2026-07-21-m3-rectangle-drop-d2-contract-options.md](2026-07-21-m3-rectangle-drop-d2-contract-options.md) | Rectangle dropのD2個別契約案: LayerId原子性・exactly-once・selection・Undo/Redo |
| [2026-07-22-m3-comfortable-use-work-map.md](2026-07-22-m3-comfortable-use-work-map.md) | 製品外殻からLocal Alpha、日常操作、配布品質までをユーザーの制作経路で並べる大地図 |
| [2026-07-22-m3-comfortable-use-granulation.md](2026-07-22-m3-comfortable-use-granulation.md) | 快適利用大地図を仕様判断・asset・core・product・E2E・人間/実機審判の検証可能な粒へ分解 |
| [2026-07-22-m3-react-product-asset-promotion-contract.md](2026-07-22-m3-react-product-asset-promotion-contract.md) | Reactモックcomponentを製品packageへ直接所有移管し、縮約再実装と二重stateを拒否する契約 |
| [2026-07-22-m3-native-easing-popup-acceptance.md](2026-07-22-m3-native-easing-popup-acceptance.md) | React起点のnative wgpu Easing popupについて所有境界とG0-9受入条件を固定 |
| [2026-07-22-m3-react-coordinate-surface-audit.md](2026-07-22-m3-react-coordinate-surface-audit.md) | 固定React source内のCanvas/SVG/座標描画面を機械監査し、native再現残量と順序を分類 |
| [2026-07-22-m3-native-multi-key-graph-view-acceptance.md](2026-07-22-m3-native-multi-key-graph-view-acceptance.md) | Blender-like操作語彙を採るnative Multi-key Graph Viewのisolated受入とGPL停止線 |
| [2026-07-22-m3-graph-headless-interaction-dependency.md](2026-07-22-m3-graph-headless-interaction-dependency.md) | Graph Viewのpan/zoom/fitをheadless依存へ委ね、selection/snap/D2をMotoliiに残す裁定 |
| [2026-07-22-m3-native-depth-rail-acceptance.md](2026-07-22-m3-native-depth-rail-acceptance.md) | React正本からnative Depth Railへ同一Z stack、scope、Layer Order Distributeを移すisolated受入契約 |
| [2026-07-22-m3-detachable-panel-window-contract.md](2026-07-22-m3-detachable-panel-window-contract.md) | Timeline/Graphの分割を全製品panelへ一般化するdetach/re-dock・multi-window・単一snapshot契約 |
| [2026-07-22-m3-surface-extension-axis-separation.md](2026-07-22-m3-surface-extension-axis-separation.md) | OS topology、presentation runtime、製品module、plugin、provenance/trustを別軸として固定 |
| [2026-07-22-u0e-2-delegation-guardrails.md](2026-07-22-u0e-2-delegation-guardrails.md) | U0e-2縮約再実装を再発させないdispatch・source provenance・fixture因果・原子性・証跡ガード |
| [2026-07-22-creator-developer-continuum-decision.md](2026-07-22-creator-developer-continuum-decision.md) | 利用者から作者までを一つの経路にし、React・Vism・first-party参照実装を多数作者の成長戦略へ統合 |
| [2026-07-22-ui-music-metaphor-retirement.md](2026-07-22-ui-music-metaphor-retirement.md) | 「演奏・譜面台・楽曲が背骨」を製品全体の比喩とする仮説を撤回し、音声機能と製品存在論を分離 |
| [2026-07-23-losing-specification-value-recovery.md](2026-07-23-losing-specification-value-recovery.md) | 「負けた仕様」を主張単位で分類し、single camera／2.5Dの系譜と旧KitのPlugin Set／Lock価値を回収 |
| [2026-07-23-historical-value-recovery-coverage-ledger.md](2026-07-23-historical-value-recovery-coverage-ledger.md) | 全refのMarkdown履歴を固定manifestとblob receiptで単位別回収するcoverage台帳 |
| [2026-07-23-historical-semantic-graph-recovery-tooling.md](2026-07-23-historical-semantic-graph-recovery-tooling.md) | Git corpus・receipt正本・決定的projection・任意の意味索引を分離する履歴回収tooling契約 |
| [2026-07-23-historical-foundation-lineage-recovery.md](2026-07-23-historical-foundation-lineage-recovery.md) | historical-only基盤文書を処分し、D1n external revisionを再採択、multi-key Graphを未採択候補へ訂正 |
| [2026-07-23-historical-react-webview-lineage-recovery.md](2026-07-23-historical-react-webview-lineage-recovery.md) | historical-only React/WebView文書を処分し、built-in Host不変条件と四面同期縦切りを現行境界へ再採択 |
| [2026-07-23-historical-d2-selection-timeline-lineage-recovery.md](2026-07-23-historical-d2-selection-timeline-lineage-recovery.md) | historical-only Place/Add Key/selection/headless Timeline契約を採択済み・未実装follow-upへ回収 |
| [2026-07-23-historical-core-plugin-boundary-lineage-recovery.md](2026-07-23-historical-core-plugin-boundary-lineage-recovery.md) | 小さなCore、M1 plugin境界、M2締結撤回の12履歴blobを処分し、crate／Host module／plugin／provenanceの混線を解消 |
| [2026-07-23-historical-plugin-ui-lineage-recovery.md](2026-07-23-historical-plugin-ui-lineage-recovery.md) | plugin UI比較とv1境界の15履歴blobを処分し、自動panel未実装、G0-3/GAP-13停止線、宣言語彙の再入場条件を整理 |
| [2026-07-23-historical-plugin-resource-runtime-lineage-recovery.md](2026-07-23-historical-plugin-resource-runtime-lineage-recovery.md) | plugin resource/VSM-A0の5履歴blobを処分し、PipelineCache/AssetRef実装済みとGpuAssetCache/Importer/Feedback未実装を分離 |
| [2026-07-23-historical-plugin-authoring-lineage-recovery.md](2026-07-23-historical-plugin-authoring-lineage-recovery.md) | plugin authoring全41版を処分し、static first-party実証、未実装の外部crate scaffold、native/WASM/Vism配布停止線を分離 |
| [2026-07-23-historical-frame-desc-shared-types-lineage-recovery.md](2026-07-23-historical-frame-desc-shared-types-lineage-recovery.md) | M1全28版のFrameDesc／TextureRef共有型lineageを処分し、6意味の凍結、歴史的trait skeleton、現行constructor/serde/error gapを分離 |
| [2026-07-23-historical-public-capability-provenance-lineage-recovery.md](2026-07-23-historical-public-capability-provenance-lineage-recovery.md) | A1公開crate全9版とsurface/provenance・creator連続体を処分し、bundled first-party source実証と未成立third-party runtimeを分離 |
| [2026-07-23-historical-vism-kit-distribution-lineage-recovery.md](2026-07-23-historical-vism-kit-distribution-lineage-recovery.md) | Vism package／Kit／実装計画29版を処分し、Kit構成とPlugin Set／Project Lock／catalog／hostless配布を別責任で再接続 |
| [2026-07-23-historical-plugin-ecosystem-lineage-recovery.md](2026-07-23-historical-plugin-ecosystem-lineage-recovery.md) | 旧plugin ecosystemの未処分11版を処分し、中央人気／dedupeを持たないcommunity原則と旧tap/lock/build schemaを分離 |
| [2026-07-23-vism-kit-rack-unification-decision.md](2026-07-23-vism-kit-rack-unification-decision.md) | 独立Plugin Setを廃止し、接続済み一式をRack型Vism Kitへ、推薦だけの集合をcurator list／feedへ分離 |
| [2026-07-22-terra-grok-delegation-policy.md](2026-07-22-terra-grok-delegation-policy.md) | **ARCHIVED** — task class別のLuna／Terra／Sol実装、Grok検収、Fable追加検収を定めた旧運用 |
| [2026-07-25-opus-spark-grok-supervision-loop-decision.md](2026-07-25-opus-spark-grok-supervision-loop-decision.md) | 撤回済み固定model監督ループのtombstoneと現行runner非依存正本への移動先 |
| [2026-07-23-parallel-order-pipeline-comparison.md](2026-07-23-parallel-order-pipeline-comparison.md) | **ARCHIVED** — 旧task-class運用上でpreflight・実装・検収を重ねた発注パイプライン比較 |
| [2026-07-23-first-party-vism-expression-demand-survey.md](2026-07-23-first-party-vism-expression-demand-survey.md) | AE・AviUtl 1.x／2・Cavalryの表現需要と、人気plugin／公開script系統からVism候補、Host／Infrastructure／Adapter責任、Kitを分ける調査 |
| [2026-07-23-group-source-pool-cloner-concept.md](2026-07-23-group-source-pool-cloner-concept.md) | Groupの直接の子を割合つきprototype poolとしてClonerへ渡すMotolii固有概念の比較 |
| [2026-07-23-m3-g0-9-staged-platform-gates.md](2026-07-23-m3-g0-9-staged-platform-gates.md) | G0-9を固定Macのlocal prerequisite gateとdistribution gateへ段階化する決定 |
| [2026-07-24-camera-object-provider-decision.md](2026-07-24-camera-object-provider-decision.md) | Cameraを換装可能なTimeline Object／Providerとし、点群・splat・volume等とrepresentation非依存Observation Contractで接続する決定 |
| [2026-07-24-dependency-first-responsibility-gate.md](2026-07-24-dependency-first-responsibility-gate.md) | 汎用機構の着手前に既存経路と導入後の総責任を比較し、Motolii固有責任を最小化する決定 |
| [2026-07-24-m3-g0-9l-l1-measurement-amendment.md](2026-07-24-m3-g0-9l-l1-measurement-amendment.md) | G0-9L L1 renderer比較armとfixed-Mac計測証拠の正本修正 |
| [2026-07-24-m3-vertical-slice-execution-decision.md](2026-07-24-m3-vertical-slice-execution-decision.md) | M3進捗を通常製品routeから既存能力へ接続する縦sliceで管理し、A〜Dをchecklist、旧粒度化を履歴snapshotへ移す決定（Fable最終ACCEPT） |
| [2026-07-25-controlled-microkernel-host-module-parallelism-decision.md](2026-07-25-controlled-microkernel-host-module-parallelism-decision.md) | Coreをauthorityとtyped protocolへ細くし、Host capabilityを並列化する。TCBをCore＋admitted Host moduleへ限定し、公開pluginは供給元を問わず非信頼とする決定 |
| [2026-07-25-controlled-microkernel-fable-counter-review.md](2026-07-25-controlled-microkernel-fable-counter-review.md) | 全体並列化の現行コードseam、未成立境界、seat別判定、最小proof順序をFable 5で反対側検収。初回REVISE訂正後ACCEPT、P0/P1=0 |
| [2026-07-25-parallel-human-response-frontier-execution-decision.md](2026-07-25-parallel-human-response-frontier-execution-decision.md) | 決定済みcontract上のprovider／consumer／fault／measureを並列に進め、通常製品routeの人間応答地点へrolling waveで返す実行決定。Fableを共有境界reviewへ限定し新しい直列barrierにしない |
| [2026-07-25-parallel-lane-readiness-map.md](2026-07-25-parallel-lane-readiness-map.md) | Wave 0の7 lane、lane-local直列性、変更path衝突、Human Response Frontier、WAITを固定する着手地図 |
| [2026-07-25-parallel-lane-readiness-fable-review.md](2026-07-25-parallel-lane-readiness-fable-review.md) | 並列レーン候補をFable 5で反対側レビューし、R2A/R2B混同とK0/P0I旧全体直列文言のP1二件を訂正後、再審査ACCEPTとなった記録 |
| [2026-07-25-cu-0a05a-interrupted-worktree-restart-disposition.md](2026-07-25-cu-0a05a-interrupted-worktree-restart-disposition.md) | CU-0A05A旧隔離差分を完了証拠にしない停止線と、固定oracle／現行closure hash、status、single-owner triggerの再入場decision |
| [2026-07-25-cu-0a06-r3-readiness-split-decision.md](2026-07-25-cu-0a06-r3-readiness-split-decision.md) | KEYS/LAYERS独立source不在を受け、R3をmock-side JSX/CSS抽出とbyte同一product ownershipへ分割する決定 |
| [2026-07-25-cu-0a07-r4-readiness-split-decision.md](2026-07-25-cu-0a07-r4-readiness-split-decision.md) | Inspector独立React source不在を受け、R4を未変更source oracle、mock-side同形React化、byte同一product ownershipへ分割する決定 |
| [2026-07-25-supervised-runner-derived-target-closure.md](2026-07-25-supervised-runner-derived-target-closure.md) | workspace試験が作る既知のworktree-root派生物を検収前にfail-closed清掃し、ignored scope監査を維持するGR-D3実行決定 |
| [2026-07-20-m3-keymap-codec-contract.md](2026-07-20-m3-keymap-codec-contract.md) | U0d-2 keymap JSON codec契約 |
| [2026-07-20-m3-u2a-1-command-adapter-contract.md](2026-07-20-m3-u2a-1-command-adapter-contract.md) | U2a-1 gesture command adapter契約 |
| [2026-07-21-m3-u1a-1-static-viewport-contract.md](2026-07-21-m3-u1a-1-static-viewport-contract.md) | U1a-1 静止viewport実装前契約 |
| [2026-07-21-m3-u0e-1-token-generator-contract.md](2026-07-21-m3-u0e-1-token-generator-contract.md) | U0e-1 DTCG token generator契約 |
| [2026-07-21-m3-u0e-2-reference-fixture-contract.md](2026-07-21-m3-u0e-2-reference-fixture-contract.md) | U0e-2 React再結合・5 reference fixture契約 |
| [2026-07-21-m3-u1a-2-layout-projection-contract.md](2026-07-21-m3-u1a-2-layout-projection-contract.md) | U1a-2 panel layout intent / runtime投影契約 |
| [2026-07-21-m3-u1b-1-render-worker-contract.md](2026-07-21-m3-u1b-1-render-worker-contract.md) | U1b-1 latest mailbox / render worker契約 |
| [2026-07-21-m3-u1b-2-latest-projection-contract.md](2026-07-21-m3-u1b-2-latest-projection-contract.md) | U1b-2 latest result / event-loop投影契約 |
| [2026-07-21-m3-u2b-1-single-writer-e2e-contract.md](2026-07-21-m3-u2b-1-single-writer-e2e-contract.md) | U2b-1 single writer配送E2E契約 |
| [2026-07-21-m3-u2c-1-interaction-state-contract.md](2026-07-21-m3-u2c-1-interaction-state-contract.md) | U2c-1 共通interaction state machine契約 |
| [2026-07-21-m3-u2c-4-diagnostic-envelope-contract.md](2026-07-21-m3-u2c-4-diagnostic-envelope-contract.md) | U2c-4 Transient Diagnostic Envelope契約 |

### 歴史価値回収（固定 corpus）

| 文書 | 内容 |
|---|---|
| [2026-07-23-historical-semantic-graph-recovery-tooling.md](2026-07-23-historical-semantic-graph-recovery-tooling.md) | Git corpus・receipt・可搬projection・任意索引の責任境界 |
| [2026-07-23-historical-value-recovery-coverage-ledger.md](2026-07-23-historical-value-recovery-coverage-ledger.md) | 固定manifestとblob receiptによるcoverage台帳 |
| [2026-07-23-losing-specification-value-recovery.md](2026-07-23-losing-specification-value-recovery.md) | 旧仕様を主張単位で分類する回収方針 |
| [2026-07-23-vism-kit-rack-unification-decision.md](2026-07-23-vism-kit-rack-unification-decision.md) | Vism Kit／Plugin Set／curator listの責任分離 |
| [2026-07-23-historical-audio-generalization-lineage-recovery.md](2026-07-23-historical-audio-generalization-lineage-recovery.md) | 音声一般化の歴史回収 |
| [2026-07-23-historical-color-export-lineage-recovery.md](2026-07-23-historical-color-export-lineage-recovery.md) | 色・exportの歴史回収 |
| [2026-07-23-historical-core-plugin-boundary-lineage-recovery.md](2026-07-23-historical-core-plugin-boundary-lineage-recovery.md) | Core／Host／plugin境界の歴史回収 |
| [2026-07-23-historical-d1-code-audit-lineage-recovery.md](2026-07-23-historical-d1-code-audit-lineage-recovery.md) | D1 code auditの歴史回収 |
| [2026-07-23-historical-d1-spec-holes-lineage-recovery.md](2026-07-23-historical-d1-spec-holes-lineage-recovery.md) | D1仕様穴の歴史回収 |
| [2026-07-23-historical-d1l-constructor-lint-lineage-recovery.md](2026-07-23-historical-d1l-constructor-lint-lineage-recovery.md) | constructor／lintの歴史回収 |
| [2026-07-23-historical-d1l-counter-review-evidence-recovery.md](2026-07-23-historical-d1l-counter-review-evidence-recovery.md) | D1l反対側レビュー証拠の回収 |
| [2026-07-23-historical-d1l-journal-undo-lineage-recovery.md](2026-07-23-historical-d1l-journal-undo-lineage-recovery.md) | journal／Undoの歴史回収 |
| [2026-07-23-historical-d1m-sidecar-session-lineage-recovery.md](2026-07-23-historical-d1m-sidecar-session-lineage-recovery.md) | sidecar／sessionの歴史回収 |
| [2026-07-23-historical-d2-selection-timeline-lineage-recovery.md](2026-07-23-historical-d2-selection-timeline-lineage-recovery.md) | D2／selection／Timelineの歴史回収 |
| [2026-07-23-historical-d5-transport-lineage-recovery.md](2026-07-23-historical-d5-transport-lineage-recovery.md) | Transportの歴史回収 |
| [2026-07-23-historical-first-code-audit-lineage-recovery.md](2026-07-23-historical-first-code-audit-lineage-recovery.md) | 第一code auditの歴史回収 |
| [2026-07-23-historical-foundation-lineage-recovery.md](2026-07-23-historical-foundation-lineage-recovery.md) | 基盤文書の歴史回収 |
| [2026-07-23-historical-frame-desc-shared-types-lineage-recovery.md](2026-07-23-historical-frame-desc-shared-types-lineage-recovery.md) | FrameDesc共有型の歴史回収 |
| [2026-07-23-historical-llm-plugin-demo-lineage-recovery.md](2026-07-23-historical-llm-plugin-demo-lineage-recovery.md) | LLM plugin demo証拠の回収 |
| [2026-07-23-historical-m2-camera-contract-lineage-recovery.md](2026-07-23-historical-m2-camera-contract-lineage-recovery.md) | M2 camera契約の歴史回収 |
| [2026-07-23-historical-m2-entry-gate-lineage-recovery.md](2026-07-23-historical-m2-entry-gate-lineage-recovery.md) | M2入口gateの歴史回収 |
| [2026-07-23-historical-m2-reclosure-gate-lineage-recovery.md](2026-07-23-historical-m2-reclosure-gate-lineage-recovery.md) | M2再締結gateの歴史回収 |
| [2026-07-23-historical-m2-supplementary-review-lineage-recovery.md](2026-07-23-historical-m2-supplementary-review-lineage-recovery.md) | M2追補レビューの歴史回収 |
| [2026-07-23-historical-m4-cache-analysis-spec-lineage-recovery.md](2026-07-23-historical-m4-cache-analysis-spec-lineage-recovery.md) | M4 cache／analysisの歴史回収 |
| [2026-07-23-historical-media-portability-gpu-resurvey-plan-recovery.md](2026-07-23-historical-media-portability-gpu-resurvey-plan-recovery.md) | media可搬性／GPU再調査計画の回収 |
| [2026-07-23-historical-memory-model-lineage-recovery.md](2026-07-23-historical-memory-model-lineage-recovery.md) | memory modelの歴史回収 |
| [2026-07-23-historical-param-element-constraint-lineage-recovery.md](2026-07-23-historical-param-element-constraint-lineage-recovery.md) | Param／Element／Constraintの歴史回収 |
| [2026-07-23-historical-performance-model-lineage-recovery.md](2026-07-23-historical-performance-model-lineage-recovery.md) | performance modelの歴史回収 |
| [2026-07-23-historical-permanence-prevention-lineage-recovery.md](2026-07-23-historical-permanence-prevention-lineage-recovery.md) | 恒久焼き込み予防の歴史回収 |
| [2026-07-23-historical-plugin-authoring-lineage-recovery.md](2026-07-23-historical-plugin-authoring-lineage-recovery.md) | plugin authoringの歴史回収 |
| [2026-07-23-historical-plugin-ecosystem-lineage-recovery.md](2026-07-23-historical-plugin-ecosystem-lineage-recovery.md) | plugin ecosystemの歴史回収 |
| [2026-07-23-historical-plugin-resource-runtime-lineage-recovery.md](2026-07-23-historical-plugin-resource-runtime-lineage-recovery.md) | plugin resource／runtimeの歴史回収 |
| [2026-07-23-historical-plugin-ui-lineage-recovery.md](2026-07-23-historical-plugin-ui-lineage-recovery.md) | plugin UIの歴史回収 |
| [2026-07-23-historical-public-capability-provenance-lineage-recovery.md](2026-07-23-historical-public-capability-provenance-lineage-recovery.md) | 公開capability／provenanceの歴史回収 |
| [2026-07-23-historical-r1-export-gpu-safety-lineage-recovery.md](2026-07-23-historical-r1-export-gpu-safety-lineage-recovery.md) | R1 export／GPU safetyの歴史回収 |
| [2026-07-23-historical-r3-datatrack-export-correctness-lineage-recovery.md](2026-07-23-historical-r3-datatrack-export-correctness-lineage-recovery.md) | R3 DataTrack／export correctnessの歴史回収 |
| [2026-07-23-historical-r9-real-material-export-acceptance-lineage-recovery.md](2026-07-23-historical-r9-real-material-export-acceptance-lineage-recovery.md) | R9実素材／export受入の歴史回収 |
| [2026-07-23-historical-react-webview-lineage-recovery.md](2026-07-23-historical-react-webview-lineage-recovery.md) | React／WebViewの歴史回収 |
| [2026-07-23-historical-reclosure-counter-review-evidence-recovery.md](2026-07-23-historical-reclosure-counter-review-evidence-recovery.md) | 再締結反対側レビュー証拠の回収 |
| [2026-07-23-historical-render-ctx-thaw-lineage-recovery.md](2026-07-23-historical-render-ctx-thaw-lineage-recovery.md) | RenderCtx解凍の歴史回収 |
| [2026-07-23-historical-s2-decode-pipeline-lineage-recovery.md](2026-07-23-historical-s2-decode-pipeline-lineage-recovery.md) | S2 decode pipelineの歴史回収 |
| [2026-07-23-historical-semantic-oracle-boundary-recovery.md](2026-07-23-historical-semantic-oracle-boundary-recovery.md) | semantic oracle境界の歴史回収 |
| [2026-07-23-historical-shared-effect-lifecycle-lineage-recovery.md](2026-07-23-historical-shared-effect-lifecycle-lineage-recovery.md) | Shared Effect lifecycleの歴史回収 |
| [2026-07-23-historical-test-oracle-ruleset-recovery.md](2026-07-23-historical-test-oracle-ruleset-recovery.md) | test oracle rulesetの歴史回収 |
| [2026-07-23-historical-unified-stage-camera-ui-lineage-recovery.md](2026-07-23-historical-unified-stage-camera-ui-lineage-recovery.md) | Stage／Camera UIの歴史回収 |
| [2026-07-23-historical-vello-adoption-lineage-recovery.md](2026-07-23-historical-vello-adoption-lineage-recovery.md) | Vello採否の歴史回収 |
| [2026-07-23-historical-vism-a3-expression-layersource-lineage-recovery.md](2026-07-23-historical-vism-a3-expression-layersource-lineage-recovery.md) | Vism A3／LayerSourceの歴史回収 |
| [2026-07-23-historical-vism-foundation-contract-lineage-recovery.md](2026-07-23-historical-vism-foundation-contract-lineage-recovery.md) | Vism foundation contractの歴史回収 |
| [2026-07-23-historical-vism-kit-distribution-lineage-recovery.md](2026-07-23-historical-vism-kit-distribution-lineage-recovery.md) | Vism Kit／distributionの歴史回収 |
| [2026-07-23-historical-wgpu-readback-cold-compile-lineage-recovery.md](2026-07-23-historical-wgpu-readback-cold-compile-lineage-recovery.md) | wgpu readback／cold compileの歴史回収 |
| [2026-07-24-replaceable-semantic-seat-decision.md](2026-07-24-replaceable-semantic-seat-decision.md) | HVR-D04 Unit 8A — Host semantic seat、換装可能Provider、Effect／Filter分類とContent-Aware Scale候補 |
| [2026-07-26-cu-0a08i-inspector-read-model-split-decision.md](2026-07-26-cu-0a08i-inspector-read-model-split-decision.md) | CU-0A08I Inspector read-model再判定・分割決定 |
| [2026-07-26-cu-0a08is-inspector-read-model-inventory.md](2026-07-26-cu-0a08is-inspector-read-model-inventory.md) | CU-0A08IS Inspector read-model inventory・fixture拒否契約 |
| [2026-07-26-cu-g09-browser-catalog-projection-contract-decision.md](2026-07-26-cu-g09-browser-catalog-projection-contract-decision.md) | CU-G09 Browser catalog projection契約決定 |
| [2026-07-26-cu-g09o-browser-decoder-output-contract-decision.md](2026-07-26-cu-g09o-browser-decoder-output-contract-decision.md) | CU-G09O Browser decoder output契約決定 |
| [2026-07-26-cu-g09r-browser-decoder-rejection-precedence-decision.md](2026-07-26-cu-g09r-browser-decoder-rejection-precedence-decision.md) | CU-G09R Browser decoder拒否優先順決定 |
| [2026-07-26-cu-102-fresh-layerid-addtrackitem-atomicity-decision.md](2026-07-26-cu-102-fresh-layerid-addtrackitem-atomicity-decision.md) | CU-102 fresh LayerId + AddTrackItem原子性決定 |
| [2026-07-26-cu-g03-edit-durability-ordering-decision.md](2026-07-26-cu-g03-edit-durability-ordering-decision.md) | CU-G03 edit durability / publish順序決定 |
| [2026-07-26-u3a-1-headless-timeline-owner-visibility-split-decision.md](2026-07-26-u3a-1-headless-timeline-owner-visibility-split-decision.md) | U3a-1 headless Timeline owner/visibility分割決定 |
| [2026-07-27-cu-104-selection-publish-envelope-decision.md](2026-07-27-cu-104-selection-publish-envelope-decision.md) | CU-104 selection publish envelope決定 |
| [2026-07-27-u2h-1-primary-selection-implementation-split-decision.md](2026-07-27-u2h-1-primary-selection-implementation-split-decision.md) | U2h-1 primary selection implementation split決定 |
| [2026-07-27-cu-104e-projection-generation-exhaustion-decision.md](2026-07-27-cu-104e-projection-generation-exhaustion-decision.md) | CU-104E projection generation枯渇境界決定 |
| [2026-07-27-u2h-1p-selection-input-reachability-decision.md](2026-07-27-u2h-1p-selection-input-reachability-decision.md) | U2h-1P selection入力到達性決定 |
| [2026-07-27-cu-105-dense-timeline-responsibility-recheck.md](2026-07-27-cu-105-dense-timeline-responsibility-recheck.md) | CU-105 dense Timeline責任再確認 |
| [2026-07-27-cu-106-selection-consumer-split-decision.md](2026-07-27-cu-106-selection-consumer-split-decision.md) | CU-106 selection consumer分割決定 |
| [2026-07-27-u3a-2s-windowed-timeline-readiness-split-decision.md](2026-07-27-u3a-2s-windowed-timeline-readiness-split-decision.md) | U3a-2S windowed native Timeline readiness分割決定 |
| [2026-07-27-u3a-2r-renderer-adoption-scope-decision.md](2026-07-27-u3a-2r-renderer-adoption-scope-decision.md) | U3a-2R windowed native Timeline renderer採択範囲決定 |
| [2026-07-27-u3a-2z-semantic-zoom-responsibility-decision.md](2026-07-27-u3a-2z-semantic-zoom-responsibility-decision.md) | U3a-2Z windowed native Timeline semantic zoom責任所在決定 |
| [2026-07-27-u3a-2a-renderer-adoption-decision.md](2026-07-27-u3a-2a-renderer-adoption-decision.md) | U3a-2A windowed native Timeline renderer採択決定 |
| [2026-07-27-u3a-2p-playhead-visible-range-scope-decision.md](2026-07-27-u3a-2p-playhead-visible-range-scope-decision.md) | U3a-2P playhead / visible range owner 判断の範囲決定 |
| [2026-07-27-u3a-2q-playhead-visible-range-owner-split-decision.md](2026-07-27-u3a-2q-playhead-visible-range-owner-split-decision.md) | U3a-2Q playhead / visible range owner 採択の分割判断 |
| [2026-07-27-u3a-2q-p-playhead-owner-evidence-supplement.md](2026-07-27-u3a-2q-p-playhead-owner-evidence-supplement.md) | U3a-2Q-P playhead owner admissibility / evidence 補遺 |
| [2026-07-27-u3a-2q-p2-playhead-reopen-lifetime-decision.md](2026-07-27-u3a-2q-p2-playhead-reopen-lifetime-decision.md) | U3a-2Q-P2 playhead 再 open lifetime 決定 |
| [2026-07-27-u3a-2q-p3-playhead-future-restore-posture-decision.md](2026-07-27-u3a-2q-p3-playhead-future-restore-posture-decision.md) | U3a-2Q-P3 playhead 将来 reopen 復元 posture 決定 |
| [2026-07-27-u3a-2q-p4-playhead-five-layer-owner-adoption-decision.md](2026-07-27-u3a-2q-p4-playhead-five-layer-owner-adoption-decision.md) | U3a-2Q-P4 playhead 五層 state owner 採択 |
| [2026-07-27-cu-109s0-readiness-recheck-selection-decision.md](2026-07-27-cu-109s0-readiness-recheck-selection-decision.md) | CU-109S0 CU-109 readiness / order-boundary recheck 選定 |
| [2026-07-27-cu-109s-undo-redo-prepared-action-order-recheck.md](2026-07-27-cu-109s-undo-redo-prepared-action-order-recheck.md) | CU-109S Undo / Redo prepared-action 順序再確認 |
| [2026-07-27-cu-109sp-cu-111-prepared-action-order-prerequisite-decision.md](2026-07-27-cu-109sp-cu-111-prepared-action-order-prerequisite-decision.md) | CU-109SP CU-109 / CU-111 prepared-action 順序前提（P1 precedence） |
| [2026-07-27-cu-g04s0-session-source-selection-decision.md](2026-07-27-cu-g04s0-session-source-selection-decision.md) | CU-G04S0 VS-1 edit runtime ProjectSession source 判断の選定 |
| [2026-07-27-cu-g04s-edit-runtime-session-source-decision.md](2026-07-27-cu-g04s-edit-runtime-session-source-decision.md) | CU-G04S VS-1 edit runtime session source / no-session / interim action disposition |
| [2026-07-27-cu-g04sc0-product-path-handoff-selection-decision.md](2026-07-27-cu-g04sc0-product-path-handoff-selection-decision.md) | CU-G04SC0 VS-1 edit runtime product path handoff 判断の選定 |
| [2026-07-27-cu-g04sc-edit-runtime-product-path-handoff-decision.md](2026-07-27-cu-g04sc-edit-runtime-product-path-handoff-decision.md) | CU-G04SC VS-1 edit runtime product path handoff（argv carrier / entry境界 / fail-closed / test-flag降格） |
| [2026-07-28-cu-110s-dependency-scope-decision-selection.md](2026-07-28-cu-110s-dependency-scope-decision-selection.md) | CU-110S CU-110 前提範囲（CU-107 依存）判断の選定 |
| [2026-07-28-cu-110d-cu-107-dependency-scope-decision.md](2026-07-28-cu-110d-cu-107-dependency-scope-decision.md) | CU-110D CU-110 の CU-107 依存範囲裁定 |
| [2026-07-28-cu-107s-split-concretization-scope-selection.md](2026-07-28-cu-107s-split-concretization-scope-selection.md) | CU-107S CU-107 分割具体化範囲の選定 |
| [2026-07-28-cu-107d-cu-110-required-responsibility-scope-decision.md](2026-07-28-cu-107d-cu-110-required-responsibility-scope-decision.md) | CU-107D CU-110 が必要とする CU-107 責任範囲の先行限定 |
| [2026-07-28-cu-107r-cu-110-required-responsibility-decision.md](2026-07-28-cu-107r-cu-110-required-responsibility-decision.md) | CU-107R CU-110 が必要とする CU-107 責任範囲の限定裁定 |
| [2026-07-28-cu-107n-cu-107-narrow-prerequisite-closed-set.md](2026-07-28-cu-107n-cu-107-narrow-prerequisite-closed-set.md) | CU-107N CU-107 狭い名前付き前提の閉集合（4前提・単一 owner・依存順） |
| [2026-07-28-cu-107w-w0-mirror-rewrite-decision.md](2026-07-28-cu-107w-w0-mirror-rewrite-decision.md) | CU-107W W0表・CU-110依存の閉集合名反映（次PRODUCT-ASSET `DO`未選定） |
| [2026-07-28-g0-6h-e-candidate-approval-evidence-selection.md](2026-07-28-g0-6h-e-candidate-approval-evidence-selection.md) | G0-6H-E0 現行候補5画面承認証拠の限定取込選定 |
| [2026-07-28-g0-6h-e-candidate-approval-observation.md](2026-07-28-g0-6h-e-candidate-approval-observation.md) | G0-6H-E 現行候補5画面承認の限定観察 |
| [2026-07-28-g0-6h-r0-reference-authority-reconciliation-selection.md](2026-07-28-g0-6h-r0-reference-authority-reconciliation-selection.md) | G0-6H-R0 reference authority再照合粒の選定 |
| [2026-07-28-g0-6h-r-reference-authority-role-reconciliation-decision.md](2026-07-28-g0-6h-r-reference-authority-role-reconciliation-decision.md) | G0-6H-R reference authority役割再照合 |
| [2026-07-28-g0-6h-s-human-judgment-input-route-decision.md](2026-07-28-g0-6h-s-human-judgment-input-route-decision.md) | G0-6H-S 人間審判入力routeの裁定 |
| [2026-07-28-g0-6h-m0-current-route-semantic-gap-selection.md](2026-07-28-g0-6h-m0-current-route-semantic-gap-selection.md) | G0-6H-M0 現行route semantic gap確認粒の選定 |
| [2026-07-28-g0-6h-m-current-route-semantic-gap-mapping.md](2026-07-28-g0-6h-m-current-route-semantic-gap-mapping.md) | G0-6H-M 現行route element-level semantic gap map |
| [2026-07-28-g0-6h-a0-empty-project-starter-media-selection.md](2026-07-28-g0-6h-a0-empty-project-starter-media-selection.md) | G0-6H-A0 empty-project + Starter Media裁定の受領と契約粒選定 |
| [2026-07-28-g0-6h-a-empty-project-starter-media-scenario-contract.md](2026-07-28-g0-6h-a-empty-project-starter-media-scenario-contract.md) | G0-6H-A empty Project + local Starter Media scenario / fixture 所有契約 |
| [2026-07-28-g0-6h-af-starter-media-source-provenance-decision.md](2026-07-28-g0-6h-af-starter-media-source-provenance-decision.md) | G0-6H-AF Starter Media 媒体源・provenance class 裁定 |
| [2026-07-28-g0-6h-ag0-starter-media-generator-closure-inventory.md](2026-07-28-g0-6h-ag0-starter-media-generator-closure-inventory.md) | G0-6H-AG0 Starter Media generator / output closure 棚卸しと責任処分 |
| [2026-07-28-g0-6h-v0-current-route-variant-evidence-contract.md](2026-07-28-g0-6h-v0-current-route-variant-evidence-contract.md) | G0-6H-V0 現行route variant evidence契約 |
| [2026-07-28-g0-6h-v1s-current-route-capture-boundary-decision.md](2026-07-28-g0-6h-v1s-current-route-capture-boundary-decision.md) | G0-6H-V1S 現行route capture境界の裁定 |
| [2026-07-28-g0-6h-v1p-current-route-capture-prerequisite-decision.md](2026-07-28-g0-6h-v1p-current-route-capture-prerequisite-decision.md) | G0-6H-V1P 現行route capture前提の裁定 |
| [2026-07-28-g0-6h-v1r-envelope-generation-split-decision.md](2026-07-28-g0-6h-v1r-envelope-generation-split-decision.md) | G0-6H-V1R envelope / generation分割の裁定 |
| [2026-07-28-g0-6h-v1eta-empty-projection-staging-decision.md](2026-07-28-g0-6h-v1eta-empty-projection-staging-decision.md) | G0-6H-V1ETA empty projection段階化の裁定 |
| [2026-07-28-g0-6h-v1etb-h-browser-post-promotion-authority-reclosure-decision.md](2026-07-28-g0-6h-v1etb-h-browser-post-promotion-authority-reclosure-decision.md) | G0-6H-V1ETB-H Browser post-promotion authority再締結の裁定 |
| [2026-07-28-g0-6h-v1etb-p-browser-projection-consumer-capsule-boundary-decision.md](2026-07-28-g0-6h-v1etb-p-browser-projection-consumer-capsule-boundary-decision.md) | G0-6H-V1ETB-P Browser projection consumerとcapsule境界の裁定 |
| [2026-07-28-g0-6h-v1etb-q-browser-route-oracle-allowlist-correction-decision.md](2026-07-28-g0-6h-v1etb-q-browser-route-oracle-allowlist-correction-decision.md) | G0-6H-V1ETB-Q Browser route oracle allowlist補正の裁定 |
| [2026-07-29-g0-6h-v1g-c-p-current-route-capture-environment-authority-correction-decision.md](2026-07-29-g0-6h-v1g-c-p-current-route-capture-environment-authority-correction-decision.md) | G0-6H-V1G-C-P 現行route capture環境 authority 補正決定 |
| [2026-07-29-g0-6h-v1g-o-h-current-route-command-authority-hash-correction-decision.md](2026-07-29-g0-6h-v1g-o-h-current-route-command-authority-hash-correction-decision.md) | G0-6H-V1G-O-H 現行route command authority hash 補正決定 |
| [2026-07-29-g0-6h-v1g-p-current-route-generation-mechanics-decision.md](2026-07-29-g0-6h-v1g-p-current-route-generation-mechanics-decision.md) | G0-6H-V1G-P 現行route generation mechanics決定 |
| [2026-07-29-cu-0a08rs0-browser-inspector-read-projection-dependency-scope-selection.md](2026-07-29-cu-0a08rs0-browser-inspector-read-projection-dependency-scope-selection.md) | CU-0A08RS0 Browser / Inspector read-projection 依存範囲の選定 |
| [2026-07-29-cu-0a08rs-browser-inspector-read-projection-u4a2-dependency-decision.md](2026-07-29-cu-0a08rs-browser-inspector-read-projection-u4a2-dependency-decision.md) | CU-0A08RS Browser / Inspector read-only projection U4a-2 依存裁定 |
| [2026-07-29-cu-0a08rm0-browser-typed-intent-dependency-adjudication-scope-selection.md](2026-07-29-cu-0a08rm0-browser-typed-intent-dependency-adjudication-scope-selection.md) | CU-0A08RM0 Browser typed-intent 依存裁定範囲の選定 |
| [2026-07-29-cu-0a08rmd-browser-typed-intent-dependency-adjudication.md](2026-07-29-cu-0a08rmd-browser-typed-intent-dependency-adjudication.md) | CU-0A08RMD Browser typed-intent 依存裁定 |
| [2026-07-29-cu-0a08bd0-browser-typed-intent-dependency-direction-scope-selection.md](2026-07-29-cu-0a08bd0-browser-typed-intent-dependency-direction-scope-selection.md) | CU-0A08BD0 Browser typed-intent 依存方向の選定範囲 |
| [2026-07-29-cu-0a08bdd-browser-typed-intent-dependency-direction-decision.md](2026-07-29-cu-0a08bdd-browser-typed-intent-dependency-direction-decision.md) | CU-0A08BDD Browser typed-intent 依存方向の裁定 |
| [2026-07-29-cu-0a08btr-browser-read-projection-dependency-reclosure-decision.md](2026-07-29-cu-0a08btr-browser-read-projection-dependency-reclosure-decision.md) | CU-0A08BTR Browser read-projection 依存再締結 |
| [2026-07-29-cu-0a08btp-browser-read-projection-jsx-connection-implementation-decision.md](2026-07-29-cu-0a08btp-browser-read-projection-jsx-connection-implementation-decision.md) | CU-0A08BTP Browser read projection / JSX connection 実装決定 |
| [2026-07-29-cu-0a08itp-p-inspector-post-promotion-authority-amendment.md](2026-07-29-cu-0a08itp-p-inspector-post-promotion-authority-amendment.md) | CU-0A08ITP-P Inspector post-promotion authority 改訂 |
| [2026-07-29-cu-0a08itp-inspector-read-projection-jsx-connection-implementation-decision.md](2026-07-29-cu-0a08itp-inspector-read-projection-jsx-connection-implementation-decision.md) | CU-0A08ITP Inspector read projection / JSX connection 実装決定 |
| [2026-07-29-cu-0a08ss0-browser-place-source-seam-implementation-boundary-scope-selection.md](2026-07-29-cu-0a08ss0-browser-place-source-seam-implementation-boundary-scope-selection.md) | CU-0A08SS0 Browser Place source seam の最小実装境界 選定範囲 |
| [2026-07-29-cu-0a08ssd-browser-place-source-seam-implementation-boundary-decision.md](2026-07-29-cu-0a08ssd-browser-place-source-seam-implementation-boundary-decision.md) | CU-0A08SSD Browser Place source seam の最小実装境界 裁定 |
| [2026-07-29-cu-0a08ssc-browser-place-source-seam-contract-concretization-scope-selection.md](2026-07-29-cu-0a08ssc-browser-place-source-seam-contract-concretization-scope-selection.md) | CU-0A08SSC Browser Place source seam 契約具体化 選定範囲 |
| [2026-07-29-cu-0a08sscd-browser-place-source-seam-contract-concretization-decision.md](2026-07-29-cu-0a08sscd-browser-place-source-seam-contract-concretization-decision.md) | CU-0A08SSCD Browser Place source seam 契約具体化 裁定 |
| [2026-07-29-cu-0a08ssci-browser-place-source-seam-prerequisite-order-decision.md](2026-07-29-cu-0a08ssci-browser-place-source-seam-prerequisite-order-decision.md) | CU-0A08SSCI Browser Place source seam 前提順序 裁定 |
| [2026-07-29-cu-0a08ssci-i-browser-scoped-identity-input-seam-contract-shape-decision.md](2026-07-29-cu-0a08ssci-i-browser-scoped-identity-input-seam-contract-shape-decision.md) | CU-0A08SSCI-I Browser scoped identity input seam 契約形 裁定 |
| [2026-07-29-cu-0a08ssci-i0-browser-scoped-identity-input-seam-grain-numbering-decision.md](2026-07-29-cu-0a08ssci-i0-browser-scoped-identity-input-seam-grain-numbering-decision.md) | CU-0A08SSCI-I0 Browser scoped identity input seam grain numbering |
| [2026-07-29-cu-0a08ssci-i0-browser-scoped-identity-input-seam-scope-selection.md](2026-07-29-cu-0a08ssci-i0-browser-scoped-identity-input-seam-scope-selection.md) | CU-0A08SSCI-I0 Browser scoped identity input seam 選定 |
| [2026-07-29-cu-0a08ssci-t-browser-private-component-verification-harness-boundary-decision.md](2026-07-29-cu-0a08ssci-t-browser-private-component-verification-harness-boundary-decision.md) | CU-0A08SSCI-T Browser private component verification harness boundary |
| [2026-07-29-cu-0a08ssci-t1-browser-private-component-verification-harness-implementation-decision.md](2026-07-29-cu-0a08ssci-t1-browser-private-component-verification-harness-implementation-decision.md) | CU-0A08SSCI-T1 Browser private component verification harness 実装決定 |
| [2026-07-29-cu-0a08ssci-t0-browser-private-component-verification-harness-grain-numbering-decision.md](2026-07-29-cu-0a08ssci-t0-browser-private-component-verification-harness-grain-numbering-decision.md) | CU-0A08SSCI-T0 Browser private component verification harness grain numbering |
| [2026-07-29-cu-0a08ssci-p-browser-post-promotion-provenance-chain-authority-amendment.md](2026-07-29-cu-0a08ssci-p-browser-post-promotion-provenance-chain-authority-amendment.md) | CU-0A08SSCI-P Browser post-promotion provenance chain authority 改訂 |
| [2026-07-29-cu-0a08ssci-p1-browser-post-promotion-provenance-chain-guard-reconciliation-decision.md](2026-07-29-cu-0a08ssci-p1-browser-post-promotion-provenance-chain-guard-reconciliation-decision.md) | CU-0A08SSCI-P1 Browser post-promotion provenance chain guard 整合 |
| [2026-07-29-cu-0a08sscs-browser-place-source-seam-implementation-scope-selection.md](2026-07-29-cu-0a08sscs-browser-place-source-seam-implementation-scope-selection.md) | CU-0A08SSCS Browser Place source seam 実装範囲 選定 |
| [2026-07-29-cu-0a08sscsd-browser-place-source-seam-implementation-scope-decision.md](2026-07-29-cu-0a08sscsd-browser-place-source-seam-implementation-scope-decision.md) | CU-0A08SSCSD Browser Place source seam 実装範囲 裁定 |
| [2026-07-29-cu-0a09s-r6-surface-closure-boundary-decision.md](2026-07-29-cu-0a09s-r6-surface-closure-boundary-decision.md) | CU-0A09S R6 surface closure boundary 決定 |
| [2026-07-29-g0-6h-human-acceptance-decision.md](2026-07-29-g0-6h-human-acceptance-decision.md) | G0-6H 現行React UI 人間審判 ACCEPT |
| [2026-07-29-cu-0b02s-product-token-ownership-split-decision.md](2026-07-29-cu-0b02s-product-token-ownership-split-decision.md) | CU-0B02S 製品token所有と接続粒の分割決定 |
| [2026-07-29-cu-0b02t-product-token-authority-implementation-decision.md](2026-07-29-cu-0b02t-product-token-authority-implementation-decision.md) | CU-0B02T 製品token単一authority実装決定 |
| [2026-07-29-cu-0a08bti-browser-place-typed-intent-implementation-decision.md](2026-07-29-cu-0a08bti-browser-place-typed-intent-implementation-decision.md) | CU-0A08BTI Browser Place typed intent実装決定 |
| [2026-07-29-cu-0b03h-browser-host-contract-offline-mount-decision.md](2026-07-29-cu-0b03h-browser-host-contract-offline-mount-decision.md) | CU-0B03H Browser Host契約・offline mount決定 |
| [2026-07-29-cu-0b03-native-browser-host-codec-inbox-implementation-decision.md](2026-07-29-cu-0b03-native-browser-host-codec-inbox-implementation-decision.md) | CU-0B03 native Browser Host codec/inbox実装決定 |
| [2026-07-29-cu-0b04s-browser-native-place-terminal-ownership-reclosure-decision.md](2026-07-29-cu-0b04s-browser-native-place-terminal-ownership-reclosure-decision.md) | CU-0B04S Browser→native Place終端所有の再締結 |
| [2026-07-29-cu-0b04p-host-platform-pointer-capture-implementation-decision.md](2026-07-29-cu-0b04p-host-platform-pointer-capture-implementation-decision.md) | CU-0B04P Host platform pointer capture実装決定 |
| [2026-07-29-cu-0b04na-product-window-lifecycle-adapter-boundary-decision.md](2026-07-29-cu-0b04na-product-window-lifecycle-adapter-boundary-decision.md) | CU-0B04NA product window lifecycle adapter境界決定 |
| [2026-07-29-cu-0b04n-native-stage-surface-layout-implementation-decision.md](2026-07-29-cu-0b04n-native-stage-surface-layout-implementation-decision.md) | CU-0B04N native Stage Surface / layout epoch実装決定 |
| [2026-07-29-cu-0b04r-browser-island-focus-geometry-implementation-decision.md](2026-07-29-cu-0b04r-browser-island-focus-geometry-implementation-decision.md) | CU-0B04R Browser island focus / geometry epoch実装決定 |
| [2026-07-29-cu-0b05s-browser-lifecycle-reprojection-contract-decision.md](2026-07-29-cu-0b05s-browser-lifecycle-reprojection-contract-decision.md) | CU-0B05S Browser lifecycle再投影契約決定 |
| [2026-07-29-cu-0b05-browser-lifecycle-reprojection-implementation-decision.md](2026-07-29-cu-0b05-browser-lifecycle-reprojection-implementation-decision.md) | CU-0B05 Browser lifecycle再投影実装決定 |
| [2026-07-29-cu-107pv-place-preview-delivery-implementation-decision.md](2026-07-29-cu-107pv-place-preview-delivery-implementation-decision.md) | CU-107PV Place preview配送実装決定 |
| [2026-07-29-cu-107tc-place-terminal-cause-classification-implementation-decision.md](2026-07-29-cu-107tc-place-terminal-cause-classification-implementation-decision.md) | CU-107TC Place候補terminal原因分類実装決定 |
| [2026-07-29-cu-107ad-place-terminal-admission-implementation-decision.md](2026-07-29-cu-107ad-place-terminal-admission-implementation-decision.md) | CU-107AD Place候補terminal admission実装決定 |
| [2026-07-29-cu-107td-place-terminal-delivery-implementation-decision.md](2026-07-29-cu-107td-place-terminal-delivery-implementation-decision.md) | CU-107TD Place terminal配送実装決定 |
| [2026-07-29-cu-107-place-coordinator-parent-closure-decision.md](2026-07-29-cu-107-place-coordinator-parent-closure-decision.md) | CU-107 Place coordinator親閉鎖決定 |
| [2026-07-29-cu-110-product-place-d2-commit-implementation-decision.md](2026-07-29-cu-110-product-place-d2-commit-implementation-decision.md) | CU-110 通常製品Place D2 commit接続実装決定 |
| [2026-07-29-cu-110p-product-published-snapshot-projection-split-decision.md](2026-07-29-cu-110p-product-published-snapshot-projection-split-decision.md) | CU-110P 通常製品published snapshot投影の分割決定 |
| [2026-07-29-cu-110ps-native-stage-published-snapshot-projection-implementation-decision.md](2026-07-29-cu-110ps-native-stage-published-snapshot-projection-implementation-decision.md) | CU-110PS native Stage published snapshot投影 実装決定 |
| [2026-07-29-cu-110pt0-native-timeline-projection-envelope-decision.md](2026-07-29-cu-110pt0-native-timeline-projection-envelope-decision.md) | CU-110PT0 native Timeline投影envelope決定 |
| [2026-07-29-cu-110pt-native-timeline-published-snapshot-projection-implementation-decision.md](2026-07-29-cu-110pt-native-timeline-published-snapshot-projection-implementation-decision.md) | CU-110PT native Timeline published snapshot投影 実装決定 |
| [2026-07-30-native-timeline-product-asset-transfer-implementation-decision.md](2026-07-30-native-timeline-product-asset-transfer-implementation-decision.md) | native Timeline比較面の通常製品window移管 実装決定 |
| [2026-07-29-cu-110pi-inspector-product-connection-split-decision.md](2026-07-29-cu-110pi-inspector-product-connection-split-decision.md) | CU-110PI Inspector通常製品接続 分割決定 |
| [2026-07-29-cu-110pir-inspector-safe-read-only-branch-implementation-decision.md](2026-07-29-cu-110pir-inspector-safe-read-only-branch-implementation-decision.md) | CU-110PIR Inspector safe read-only branch 実装決定 |
| [2026-07-30-cu-110pih-inspector-host-island-projection-implementation-decision.md](2026-07-30-cu-110pih-inspector-host-island-projection-implementation-decision.md) | CU-110PIH Inspector Host island projection 実装決定 |
| [2026-07-30-cu-106p-native-timeline-primary-selection-implementation-decision.md](2026-07-30-cu-106p-native-timeline-primary-selection-implementation-decision.md) | CU-106P native Timeline primary selection 実装決定 |
| [2026-07-30-cu-111-product-undo-redo-implementation-decision.md](2026-07-30-cu-111-product-undo-redo-implementation-decision.md) | CU-111 製品Undo/Redo配送 実装決定 |
| [2026-07-30-cu-108-rectangle-product-spine-e2e-decision.md](2026-07-30-cu-108-rectangle-product-spine-e2e-decision.md) | CU-108 Rectangle通常製品spine E2E決定 |
| [2026-07-30-cu-108rds-drop-release-routing-repair-selection.md](2026-07-30-cu-108rds-drop-release-routing-repair-selection.md) | CU-108RDS drop release排他routing修復選定と次ゴールhandoff |
| [2026-07-30-cu-108-product-connection-human-acceptance-observation.md](2026-07-30-cu-108-product-connection-human-acceptance-observation.md) | CU-108 通常製品接続の人間受け入れ観察 |
| [2026-07-30-sd-02g-product-host-layout-geometry-implementation-decision.md](2026-07-30-sd-02g-product-host-layout-geometry-implementation-decision.md) | SD-02G product Host layout geometry単一owner実装 |
| [2026-07-31-repository-validation-topology-decision.md](2026-07-31-repository-validation-topology-decision.md) | Cargo単独完了ownerをRust laneへ限定し、repository検証と外部審判を分離する決定 |
| [2026-08-01-supervision-loop-cost-driver-observation.md](2026-08-01-supervision-loop-cost-driver-observation.md) | 監督ループの速度支配項と計装(rework支配・文献監査・銀の弾丸不在) |
| [2026-08-01-m5-3d-import-rendering-boundary-decision.md](2026-08-01-m5-3d-import-rendering-boundary-decision.md) | M5 3Dインポート／レンダリング境界決定 |
| [2026-08-02-supervised-runner-retirement-decision.md](2026-08-02-supervised-runner-retirement-decision.md) | Motolii独自監督runnerを非破壊的に廃止し、Agentex共通入口の実地検証範囲と未閉鎖を固定する決定 |
| [2026-08-03-thin-observed-cli-harness-decision.md](2026-08-03-thin-observed-cli-harness-decision.md) | Claude Code、Codex CLI、Cursor Agentをexact argvと生logだけで接続するtransport-only harness決定 |
| [2026-08-03-runner-independent-supervision-decision.md](2026-08-03-runner-independent-supervision-decision.md) | 旧route/order/receipt状態機械を撤回し、scope・worktree・検収・採否をCodexへ戻す監督責任決定 |
| [2026-08-03-history-calibrated-llm-role-selection-decision.md](2026-08-03-history-calibrated-llm-role-selection-decision.md) | Sol総監督、Spark初回機械施工、Luna同一境界修正、一粒／短wave session、別family検収を固定routeなしで選ぶ決定 |
| [2026-08-03-claude-low-closed-review-calibration-observation.md](2026-08-03-claude-low-closed-review-calibration-observation.md) | Claude lowをCLOSED reviewへ使うbounded合成packet／保存済み過去diff再現とCLI turn制御の実測観察 |
| [2026-08-03-blind-evidence-envelope-counterexample-observation.md](2026-08-03-blind-evidence-envelope-counterexample-observation.md) | 単一blind envelopeの速度比較と全hit inventory／EVIDENCE_GAP／fresh waveによる選択バイアス反例捕捉の観察 |
| [2026-08-03-renewal-branch-reconciliation-handoff.md](2026-08-03-renewal-branch-reconciliation-handoff.md) | main統合後のbranch／worktreeを非破壊で分類し、renewal後のDO候補とfresh開始手順を固定する引継ぎsnapshot |
| [2026-08-03-llm-large-repository-context-routing-prior-art-observation.md](2026-08-03-llm-large-repository-context-routing-prior-art-observation.md) | LLM大規模repository開発の短い入口地図、hybrid retrieval、dependency graph、long-context昇格を比較し、Motolii向けauthority-aware mapの検証仮説を置く観察 |
| [2026-07-31-cu-0b02c-component-state-source-supply-decision.md](2026-07-31-cu-0b02c-component-state-source-supply-decision.md) | CU-0B02C component state source / supply裁定 |
| [2026-07-31-cu-0b02cv-private-carry-final-disposition.md](2026-07-31-cu-0b02cv-private-carry-final-disposition.md) | CU-0B02C-V component-private carry最終処分 |
| [2026-07-31-cu-203-feedback-source-ownership-split-decision.md](2026-07-31-cu-203-feedback-source-ownership-split-decision.md) | CU-203 共通feedback source / ownership分割決定 |
| [2026-07-31-cu-204-staged-diagnostic-projection-split-decision.md](2026-07-31-cu-204-staged-diagnostic-projection-split-decision.md) | CU-204 段階診断投影 S/A/P分割決定 |
| [2026-07-31-cu-204a-diagnostic-projection-adapter-implementation-decision.md](2026-07-31-cu-204a-diagnostic-projection-adapter-implementation-decision.md) | CU-204A 純粋段階診断投影adapter実装決定 |
| [2026-07-31-cu-205s-opacity-direct-route-split-decision.md](2026-07-31-cu-205s-opacity-direct-route-split-decision.md) | CU-205S first-party Opacity Direct通常製品route分割決定 |
| [2026-08-01-cu-205e-opacity-normal-product-route-e2e-receipt.md](2026-08-01-cu-205e-opacity-normal-product-route-e2e-receipt.md) | CU-205E Opacity通常製品route E2E receipt |
| [2026-08-01-cu-204p-normal-source-readiness-recheck.md](2026-08-01-cu-204p-normal-source-readiness-recheck.md) | CU-204P 通常製品source到達性の再確認 |
| [2026-08-01-cu-201-u3b-move-trim-snap-responsibility-split-decision.md](2026-08-01-cu-201-u3b-move-trim-snap-responsibility-split-decision.md) | CU-201 U3b move/trim/snap責任分割決定 |
| [2026-08-01-cu-201m-s-clip-start-command-contract-decision.md](2026-08-01-cu-201m-s-clip-start-command-contract-decision.md) | CU-201M-S Clip start command契約決定 |
| [2026-08-01-cu-201t-s-clip-trim-timemap-contract-decision.md](2026-08-01-cu-201t-s-clip-trim-timemap-contract-decision.md) | CU-201T-S Clip trim / TimeMap契約決定 |
| [2026-08-03-cu-201n-s-snap-target-contract-decision.md](2026-08-03-cu-201n-s-snap-target-contract-decision.md) | CU-201N-S snap target / priority / unit契約決定 |
| [2026-08-03-cu-201p-target-gap-observation.md](2026-08-03-cu-201p-target-gap-observation.md) | CU-201P native Timeline gesture target gap観察 |
| [2026-08-03-p06-c1-mac-rfd-adoption-probe-observation.md](2026-08-03-p06-c1-mac-rfd-adoption-probe-observation.md) | P06-C1-MAC rfd採択probeと固定Mac parent/selection/Cancel/typed failure外部gate PASSの観察 |
| [2026-08-03-cu-201p-move-known-semantics-adoption-decision.md](2026-08-03-cu-201p-move-known-semantics-adoption-decision.md) | CU-201P-MOVE native Timeline body-drag既知意味採択・move-only縮小決定 |
| [2026-08-03-cu-201p-trim-edge-known-semantics-adoption-decision.md](2026-08-03-cu-201p-trim-edge-known-semantics-adoption-decision.md) | CU-201P-TRIM Blender VSE handle hit規則のPATTERN採択・in/out trim target決定 |
| [2026-08-04-cu-201p-host-input-spine-decision.md](2026-08-04-cu-201p-host-input-spine-decision.md) | CU-201P通常Product Host入力背骨・logical Escape cancel既知実装採択決定 |
| [2026-08-04-cu-201p-host-input-implementation-acceptance.md](2026-08-04-cu-201p-host-input-implementation-acceptance.md) | CU-201P-HOST-INPUT実装受入・MOVE再締結・TRIM再開・M3最終HUMAN集約 |
| [2026-08-04-cu-201p-trim-implementation-acceptance.md](2026-08-04-cu-201p-trim-implementation-acceptance.md) | CU-201P-TRIM実装受入・独立検収・CU-201R開始・M3最終HUMAN集約 |
| [2026-08-04-cu-201r-random-move-trim-oracle-decision.md](2026-08-04-cu-201r-random-move-trim-oracle-decision.md) | CU-201R既存proptest系列採択・no-ripple/identity/全Undo oracle固定 |
| [2026-08-04-cu-201r-random-move-trim-oracle-acceptance.md](2026-08-04-cu-201r-random-move-trim-oracle-acceptance.md) | CU-201R 2,048-step実装・独立review受入・CU-201E開始 |
| [2026-08-04-cu-201e-normal-product-route-e2e-receipt.md](2026-08-04-cu-201e-normal-product-route-e2e-receipt.md) | CU-201E 通常製品move/trim/reopen E2E PASS（pointer-loss分離） |
| [2026-08-04-outcome-spine-autonomous-gap-research-decision.md](2026-08-04-outcome-spine-autonomous-gap-research-decision.md) | 利用者成果の背骨・調査不足粒の自律再検索・REMAP/REDUCE・M3 HUMAN最終集約 |
| [2026-08-06-storage-to-gpu-direct-io-design-observation.md](2026-08-06-storage-to-gpu-direct-io-design-observation.md) | cuFile／xio-sig一次資料からartifact identityをCore、storage→GPU到達経路をHost private policyへ分離する設計原則、非証明範囲、再入場条件を固定 |
| [2026-08-04-u4b0-durable-position-key-closed-contract.md](2026-08-04-u4b0-durable-position-key-closed-contract.md) | U4b-0 Position専用durable command・Bezier分割・journal v2据え置きの実装前closed contract |
| [2026-08-04-u4b0v-position-key-value-edit-contract.md](2026-08-04-u4b0v-position-key-value-edit-contract.md) | U4b-0V explicit Add後のexact on-key Vec2 value edit・dedicated D2・React Inspector gesture closed contract |
| [2026-08-04-u4b0v-position-key-value-edit-implementation-acceptance.md](2026-08-04-u4b0v-position-key-value-edit-implementation-acceptance.md) | U4b-0V React Inspector X/Y・key-local CAS・preview・one durable terminal・Undo/Redo/reopenのcode/main受入 |
| [2026-08-04-inspector-position-key-product-entry-reclosure.md](2026-08-04-inspector-position-key-product-entry-reclosure.md) | Inspector Position行をAdd Position Key通常入口へ選定し、normal row/current-playhead carrier不在をlocal WAIT_TARGETへ再締結 |
| [2026-08-04-inspector-position-row-direct-promotion-contract.md](2026-08-04-inspector-position-row-direct-promotion-contract.md) | Inspector Position行のConst(Vec2) read-only projectionを既存product source内で直接昇格し、intent/queueを別WAIT_TARGETへ分離 |
| [2026-08-04-inspector-position-row-implementation-acceptance.md](2026-08-04-inspector-position-row-implementation-acceptance.md) | CU-0A08ITIA finite Const X/Y・tag-only animated・inert同一source row・provenanceのcode/main受入 |
| [2026-08-04-inspector-position-key-one-shot-intent-contract.md](2026-08-04-inspector-position-key-one-shot-intent-contract.md) | CU-0A08ITIBをsequence-only private one-shot、separate Host inbox、current primary/playhead、既存prepareへ閉じた契約 |
| [2026-08-04-inspector-position-key-one-shot-intent-implementation-acceptance.md](2026-08-04-inspector-position-key-one-shot-intent-implementation-acceptance.md) | CU-0A08ITIB adjacent affordance・separate FIFO・Wake時current primary/playhead・existing durable routeのcode/main受入 |
| [2026-08-04-native-timeline-editor-playhead-contract.md](2026-08-04-native-timeline-editor-playhead-contract.md) | P02-C3 native Timeline editor playhead producer/carrier契約 |
| [2026-08-04-native-timeline-editor-playhead-implementation-acceptance.md](2026-08-04-native-timeline-editor-playhead-implementation-acceptance.md) | P02-C3 native Timeline editor playhead ruler producer/carrier実装受入 |
| [2026-08-04-p07-c1-playback-session-product-route-preflight.md](2026-08-04-p07-c1-playback-session-product-route-preflight.md) | P07-C1 mixed AudioProgram / PlaybackSession 製品routeの4経路 preflight・TARGET_MISSING |
| [2026-08-04-p07-c1a-video-only-program-supply-contract.md](2026-08-04-p07-c1a-video-only-program-supply-contract.md) | P07-C1A zero-source AudioProgramへ既存composition durationを供給floorとして再利用するclosed contract |
| [2026-08-04-p07-c1b-mixed-playback-session-contract.md](2026-08-04-p07-c1b-mixed-playback-session-contract.md) | P07-C1B existing PlaybackSessionをAudioProgram/MixProducerへ置換するadapter contract |
| [2026-08-04-p07-c1c-playback-origin-clock-contract.md](2026-08-04-p07-c1c-playback-origin-clock-contract.md) | P07-C1C nonZERO start_frameをexisting Transport sole clock originへ運ぶclosed contract |
| [2026-08-04-p07-c1d-product-playback-spine-contract.md](2026-08-04-p07-c1d-product-playback-spine-contract.md) | P07-C1D React Stage playからProductApp-owned PlaybackSessionとaudio-clock current timeを接続するclosed contract |
| [2026-08-04-p07-c1d-product-playback-spine-implementation-acceptance.md](2026-08-04-p07-c1d-product-playback-spine-implementation-acceptance.md) | P07-C1D typed Stage play・one ProductApp PlaybackSession・Transport sole clock・native Timeline投影のcode/main受入 |
| [2026-08-04-position-active-interval-read-model-contract.md](2026-08-04-position-active-interval-read-model-contract.md) | P04-C2 ACTIVE-INTERVAL Position read-model契約 |
| [2026-08-04-interp-command-d2-contract.md](2026-08-04-interp-command-d2-contract.md) | P04-C2 INTERP-COMMAND Position outgoing Interp D2 command契約 |
| [2026-08-04-interp-command-d2-implementation-acceptance.md](2026-08-04-interp-command-d2-implementation-acceptance.md) | P04-C2 INTERP-COMMAND D2実装受入。dedicated command/Writer/Undo/journal replayをDONE / ACCEPTED |
| [2026-08-04-p04-c2-easing-product-route-contract.md](2026-08-04-p04-c2-easing-product-route-contract.md) | P04-C2 React Easing trigger→surface-local Host popup session→existing queue/D2の通常製品route契約 |
| [2026-08-04-p04-c2-easing-c7a-implementation-acceptance.md](2026-08-04-p04-c2-easing-c7a-implementation-acceptance.md) | P04-C2-EASING-C7A React anchor/layout→private child egui popup→Position-only D2 のcode/main受入・M3-final外部gate集約 |
| [2026-08-04-p04-c2-diagnostic-correction-implementation-acceptance.md](2026-08-04-p04-c2-diagnostic-correction-implementation-acceptance.md) | P04-C2 diagnostic `SetPositionKeyInterp` label correction実装受入・DONE / ACCEPTED |
| [2026-08-04-stage-transport-easing-trigger-consumer-contract.md](2026-08-04-stage-transport-easing-trigger-consumer-contract.md) | P04-C2 ACTIVE-INTERVAL のStage transport既存slot→Easing trigger read-only consumer契約 |
| [2026-08-04-stage-transport-easing-trigger-implementation-acceptance.md](2026-08-04-stage-transport-easing-trigger-implementation-acceptance.md) | P04-C2 ACTIVE-INTERVAL Stage transport Easing trigger実装受入・EXTERNAL_GATE_PENDING |
| [2026-08-04-position-active-interval-implementation-admissibility-rejection.md](2026-08-04-position-active-interval-implementation-admissibility-rejection.md) | P04-C2 ACTIVE-INTERVAL compiler oracleによるconsumer不在の歴史観察・REMAPPED |
| [2026-08-10-out-of-repo-recovery-and-docs-drift-audit.md](2026-08-10-out-of-repo-recovery-and-docs-drift-audit.md) | リポ外資産回収・docs乖離監査。MotoliiRnProbe回収2割/skia-timeline-probe回収0%%、リポ外パス参照13本、台帳・地図の08-09/08-10乖離、退役テスト復活はgit実体ゼロ |
| [2026-08-10-main-merge-friction-removal-decision.md](2026-08-10-main-merge-friction-removal-decision.md) | mainマージ条件から全検証段差を撤廃、laneは事後観測・fix-forwardへ降格。虚偽green報告禁止とtest意味保護は維持 |
| [2026-08-01-cu-201e-timeline-interval-normal-product-e2e-receipt.md](2026-08-01-cu-201e-timeline-interval-normal-product-e2e-receipt.md) | CU-201E Timeline interval通常製品route E2E receipt(2026-08-10歴史回収: 旧M3 local-alpha線ブランチから。route水準の現行authorityは2026-08-07 RN再基線後のledger・実行地図が正) |
| [2026-08-01-cu-201n-s-timeline-snap-contract-decision.md](2026-08-01-cu-201n-s-timeline-snap-contract-decision.md) | CU-201N-S Timeline snap契約決定(2026-08-10歴史回収: 旧M3 local-alpha線ブランチから。route水準の現行authorityは2026-08-07 RN再基線後のledger・実行地図が正) |
| [2026-08-01-cu-201p-native-timeline-interval-gesture-implementation-decision.md](2026-08-01-cu-201p-native-timeline-interval-gesture-implementation-decision.md) | CU-201P native Timeline interval gesture 実装決定(2026-08-10歴史回収: 旧M3 local-alpha線ブランチから。route水準の現行authorityは2026-08-07 RN再基線後のledger・実行地図が正) |
| [2026-08-01-cu-201r-timeline-interval-sequence-oracle-decision.md](2026-08-01-cu-201r-timeline-interval-sequence-oracle-decision.md) | CU-201R Timeline interval系列oracle決定(2026-08-10歴史回収: 旧M3 local-alpha線ブランチから。route水準の現行authorityは2026-08-07 RN再基線後のledger・実行地図が正) |
| [2026-08-02-cu-206c-ordinary-timeline-viewport-implementation-decision.md](2026-08-02-cu-206c-ordinary-timeline-viewport-implementation-decision.md) | CU-206C ordinary Timeline viewport接続実装(2026-08-10歴史回収: 旧M3 local-alpha線ブランチから。route水準の現行authorityは2026-08-07 RN再基線後のledger・実行地図が正) |
| [2026-08-02-cu-210p-paused-playhead-product-connection-decision.md](2026-08-02-cu-210p-paused-playhead-product-connection-decision.md) | CU-210P paused playhead 製品接続決定(2026-08-10歴史回収: 旧M3 local-alpha線ブランチから。route水準の現行authorityは2026-08-07 RN再基線後のledger・実行地図が正) |
| [2026-08-02-cu-210r-video-playback-product-connection-implementation-decision.md](2026-08-02-cu-210r-video-playback-product-connection-implementation-decision.md) | CU-210R video-only playback 製品接続実装決定(2026-08-10歴史回収: 旧M3 local-alpha線ブランチから。route水準の現行authorityは2026-08-07 RN再基線後のledger・実行地図が正) |
| [2026-08-02-cu-211-project-export-local-alpha-decision.md](2026-08-02-cu-211-project-export-local-alpha-decision.md) | CU-211 — Local Alpha Project Save / reopen / Export 接続決定(2026-08-10歴史回収: 旧M3 local-alpha線ブランチから。route水準の現行authorityは2026-08-07 RN再基線後のledger・実行地図が正) |
| [2026-08-02-cu-212-mixed-audio-playback-product-connection-decision.md](2026-08-02-cu-212-mixed-audio-playback-product-connection-decision.md) | CU-212 — mixed AudioProgram playback 製品接続決定(2026-08-10歴史回収: 旧M3 local-alpha線ブランチから。route水準の現行authorityは2026-08-07 RN再基線後のledger・実行地図が正) |
| [2026-08-02-m3-global-chrome-settings-export-semantics-decision.md](2026-08-02-m3-global-chrome-settings-export-semantics-decision.md) | M3 global chrome / Settings / recovery / Export 接続決定(2026-08-10歴史回収: 旧M3 local-alpha線ブランチから。route水準の現行authorityは2026-08-07 RN再基線後のledger・実行地図が正) |
| [2026-08-02-m3-local-alpha-goal-line-owner-inventory.md](2026-08-02-m3-local-alpha-goal-line-owner-inventory.md) | M3 固定Mac Local Alpha ゴール線 owner 棚卸し(2026-08-10歴史回収: 旧M3 local-alpha線ブランチから。route水準の現行authorityは2026-08-07 RN再基線後のledger・実行地図が正) |
| [2026-08-02-u3a-2q-v-ordinary-timeline-viewport-decision.md](2026-08-02-u3a-2q-v-ordinary-timeline-viewport-decision.md) | U3a-2Q-V ordinary Timeline viewport決定(2026-08-10歴史回収: 旧M3 local-alpha線ブランチから。route水準の現行authorityは2026-08-07 RN再基線後のledger・実行地図が正) |
| [2026-08-02-u4b0-add-position-key-closed-contract-decision.md](2026-08-02-u4b0-add-position-key-closed-contract-decision.md) | U4b-0 Add Position Key closed contract決定(2026-08-10歴史回収: 旧M3 local-alpha線ブランチから。route水準の現行authorityは2026-08-07 RN再基線後のledger・実行地図が正) |
| [2026-08-02-u4b0p-product-add-position-key-connection-decision.md](2026-08-02-u4b0p-product-add-position-key-connection-decision.md) | U4b-0P Add Position Key 通常製品接続決定(2026-08-10歴史回収: 旧M3 local-alpha線ブランチから。route水準の現行authorityは2026-08-07 RN再基線後のledger・実行地図が正) |
| [2026-08-02-u4b1-outgoing-interp-command-contract-decision.md](2026-08-02-u4b1-outgoing-interp-command-contract-decision.md) | U4b-1 outgoing Interp command契約決定(2026-08-10歴史回収: 旧M3 local-alpha線ブランチから。route水準の現行authorityは2026-08-07 RN再基線後のledger・実行地図が正) |
| [2026-08-02-u4b1-product-easing-connection-implementation-decision.md](2026-08-02-u4b1-product-easing-connection-implementation-decision.md) | U4b-1 製品Easing接続 実装記録(2026-08-10歴史回収: 旧M3 local-alpha線ブランチから。route水準の現行authorityは2026-08-07 RN再基線後のledger・実行地図が正) |
| [2026-08-05-m3-dock-and-native-drag-technology-transfer-decision.md](2026-08-05-m3-dock-and-native-drag-technology-transfer-decision.md) | M3 Dock Host / OS native drag技術移管決定(2026-08-10歴史回収: ローカル専用ブランチから。後続決定がある場合はそちらが正) |
| [2026-07-27-wgsl-hot-reload-author-journey.md](2026-07-27-wgsl-hot-reload-author-journey.md) | WGSL hot reload作者経路 — INF-8具体化(2026-08-10歴史回収: ローカル専用ブランチから。後続決定がある場合はそちらが正) |
| [2026-07-22-all-docs-reclosure-inventory.md](2026-07-22-all-docs-reclosure-inventory.md) | 全docs再締結監査・第0単位 — read-only棚卸し報告(REWOR(2026-08-10歴史回収: ローカル専用ブランチから。後続決定がある場合はそちらが正) |
| [2026-07-20-m3-browser-panel-egui-taffy-spike.md](2026-07-20-m3-browser-panel-egui-taffy-spike.md) | M3 Browser panel egui/taffy spike観察(2026-08-10歴史回収: ローカル専用ブランチから。egui方向は2026-08-07 RN再基線で退役済み) |
| [2026-08-10-session-handoff-friction-removal-and-recovery.md](2026-08-10-session-handoff-friction-removal-and-recovery.md) | 段差撤廃・歴史回収セッションの引き継ぎ。回収一覧、rescue保全、未完の次手(M2-ASSET-1C capsule設計ほか)、発注実務メモ |
| [2026-08-10-session-handoff-node-registration-and-skia-timeline-in-rn-probe.md](2026-08-10-session-handoff-node-registration-and-skia-timeline-in-rn-probe.md) | 完成条件を塞ぐ8件のnode化と、実装なぞりによる自己訂正3件(media鎖は端から端まで製品コード0、file dialogの席1件へ収束)、RN probe timelineのSkia差し替えとvisual reference所在。仮コード先行を止め1つずつ繋ぐ方針へ変更 |
| [2026-08-12-set-position-key-time-contract.md](2026-08-12-set-position-key-time-contract.md) | SetPositionKeyTime(position key時刻移動)の閉じた契約 — SetPositionKeyValue CAS族の鏡映、native Timeline keydragが通常入口。Rerun原本にkeyframe編集不在を確認済み |
| [2026-08-12-remove-position-key-contract.md](2026-08-12-remove-position-key-contract.md) | RemovePositionKey(position key削除)の閉じた契約 — Add/UndoAdd対の鏡映、最後の1個はConst収束、undoは同一KeyframeId復元。Timeline Deleteが通常入口 |
| [2026-08-12-pre-handson-ux-decision-demotion.md](2026-08-12-pre-handson-ux-decision-demotion.md) | 実機以前のUX決定を仮説へ一括降格(利用者裁定)。object bar read-onlyを明示撤回、UX authorityは実機裁定>品質バー>文法地図>旧仮説。工学契約(絶対規律/D2)は対象外 |
| [2026-08-13-pr476-structural-skepticism-audit.md](2026-08-13-pr476-structural-skepticism-audit.md) | PR #476全成果物の構造懐疑監査S1〜S12(利用者裁定「見た目だけ通ればいい話ではない」)。最重症=Stage gizmo迂回(採択済みtransform_gizmo資産をhost接続で迂回し実UXが退化)。検収体系の欠陥(見た目保存の報酬化)を根本原因と認定、PRIOR ART欄義務化を即日施行 |
| [2026-08-14-user-palette-library-contract.md](2026-08-14-user-palette-library-contract.md) | Paletteをproject横断User Settings、適用RGBAを既存Document Color、Stage表示を既存Rerun Spatial Viewer投影へ分離する実データ契約 |
| [2026-08-17-rerun-layer-display-seat-measurement.md](2026-08-17-rerun-layer-display-seat-measurement.md) | Vism出力の透明レイヤー表示座席をGridMap→RectangleRendererへ確定し、Mesh3Dのtexture alpha不可・Imageの3D不在・ゼロコピーimport・coplanar draw orderを実測した | **決定/観察**(2026-08-17) |
| [2026-08-17-vsm-a4i-external-author-path-measurement.md](2026-08-17-vsm-a4i-external-author-path-measurement.md) | 外部作者経路がLayerSource専用であることの実測。汎用化にはregistry列挙口とgoldenの作者opt-inが要る | **観察**(2026-08-17) |
| [2026-08-17-vism-param-list-type-decision.md](2026-08-17-vism-param-list-type-decision.md) | parameterに同種の並びを足す決定。keyframeはlist全体で1キー、補間は要素ごと。未実装 | **決定**(2026-08-17) |
| [2026-08-17-vsm-b0-identity-fixture.md](2026-08-17-vsm-b0-identity-fixture.md) | VSM-B0 identity期待値マトリクス。6ケース×6操作×5identityの180セルを台帳根拠付きで固定し、根拠が台帳に無い30セルをUNDETERMINEDとして必要な決定先ごと名指しした意味fixture。ケース5・6の採否は決めない |
| [2026-08-17-vism-identity-known-implementation-survey.md](2026-08-17-vism-identity-known-implementation-survey.md) | VSM-B0の未決6問に対しOFX／CLAP／cargoの既知解を対照。5問は先行実装が答えており、U2のみ対応物なし | **比較中**(2026-08-17) |
| [2026-08-18-rerun-e0-composition-probe.md](2026-08-18-rerun-e0-composition-probe.md) | Rerunを空間合成の基盤にできるかのE0 3点実測。窓なし描画と遮蔽は成立、カメラ注入は不成立でfork seam 3箇所を名指し | **観察**(2026-08-18) |
| [2026-08-18-rerun-fork-seam-ledger.md](2026-08-18-rerun-fork-seam-ledger.md) | Rerun fork(`oshikaidesu/rerun`)が上流とどこで乖離しているかの台帳と、カメラ注入seamの実装記録。blueprint系seam(S1/S2/S3)は採らず`SpatialStage::set_camera`という公開APIで通した理由、上流rebase時の再適用手順、rev bumpの恒久oracle(`cargo test -p rerun-e0-composition-probe`)。実測=注入したdocument cameraが既知レイヤーを期待座標へ写す(2304点wrong 0)。 | 観察 |
