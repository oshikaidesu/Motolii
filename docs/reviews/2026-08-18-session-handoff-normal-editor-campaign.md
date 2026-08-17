# セッション引き継ぎ — 「普通の動画編集ソフト」campaign 第1区切り

日付: 2026-08-18
状態: **引き継ぎ**(観察。決定を含まない)

## セッションID

- 前半(クラッシュ前): `3632460e-2dad-454a-8eec-5e7ef46b8e9a`
- 後半(クラッシュ復帰後・本区切りまで): `adeabfb2-adca-4488-a381-0cab097b3d01`
- 途中でホストPCが負荷により強制終了し、プロセスとtranscriptが失われた。worktreeと
  commitは無傷で、agent worktree内の未commit部分成果から復旧した。

## 現在地(機械的事実)

- 統合branch: `claude/motolii-progress-check-5d971f`
  (worktree `.claude/worktrees/neoaviutl-motoliis-comparison-4f4b1c`)
- tip: `4b3f837e`(スタート画面)。**remoteへは一切pushしていない。全成果はlocalのみ**
- 最終ゲート: `cargo test --workspace --locked --no-fail-fast` **exit 0・208 test target・失敗0**
  (この形式にした理由: cargoは既定で最初に落ちたtest binaryで止まり、後続の防護柵に
  到達しない。実際に隠れ債務2件がこの切替で発見された)
- agent worktree群(`agent-*`)は検収済みで残置。削除していない

## この区切りで入ったもの(merge順)

1. `SetSoundtrack` Command family(`ea5fc2ef`)
2. import経路: probe→`AssetDraft`→既存admission(`ffee0d34`)
3. shell背骨: `--project`+`ProjectSeat`(`c65db3d7`)
4. CLI薄皮+headless E2E: new/import/place/set-soundtrack→export(`51b81539`)
5. labエディタのshell結線=`timeline_editor`(`0ca6c29c`)
6. 音同期再生 `audio_seat`(`523369f1`)
7. Browser/Inspector native化(HTML正本から値写し)(`19364cc6`)
8. M-keyロケータ(`4cacb300`)
9. OSドロップ+Cmd+N/Cmd+O(`b00bc217`)
10. Cmd+S+dirty表示+未保存guard+Undo/Redoボタン(`a73f40f1`)
11. GUI Export(別thread+cancel)(`de5bd108`)
12. soundtrack波形帯(`b8a411ea`)
13. Composition解像度+contain fit(縦動画letterbox)(`c01d1105`)
14. Stageへ合成フレーム表示(`671043e1`)
15. Inspector選択配線+Position/Opacity直打ち+◇キー(worktree merge)
16. 視覚パス: カメラ向き/枠のplayhead追従/重なり(`7a07a35c`)
17. 音声ドロップ=soundtrack既定(`b5cf7b72`)/パネル磨き(`3b3eb2ae`)
18. fix-forward群: `locators`欠落、`SetSoundtrack`/`SetCompositionResolution`のUI側
    exhaustive match、公開境界runner、oracle保護領域移設、GPU台帳分類、m4a demux
    (`isomp4`)、Cmd+N解像度既定、スタート画面

## 検証の実態(ここがバイアス抜きの核)

**機械検証済み:**
- 全裁定・全レーンは落ちるテスト先行(red出力保存)→green→workspace gateで統合
- 完成条件の鎖はE2E 2本で通る: 16:9(両stream ffprobe審判)と9:16(1920x1080
  letterboxのピクセル審判)
- Stage表示はheadlessピクセルoracle(playhead t→色)と再生実測(60要求中drop3・
  絵は消えない)
- Inspector編集はUndo oracle(drag畳み=1 gesture)
- 実audio deviceのsession開始/reseekはdevice-gatedテスト1本(**無音PCM**)

**人間未検証(次セッション/利用者チェックで初めて分かる):**
- 実際に窓を操作しての一連の流れ(dialog実物・実ドロップ・耳での音同期・手触り
  全般)。機械検証はdialog抜きの関数境界とscreenshot静止画まで
- スタート画面→New→ドロップ→再生→Exportの通し実走は静止screenshotの合成でしか
  確認していない
- 波形・Stage表示の実時間の滑らかさ(数値はheadless実測のみ)

## 既知の欠陥・残タスク(修正順は未決)

- テンプレ由来project(`create_timeline_lab_project`)への`import`がplugin契約エラー
  で落ちる(空projectは通る)
- Rerun fork seam: `SpatialStage`が`AppendToStore`を落とすため、カメラの残り約25°・
  orbit/zoom持続・カメラリセットが全て塞がっている(fork rev `501a0403`への1 seam)
- soundtrack差し替え/削除UIとoffset/gain UI無し(2本目の音声はclipになる)
- clip音声のmix再生無し(soundtrackのみ)。clip波形無し
- Effect追加/削除UI・Anchor/Scale/RotationのInspector行・Custom面・auto-key無し
- Browserはfolder閲覧のみ(project assetが出ない)。COLLECTIONSは空。SVG thumbnail
  不可(image crateがSVG非対応)
- autosave/journal常時追記は未結線(`save_with_journal`実在、差し替え1箇所と報告あり)
- Export進捗はframe割合でなく経過秒。設定詳細UI無し
- テスト分離の弱さ: temp projectのnanos命名がprocess間で衝突しうる(実flake 1回)。
  実device audioテストも並列実行で稀にflake
- `GpuCtx::from_device_queue`がeframeのdevice-lost handlerを置き換える(所有権未集約)
- eguiのM-keyがtext入力focus中でも発火しうる既存挙動(現状textフィールドは限定的)
- 台帳(implementation-ledger)の「現在の並列レーン」表・Timeline座席関連の行は
  今回の着地を全て反映しきれていない(N-node表とP12-C1は更新済み)

## 運転規約(playbookメモリと同内容の要点)

- レーン発注capsule: OUTCOME/CURRENT STATE(file:line照合済)/EXACT TARGET/ALLOWLIST/
  NON-GOALS/RETURN+落ちるテスト先行。agentは`isolation: worktree`、開始時にtipへff
- **同時cargoは2本まで・全cargoに`-j 4〜6`**(3本並列でホストが落ちた)
- agentのテストは前景実行(背景待ちでturnを終えると停止する)
- FableのUI系capsuleで安全機構の誤検知2回→UIレーンはOpusで発注
- 検収: diff読み→自分で再実行→`merge --no-ff`→workspace gate(no-fail-fast)
- 防護柵に当たったら意図を読み意図どおりに直す(改名逃げしない)
- 裁定はdocs/reviews→README index→decision-indexを同一commitで

## 状態の正本

- 本引き継ぎ+[campaign運転playbook(メモリ)]+`docs/ux-check-first-ten-minutes.md`
  (利用者へ渡した台本)+`docs/decision-index.md`の2026-08-17/18行+git log
- TaskListはプロセス再起動で消える(今回2回消えた)。**残作業の正本は本文書の
  「既知の欠陥・残タスク」節とする**
- 利用者の役割分担(UX合否のみ・他は推奨で自走)と「明言した定義を合否基準に
  昇格させない」制約はメモリ`full-delegation-normal-video-editor`が正本
