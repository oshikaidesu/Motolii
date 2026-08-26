# Motolii Export panel — Icebook design drafts

Icebookで比較できるように、各案を独立したパネル設計カードとして記述する。30案は
単なる色違いではなく、Exportで止まる異なる問題を主役にする。

## 共通の設計契約

- PreviewとExportは別の画を作らない。`Composition`を共通入力にし、Export側でも同じ
  `Engine::render_frame`の結果を証拠として扱う。Export専用の簡易プレビューは作らない。
- 現在の実出力は **MP4 / H.264** 1種。品質は Normal / Lossless(qp0) の2択で、存在しない
  codecやOutput Moduleの選択肢は置かない。
- アスペクトは 16:9 / 9:16 / 1:1。プリセット操作は`Intent::SetComposition`への要求で、
  パネルが別の寸法を保持しない。fps・尺・背景は同じ`Composition`から表示する。
- 範囲は全体、またはTimelineの作業範囲 `[start, end)`。作業範囲が無い時に触れる顔の
  トグルを置かず、実際に書くフレーム数を表示する。
- 音声はExport内で`AudioProgram`をmixし、必要ならFFmpeg側でmuxする。音声があるのに
  stream copyへ黙って縮退しない。成功前の最終pathを汚さず、取消・失敗時は部分fileを残さない。
- 既存のIced文法(`section_header`、`button_style`、`toggler`、既存token)を基底にする。
  「再利用」は既存の意味・入口・評価経路を指し、「scratch」は不足した表示投影だけを指す。

## E01 — Export Hero Card

- **ID:** `E01`
- **Name:** Export Hero Card
- **Problem solved:** 書き出し直前に「何を人へ渡すのか」が見えず、設定を確認するだけで制作の勢いが切れる。
- **Hero/creation role:** 中央の大きな結果カードを、作品を完成物へ変える最後の一押しにする。静止したStageの証拠と、納品仕様を同じ視線に置く。
- **Layout/visual hierarchy:** 上段に小さな`EXPORT`見出し、中央にアスペクト比を反映したStage snapshot枠、右下に`MP4 / H.264`・寸法・fps・フレーム数を縦積み、最下段に幅広いExport CTA。失敗時はsnapshot枠を壊さず、原因カードだけを上に差し替える。
- **Interaction/entry:** `Cmd+E`またはFile > Exportで開く。中央カード上の16:9 / 9:16 / 1:1を選ぶとComposition要求を出し、Destination、Lossless、Work area onlyを確認してExportする。実行中は同じカードに進捗とClose glyphのCancelを表示する。
- **Density/scale:** 中密度、幅480–620pxの浮かし窓。結果カード6割、仕様3割、操作1割。常時表示は3つの判断だけに絞る。
- **Reuse-vs-scratch note:** 既存の`view`、`Composition`、aspect preset、progress、Cancel、既存chromeを再利用。scratchは同じCompositionから取るsnapshotの表示投影と、成功・失敗カードの状態差分だけ。別renderer、別export job、共有機能は作らない。

## E02 — Three-beat Story

- **ID:** `E02`
- **Name:** Three-beat Story
- **Problem solved:** format、範囲、音声、保存先が一列に並ぶと、初心者がどの順番で決めればよいか分からない。
- **Hero/creation role:** `Frame → Sound → Deliver`の3拍で、制作物が「画を決める」「音を守る」「渡せるfileにする」へ進む感覚を作る。
- **Layout/visual hierarchy:** 横方向の3つの大きなステップ。Frameにはアスペクトカードと寸法、Soundには音声muxの状態、Deliverには保存先と品質。現在ステップをアクセント色、完了済みを静かなcheck、未到達を薄い面で表す。
- **Interaction/entry:** 各ステップはクリックで開閉するが、操作は既存のAspectPresetSelect、Work area only、Lossless、PickOutputPathに写す。3つの必須値が揃うまで最終Exportをdisabledにし、足りない値の理由を直下へ出す。
- **Density/scale:** 横長の中密度、窓幅600–760px。1ステップは2行以内に収め、狭い窓では3段縦積みに折り返す。
- **Reuse-vs-scratch note:** 既存messageとread-only Composition表示を再利用。scratchはステップ状態のprojectionと、audio mux結果を「Sound」へ翻訳する小さなread modelだけ。Audio編集欄やpreset保存機構は追加しない。

## E03 — Aspect Atlas

- **ID:** `E03`
- **Name:** Aspect Atlas
- **Problem solved:** 9:16や1:1にすると、完成画面がどう変わるかを数字だけでは想像しにくい。
- **Hero/creation role:** 3つの比率を作品の掲示面として見せ、heroの見せ方を横長・縦長・正方形から選べるようにする。
- **Layout/visual hierarchy:** 中央に3枚の比率カード。各カードは比率どおりの空枠、`1920 × 1080` / `1080 × 1920` / `1080 × 1080`、選択状態を持つ。下段に「fps・尺・背景は保持」、右側にRangeとExportを置く。
- **Interaction/entry:** カードを押すと既存のAspectPresetSelectを発火し、選択後に現在寸法とフレーム数を更新する。Custom比率は`Custom`ラベルで表示し、勝手にプリセットへ丸めない。
- **Density/scale:** 視覚優先の低密度、幅560–700px。カードは同時に3枚見え、比率の輪郭が文字より先に読める大きさにする。
- **Reuse-vs-scratch note:** `AspectPreset::ALL`、`dimensions_for_aspect`、`format_resolution`を再利用。scratchは比率シルエットのIced layoutだけ。crop・自動reframe・新しいComposition ownerは作らない。

## E04 — Format Truth Bar

- **ID:** `E04`
- **Name:** Format Truth Bar
- **Problem solved:** 普通の動画ソフトのようにcodecやOutput Moduleが選べるように見えると、選べない機能への期待と失望が生まれる。
- **Hero/creation role:** 「この作品は今すぐMP4として出せる」という確かな出口を一本の真実バーで示し、選択肢の多さではなく完成を主役にする。
- **Layout/visual hierarchy:** 最上段に太い一行`MP4 / H.264`、隣に小さな`READY`。その下にLossless(qp0)だけをtogglerで置き、Aspect・Range・Destinationを2列の小さな情報群にする。未知のcodec欄は存在させない。
- **Interaction/entry:** Formatバーはread-only。Losslessは即時切替、Aspectは3つの要求ボタン、DestinationはOS picker、RangeはWork area only。Exportを押した時の最終summaryをCTA直上に固定する。
- **Density/scale:** 高密度、幅420–520px。情報行中心で、最上段のFormatだけ行高を大きくする。設定アプリのセクション乱立にはしない。
- **Reuse-vs-scratch note:** `CONTAINER_CODEC_LABEL`、品質enum、既存togglerとinfo rowを再利用。scratchはFormatを主役にする階層CSS/Iced構成のみ。未実装codec、queue、templateは追加しない。

## E05 — Preset Strip

- **ID:** `E05`
- **Name:** Preset Strip
- **Problem solved:** 毎回同じアスペクトと品質を探すため、heroを作る反復が設定操作に埋もれる。
- **Hero/creation role:** 画面上部の一筆書きのようなpreset stripで、作品を「横・縦・正方形」の納品面へ即座に着地させる。
- **Layout/visual hierarchy:** ヘッダー直下に横一列の`16:9 / 9:16 / 1:1` stripを置き、各chipに寸法を常時表示。現在のCompositionはselected chip、Customは別の薄いchip。下に範囲・出力先・品質、右下にCTA。
- **Interaction/entry:** stripのchipはAspectPresetSelect。chip選択後に寸法とFramesを更新し、Work area onlyが有効なら範囲表示も更新する。Export中はstripとtogglerをdisabledにし、snapshotを混ぜない。
- **Density/scale:** 低〜中密度、幅500px前後。preset strip 1行、summary 3行、action 1行の固定高さ。キーボードでも3候補を左右移動できる想定。
- **Reuse-vs-scratch note:** 既存の3 preset button、Composition read model、running時disabled規律を再利用。scratchはchipを横一列に圧縮するレイアウトのみ。プリセットの永続化、複数書き出しqueueは作らない。

## E06 — Frame Ledger

- **ID:** `E06`
- **Name:** Frame Ledger
- **Problem solved:** 全体を書いたのかWork areaだけを書いたのか、Outが含まれるのかが分からず、不要な尺を納品してしまう。
- **Hero/creation role:** Exportを「曖昧な時間設定」ではなく、実際に書くフレームの台帳として見せる。完成物の長さを自分で握れる。
- **Layout/visual hierarchy:** 左に細い時間線、全体帯とWork area帯を重ね、右に`start – last (frames)`を大きく表示。半開区間の`[start, end)`はcaptionで説明し、下段にfpsと尺を置く。
- **Interaction/entry:** Work area only togglerを時間線上の選択に同期。作業範囲がない時は帯とトグルを出さず「全体」と明記。範囲外ならclamp後の実フレーム数を先に表示してからExportを許可する。
- **Density/scale:** 情報密度は中、幅520–640px。時間線は高さ56px、数値を視覚の主役にする。長い説明文は置かない。
- **Reuse-vs-scratch note:** `WorkAreaFrames`、`effective_range`、`format_range_summary`、existing togglerを再利用。scratchは範囲帯の描画投影だけ。TimelineのWorkAreaをExport側で所有・編集しない。

## E07 — Range Theater

- **ID:** `E07`
- **Name:** Range Theater
- **Problem solved:** 「全体」と「作業範囲」の差が小さなtogglerだけだと、短いheroテストを書きたい時に安全に選べない。
- **Hero/creation role:** 本番全体と試作範囲を2つの舞台として比較し、試す→決める→渡すの流れを支える。
- **Layout/visual hierarchy:** 左右2枚のrange card。左は`FULL COMPOSITION`と全フレーム、右は`WORK AREA`とIn/Out/枚数。選択カードだけにアクセントを置き、最下段に「この範囲をExport」を出す。
- **Interaction/entry:** カードクリックがRangeSelectへ写る。Work areaが無い時は右カードをdisabledではなく撤去し、全体カードだけを出す。品質とAspectはカード下の共通summaryで変更する。
- **Density/scale:** 低密度、幅560–700px。2カードの比較を最優先し、出力先は下段に一行だけ置く。
- **Reuse-vs-scratch note:** 既存のWhole/WorkArea、`effective_range`、最短1フレーム防波堤を再利用。scratchは2択カードの状態投影だけ。新しいIn/Out編集操作や別時間正本は作らない。

## E08 — Destination Dock

- **ID:** `E08`
- **Name:** Destination Dock
- **Problem solved:** 書き出し設定は揃っているのに、どこへfileが出るのか見失い、完成したか確認できない。
- **Hero/creation role:** 保存先を「納品場所」として最上位に置き、作品を外へ持ち出す最後の動線を迷わせない。
- **Layout/visual hierarchy:** 上半分をDestination cardにしてpathを大きく表示、未設定時は理由付きの`Choose…`を中央配置。下半分にFormat・Aspect・Frames・Audio muxの小さなreceipt、最下段にExport。
- **Interaction/entry:** Choose…は既存のrfd pickerへ。path選択後にsummaryとCTAを更新する。Export成功後は同じDestination cardを`DELIVERED`状態へ変え、statusに実pathとframes_writtenを出す。
- **Density/scale:** 中密度、幅500–640px。pathが長い時は中央省略し、hover/tooltipで全文を出す想定。設定項目を増やさない。
- **Reuse-vs-scratch note:** `PickOutputPath`、`OutputPathChosen`、`ExportOutcome`、status帯を再利用。scratchはpathを主役にするカードと完了状態の投影だけ。Finder起動や共有シートはMotoliiへ持ち込まない。

## E09 — Preview=Export Witness

- **ID:** `E09`
- **Name:** Preview=Export Witness
- **Problem solved:** Stageで見たheroが書き出し後に変わる不安が、Exportボタンを押す直前に最大化する。
- **Hero/creation role:** 「見た画がそのまま外へ出る」という制作上の約束を、主張ではなく同じComposition・同じ評価経路の証人として見せる。
- **Layout/visual hierarchy:** 左に現在のStage snapshot、右に`SAME COMPOSITION`の仕様カード。中央を細い同一性ラベルで結び、下段にAspect変更時の寸法、Frames、Audio mux、CTAを置く。snapshotが無い時は空箱ではなく「Stageの結果を待つ」と書く。
- **Interaction/entry:** Exportを開いた時点のcurrent frameを表示する想定。Aspect presetを選ぶと同じCompositionのsnapshotと寸法が更新される。別のrender qualityやpreview-only cropの操作は置かない。
- **Density/scale:** 横長・低密度、幅720px前後。snapshot 55%、証拠カード45%。狭い窓では証拠カードを下へ送る。
- **Reuse-vs-scratch note:** `Composition`共有、`Engine::render_frame`、既存aspect/range/qualityを再利用。scratchはsnapshotのread-only投影と`same input`バッジだけ。Export専用renderer、second compositorは作らない。

## E10 — Composition Receipt

- **ID:** `E10`
- **Name:** Composition Receipt
- **Problem solved:** Export窓で解像度やfpsを編集できるように見えると、PreviewとExportの入力が分岐する。
- **Hero/creation role:** 作品の最終仕様を改変する場所ではなく、現在のCompositionを確認して安全に渡す場所だと明確にする。
- **Layout/visual hierarchy:** 上段に`CURRENT COMPOSITION`のreceipt。`Size / Aspect / FPS / Duration / Frames`を読み取り専用の5行で表示し、横に16:9・9:16・1:1の要求ボタンだけを置く。cap超過はreceiptの直下にwarning帯。
- **Interaction/entry:** 寸法・fps・尺の行は押せない。presetだけがSetComposition要求を出し、更新後にreceipt全体を再計算。cap超過時はExportを実行不可にし、理由と修正入口を同じ面に出す。
- **Density/scale:** 高密度、幅420–540px。表形式だが線を引かず、既存の行高と余白で読む。数値の視認性を最優先。
- **Reuse-vs-scratch note:** `Composition` read-only投影、`resolution_within_cap`、aspect mapping、existing info rowを再利用。scratchはreceiptの語順とwarningの配置だけ。Export専用のwidth/height/fps ownerは作らない。

## E11 — Audio Mux Check

- **ID:** `E11`
- **Name:** Audio Mux Check
- **Problem solved:** mp4は完成したのに、音声が入っているか分からず、無音のheroを人へ渡してしまう。
- **Hero/creation role:** 作品の映像と音を一本の納品物として扱い、音声muxを完成条件の一部として視覚化する。
- **Layout/visual hierarchy:** 中央に`VIDEO → MIX → MP4`の3段pipeline。Videoは書いたframes、MixはAudioProgramの状態、MP4は最終mux状態を表示。成功時は最終ノードを強調し、未確認・失敗は音声ノードだけを赤くする。
- **Interaction/entry:** Export開始前はAudioの有無をread-only確認。音声がある場合はmux込みの最終jobへ進み、Export後にAudio track presenceをstatusへ返す。音声が無い場合は「無音を意図したExport」と明示して続行できる。
- **Density/scale:** 中密度、幅560–680px。pipelineは高さ120px、下にFormat・Range・Destinationを3行で置く。
- **Reuse-vs-scratch note:** `AudioProgram::mix_audio`、`mux_mixed_pcm`、`ExportOutcome::audio_muxed`、既存Export flowを再利用。scratchはmix/mux段階のstatus projectionだけ。音声mixerやFFmpeg呼び出しをUIで再実装しない。

## E12 — Sound Trail

- **ID:** `E12`
- **Name:** Sound Trail
- **Problem solved:** 音声がどの時間範囲に存在し、書き出し範囲と重なっているかを、音量編集なしでは判断しづらい。
- **Hero/creation role:** MVのheroに音が付いていることを、波形の飾りではなく「この範囲がmixへ入る」証拠として見せる。
- **Layout/visual hierarchy:** Range timelineの下に細いsound trailを一本だけ置く。soundtrack・clip audioの存在を色面で示し、右に`mixed PCM / silent`の状態。音量ノブやEQは置かず、映像のpreviewを主役に保つ。
- **Interaction/entry:** Work area onlyを切り替えるとsound trailの範囲を更新。trailはread-onlyで、クリックは音声設定画面へ飛ばさずExportの確認に留める。音声が無い場合は線を消し、無音理由をcaptionで示す。
- **Density/scale:** 横長・中密度、幅600px、trail高さ32px。時間情報はE06のrange表示と同じ尺度を共有する。
- **Reuse-vs-scratch note:** TimelineのWorkArea、AudioProgramの入力判定、既存tokenを再利用。scratchは音声存在区間のread-only可視化だけ。Export窓にwaveform編集、gain、fadeのownerを作らない。

## E13 — Audio Sources Ledger

- **ID:** `E13`
- **Name:** Audio Sources Ledger
- **Problem solved:** どの音がmuxに入ったのか、複数素材・soundtrack・audio-only Clipの関係が分からない。
- **Hero/creation role:** 「この作品の音は何でできているか」を納品前の小さなクレジットとして見せ、音の抜けを防ぐ。
- **Layout/visual hierarchy:** 左にVideo output、右に音声sourceの縦ledger。各行は`source name / range / enabled / mix`の短いreceiptで、最後に`AudioProgram → AAC track`を大きく表示。sourceがない時は空欄を増やさず`No audio source`を1行にする。
- **Interaction/entry:** 行は選択・編集せず、既存Document/Sessionのprojectionとして表示。音声を追加・分離する入口はExport内に作らず、問題がある時は「Timelineで直して再確認」とだけ示す。
- **Density/scale:** 中〜高密度、幅520–640px。sourceは最大4行を想定し、それ以上は件数と要約に畳む。納品CTAは常時最下段。
- **Reuse-vs-scratch note:** `AudioProgram`の入力列挙、Clipの既存source、mix順序、AAC muxを再利用。scratchはsource receiptのread modelだけ。新しいaudio track schemaや隠れlinkは作らない。

## E14 — Silent Export Guard

- **ID:** `E14`
- **Name:** Silent Export Guard
- **Problem solved:** 音声が無いのか、検出に失敗したのか、意図的な無音なのかを区別できず、書き出し後に気づく。
- **Hero/creation role:** 無音を禁止するのではなく、無音を選んだのか見落としたのかを制作判断として確定させる。
- **Layout/visual hierarchy:** Export CTA直上に大きなSound guardを置く。`AUDIO INCLUDED`、`INTENTIONAL SILENCE`、`AUDIO CHECK FAILED`の3状態を同じ場所で表し、失敗時だけ理由と戻り先を展開する。映像仕様は隣の静かなreceipt。
- **Interaction/entry:** 音声ありなら確認済みで進行、音声なしなら`無音で続行`と`Timelineへ戻る`の二択、判定不能ならExportを止めてretry/close。muxエラーは再実行前に最終pathを変更しない。
- **Density/scale:** 低密度、幅500–620px。警告は一枚だけ。一般的な設定項目を増やさず、音声の安全判断に専念する。
- **Reuse-vs-scratch note:** 既存のAudioProgram/mux error/status/close経路を再利用。scratchは3状態のread-only判定とCTAの表示だけ。音声を勝手にmute/unmuteする編集機構は作らない。

## E15 — Mix Mode Badge

- **ID:** `E15`
- **Name:** Mix Mode Badge
- **Problem solved:** Exportが元streamをそのまま使うのか、加工済み音をmixしているのかが隠れ、結果の音を予測できない。
- **Hero/creation role:** 音声経路の選択をユーザーの裏で勝手に行わず、heroの再現性を守るための技術的事実として表示する。
- **Layout/visual hierarchy:** Format行の横に`STREAM COPY`または`MIXED PCM → MUX`のbadgeを表示。badgeの下に「gain / 複数source / retimeがあるため」など、現在の判定理由を一行で出す。CTAはbadgeより下で、選択肢に見せない。
- **Interaction/entry:** badgeはread-only。source条件やrangeを変えると判定が更新され、mixed path時はAAC trackの見込みを表示。ユーザーが危険なfast pathを選べるトグルは置かない。
- **Density/scale:** 高密度、幅420–540px。badge 1行＋理由1行。音声編集面ではなく、Exportの技術receiptとして扱う。
- **Reuse-vs-scratch note:** AudioProgramの判定、`mix_audio`、`mux_mixed_pcm`、codec表示を再利用。scratchは判定理由の短い文面とbadgeのvisual styleだけ。別の音声engineやcodec選択を作らない。

## E16 — Audio Safety Ladder

- **ID:** `E16`
- **Name:** Audio Safety Ladder
- **Problem solved:** 音声付きExportが失敗した時、decode・mix・WAV下書き・muxのどこで止まったかが分からない。
- **Hero/creation role:** 音を守る工程を階段として見せ、失敗しても作品の意味を失わずに直せるようにする。
- **Layout/visual hierarchy:** 縦4段のladder `READ → MIX → STAGE → MUX`。各段にpass・pending・errorを置き、成功済みの段は圧縮、エラー段だけ詳細を展開。右にvideo framesの進捗を細く併置する。
- **Interaction/entry:** 実行中は現在段とframesを更新。Error段の`Retry`は同じ設定でやり直し、`Choose destination`だけpath pickerへ戻る。中断はladderを`CANCELLED / FINAL PATH UNTOUCHED`へ遷移させる。
- **Density/scale:** 中密度、幅560–680px、高さ300px程度。実行中だけ表示し、idle時は短いAudio checkへ畳む。
- **Reuse-vs-scratch note:** `export_ops`のvideo export、mix、temporary WAV、mux、cleanupの実経路を再利用。scratchは段階statusを伝えるread modelだけ。非同期処理やcleanup処理をUIで二重化しない。

## E17 — MV Beat Card

- **ID:** `E17`
- **Name:** MV Beat Card
- **Problem solved:** モチベーション動画では、映像だけ正しく出ても音の開始位置・範囲・納品感が揃わないとheroにならない。
- **Hero/creation role:** soundtrackと映像の同じ時間範囲を一枚のbeat cardにまとめ、作品の「一周」を書き出す面にする。
- **Layout/visual hierarchy:** 上段にhero snapshot、下段に左右の`PICTURE`と`SOUNDTRACK`帯を同じIn/Outスケールで表示。中央の大きなラベルを`SYNCED EXPORT`にし、右下にframesとmux済み見込みを置く。
- **Interaction/entry:** Work area onlyを切り替えると映像とsoundtrackの帯を同時に縮める。Aspect presetはsnapshotの枠とComposition receiptを更新。音の中身を編集する入口はTimelineへ戻すリンクではなく、確認文だけにする。
- **Density/scale:** 低密度・横長、幅700px前後。hero snapshot 45%、同期帯35%、納品情報20%。
- **Reuse-vs-scratch note:** `WorkAreaFrames`、Composition、AudioProgram、same render/mux経路を再利用。scratchはpicture/soundの同期帯とMV向け言語の投影だけ。BPM、音声エフェクト、別Transportは作らない。

## E18 — Delivery Passport

- **ID:** `E18`
- **Name:** Delivery Passport
- **Problem solved:** Export完了の表示が短いstatus文だけだと、成果物の内容を再確認できず、完成した実感が弱い。
- **Hero/creation role:** 完成したfileを「納品パスポート」として記録し、作品をMotoliiの外へ持ち出す最後の証明にする。
- **Layout/visual hierarchy:** 完了後だけ中央に大きなcheckと`DELIVERED`。その下に`path / MP4 H.264 / aspect・size / fps・frames / audio track`をreceiptとして並べ、最下段に`Export again`。新しいfile共有ボタンは置かない。
- **Interaction/entry:** status帯や完了イベントから開く。`Export again`は同じ設定を保持して再実行、Aspect・Range・Destinationの変更は通常状態へ戻してから行う。Finderへの責任はOSへ渡す。
- **Density/scale:** 中密度、幅520–640px。完了状態では情報5行を常時表示し、長いpathだけ折りたたむ。
- **Reuse-vs-scratch note:** `ExportReport`、`out_path`、`frames_written`、`audio_muxed`、既存statusを再利用。scratchはpassportの成功projectionだけ。share/upload・履歴DB・queueは作らない。

## E19 — Progress Timeline

- **ID:** `E19`
- **Name:** Progress Timeline
- **Problem solved:** 進捗が`41%`という数字だけだと、処理が止まったのか、どれだけ残っているのかを直感的に判断できない。
- **Hero/creation role:** 書き出し中も作品が完成へ進んでいることを、frameという制作単位で見せる。
- **Layout/visual hierarchy:** パネル中央に横長のframe timeline。左端をstart、現在位置をアクセントのplayhead、右端をlast frameにする。下に`123 / 300 (41%)`、上に`MP4 / H.264`と小さなCancelを置く。
- **Interaction/entry:** Exportを押すとidle controlsをprogress stateへ置き換える。進捗は`ExportProgress`のframes_done/totalだけから更新し、ユーザー操作はCancelだけに制限する。
- **Density/scale:** 中密度、幅520–680px、高さ180px。数字とplayheadを同じ視野に置き、ETAなど未計測値は出さない。
- **Reuse-vs-scratch note:** `ExportProgress`、`progress_fraction`、`format_progress`、background Task、Cancelを再利用。scratchはprogress timelineの描画だけ。推定時間や別のworker制御は作らない。

## E20 — Live Frame Proof

- **ID:** `E20`
- **Name:** Live Frame Proof
- **Problem solved:** Export中にStageが止まって見えると、UIが固まったのか、書き出しが進んでいるのか分からない。
- **Hero/creation role:** 最後に処理したframeを小さなproofとして見せ、heroが完成物へ変換され続けている感覚を保つ。
- **Layout/visual hierarchy:** 左に直近frameのthumbnail、右に大きなframe countとprogress、最下段に「同じComposition / same render path」ラベルとCancel。thumbnailは主役ではなく進行証拠にする。
- **Interaction/entry:** background progress更新時に直近frameを差し替える。クリックで別previewを開かず、Cancelは即時にcancel flagを立てる。終了時はthumbnailをDelivery Passportのsnapshotへ引き継ぐ。
- **Density/scale:** 横長・中密度、幅640–760px。thumbnail 35%、進捗45%、操作20%。
- **Reuse-vs-scratch note:** `Engine::render_frame`の既存結果、`ExportProgress`、既存Cancelを再利用。scratchは直近frameのread-only cache投影だけ。Export専用レンダーや全frame蓄積は作らない。

## E21 — Cancel With Safety

- **ID:** `E21`
- **Name:** Cancel With Safety
- **Problem solved:** 書き出しを止めたいのに、途中fileが最終成果物として残る恐怖でCancelできない。
- **Hero/creation role:** 試行を安全に捨てられることを示し、hero作成の反復回数を増やす。Cancelは失敗ではなく制作の戻り道にする。
- **Layout/visual hierarchy:** progressの横に大きなCancel action、直下に`最終pathは変更しない / partial outputは削除`を固定表示。Cancel後は中央に`CANCELLED — FINAL PATH UNTOUCHED`を表示し、RetryとCloseを置く。
- **Interaction/entry:** 実行中のCancelは既存Cancel handleへ。確認dialogで操作を二重化せず、押した瞬間にflagを立て、frame境界後に安全状態へ遷移する。Retryは同じ範囲・品質・Destinationを保持する。
- **Density/scale:** 中密度、幅500–620px。安全文を常時読める高さにし、進捗バーを過剰に大きくしない。
- **Reuse-vs-scratch note:** `Cancel::cancel`、`remove_partial`、temporary video/audio cleanup、既存progress stateを再利用。scratchはcancelled receiptだけ。最終pathの削除・バックアップをUI側で実装しない。

## E22 — Error Compass

- **ID:** `E22`
- **Name:** Error Compass
- **Problem solved:** 「Exportに失敗しました」だけでは、寸法超過・保存先・encode・音声muxのどこを直せばよいか分からない。
- **Hero/creation role:** エラーを制作の行き止まりではなく、次の一手を示す方位磁針にする。作品のsnapshotと設定を保持したまま直す。
- **Layout/visual hierarchy:** 上に`EXPORT FAILED`と短い原因、中央に4分類のcompass `COMPOSITION / DESTINATION / VIDEO / AUDIO`。該当分類だけアクセント、下に一つのFix CTAとRetry。partial cleanupの結果を最後に表示する。
- **Interaction/entry:** cap超過はAspect/Compositionへ、destination失敗はChoose…へ、audio mux失敗は同じ設定のRetryまたはTimeline確認へ、video errorはRetry/Closeへ分岐。原因不明を成功扱いにしない。
- **Density/scale:** 中密度、幅560–680px。原因は1〜2行、分類4枚、行動1つ。stack traceや技術ログを主面に出さない。
- **Reuse-vs-scratch note:** `ExportError`、`resolution_within_cap`、status、cleanup、既存messageを再利用。scratchはエラー分類とFix destinationのprojectionだけ。自動修復、別codec fallback、silent downgradeは作らない。

## E23 — Recovery Postcard

- **ID:** `E23`
- **Name:** Recovery Postcard
- **Problem solved:** 失敗や取消の後に設定が消えると、同じheroをもう一度作るための復旧コストが高い。
- **Hero/creation role:** 作品の設定を「作り直し不要のはがき」として残し、失敗しても制作の文脈を失わない。
- **Layout/visual hierarchy:** 左に失敗・取消の状態stamp、右に保存された`Aspect / Range / Quality / Destination / Audio path`の5行。下に`Retry unchanged`、`Choose another path`、`Close`を並べる。
- **Interaction/entry:** `Retry unchanged`は同じsnapshotを再実行、path変更だけはpickerへ。Work areaやaspectを変える時は通常のidle viewへ戻す。成功時は同じpostcardをDelivery Passportへ遷移させる。
- **Density/scale:** 高密度、幅520–640px。receipt中心で、詳細エラーは展開式の1枠に閉じる。
- **Reuse-vs-scratch note:** Shellが保持するexport state、path picker、range/quality/aspect、Cancel cleanupを再利用。scratchは復旧用状態カードの投影だけ。別のdraft config storeやexport historyは作らない。

## E24 — Preflight Gate

- **ID:** `E24`
- **Name:** Preflight Gate
- **Problem solved:** Exportを押してから初めて、保存先未設定、cap超過、空範囲、音声経路の問題に気づく。
- **Hero/creation role:** 書き出し開始前に完成物の成立条件を一枚で確認し、失敗を待ち時間の後ろへ追いやらない。
- **Layout/visual hierarchy:** 中央に4つのgate `COMPOSITION / RANGE / DESTINATION / AUDIO`。各gateは一行の判定と証拠値だけ。全てpassで大きなExport、failは該当行の右にFixを置く。FormatはMP4/H.264のtruth barとして上に固定。
- **Interaction/entry:** Export CTAを押すとpreflightを一度表示し、passなら同じ位置から開始、failなら実行せず修正へ戻す。Work areaなしはRange passの`Whole`、無音はAudio passの`Intentional silence`として区別する。
- **Density/scale:** 中〜高密度、幅500–620px。各gateは1行、高さ200px以内。説明文を増やさない。
- **Reuse-vs-scratch note:** 既存Composition、`effective_range`、path有無、AudioProgram/mux判定、cap warningを再利用。scratchはgate集約のread modelだけ。preflightで別のrenderや試験用Exportを実行しない。

## E25 — Hero Launch

- **ID:** `E25`
- **Name:** Hero Launch
- **Problem solved:** Export窓が設定の羅列に見えると、Motoliiの「モチベーションとしての動画制作」が普通の動画ソフトの管理画面へ縮む。
- **Hero/creation role:** ボタンを`Make the hero`相当の一つの決断にし、作品を人へ渡す瞬間をパネルの主役にする。詳細は主役を邪魔しない。
- **Layout/visual hierarchy:** 上半分に現在のhero snapshotと大きな作品名／比率、中央に一文の`Previewで見た作品をMP4へ`、下に大きなExport CTA。Format・Range・AudioはCTA脇の小さな安全receipt、詳細は折りたたみ。
- **Interaction/entry:** Cmd+Eで直接この面を開く。CTAはdestination未設定ならChoose…へ、設定済みならpreflightを通過して開始。Advancedを開けば既存の品質、aspect、rangeを操作できるが、別の機能群は出さない。
- **Density/scale:** 低密度・大スケール、幅560–720px。hero 55%、CTA 20%、安全receipt 25%。小窓ではsnapshotを上下に圧縮する。
- **Reuse-vs-scratch note:** 既存のExport入口、Composition、aspect/range/quality、path、progress/Cancellationを再利用。scratchはhero文言・snapshot配置・advancedのvisibilityだけ。新しい制作モデルやプロジェクトmodeは作らない。

## E26 — Social Variant Lab

- **ID:** `E26`
- **Name:** Social Variant Lab
- **Problem solved:** 同じ作品を横・縦・正方形で試したい時、比率変更の結果と書き出し範囲を別々に確認する必要がある。
- **Hero/creation role:** 3つの納品面を同じ作品のvariantとして比較し、どの面がheroを最も強く見せるかを制作中に判断する。
- **Layout/visual hierarchy:** 左に3つのvariant tabs、中央に選択variantのsnapshot枠、右に`size / fps / duration / frames / audio`のreceipt。下に単一Export CTAを置き、batch queueは置かない。
- **Interaction/entry:** tab選択は既存AspectPresetSelect、snapshotとframesを更新。RangeとQualityはvariant間で共通。Exportは現在選択中の一つだけを出し、複数同時書き出しを暗黙に約束しない。
- **Density/scale:** 横長・中密度、幅680–780px。tabs 15%、snapshot 50%、receipt 25%、action 10%。
- **Reuse-vs-scratch note:** 既存3 preset、SetComposition、same Composition input、range/quality/audio muxを再利用。scratchはvariant比較のvisual projectionのみ。自動crop、reframe、render queueは作らない。

## E27 — One-file Handoff

- **ID:** `E27`
- **Name:** One-file Handoff
- **Problem solved:** 完成動画を渡す目的なのに、設定と技術情報が前面に出て、どの一つのfileを渡せばよいかが埋もれる。
- **Hero/creation role:** 「渡すfileはこれ」という一点へ視線を集め、Motoliiを出た後の人との共有を想像できる納品面にする。
- **Layout/visual hierarchy:** 中央に一本のfile card。pathのbasenameを最大文字、下に`MP4 / H.264`・比率・尺・音声trackをbadgeで添える。左上に小さなhero snapshot、下段にExportまたは完了stamp。OSの共有操作は表示しない。
- **Interaction/entry:** idleではChoose destination→Export、completedではpathとframes_writtenを固定。再実行は`Export again`、設定変更は小さな`Adjust`で通常controlsへ戻す。音声なしはbadgeを隠さず`SILENT`と明示する。
- **Density/scale:** 低密度、幅520–640px。file card 60%、技術receipt 25%、action 15%。長いpathは省略表示する。
- **Reuse-vs-scratch note:** out_path、status、ExportOutcome、既存format/aspect/range/audio情報を再利用。scratchはbasename中心の完了カードだけ。Motolii内にshare/upload APIやfile historyを作らない。

## E28 — Export State Canvas

- **ID:** `E28`
- **Name:** Export State Canvas
- **Problem solved:** idle、実行中、取消、失敗、完了でcontrolsがどう変わるかが不統一だと、触れるもの・触れないものの境界が崩れる。
- **Hero/creation role:** Exportを一つの制作ステートマシンとして見せ、どの状態でも「作品を失わず次に進める」感覚を保つ。
- **Layout/visual hierarchy:** 同じ中央canvasを保ち、状態ごとに`READY / RENDERING / CANCELLED / FAILED / DELIVERED`の大きな状態labelだけを差し替える。周囲のComposition receiptとDestinationは位置を固定し、実行中はcontrolsをdisable、完了後はpassportへ変える。
- **Interaction/entry:** Menu/shortcutでREADYへ、ExportでRENDERING、CancelでCANCELLED、errorでFAILED、成功でDELIVERED。各状態の主CTAは一つだけにし、選択中jobの設定を実行中に変更できない。
- **Density/scale:** 中密度、幅520–640px。canvasの中央状態label 35%、固定receipt 45%、CTA 20%。
- **Reuse-vs-scratch note:** `ViewModel::progress`、既存Message、background export、cleanup、statusを再利用。scratchは状態ごとのlayout投影と遷移表示だけ。新しい状態ownerや二重cancel経路は作らない。

## E29 — Quiet Focus

- **ID:** `E29`
- **Name:** Quiet Focus
- **Problem solved:** Export直前に大量の情報を見せると、heroを渡す判断より細部の確認が重くなり、制作の集中が切れる。
- **Hero/creation role:** 重要な3判断だけを静かに残す。`どの比率か / どの範囲か / どこへ渡すか`を決めたら、作品を外へ出せる。
- **Layout/visual hierarchy:** 背景を広く取った中央カードにhero snapshot、下に3つの大きなpill `Aspect`・`Range`・`Destination`。右下にExport、音声muxとFormatは小さな安全stamp。詳細はクリックで同じ窓内のdrawerにする。
- **Interaction/entry:** Cmd+Eはこのfocus面へ。pillを押すと既存のaspect buttons、Work area toggler、Choose…を展開する。Losslessとframe数はdrawer側へ置くが、実行中は全て閉じてCancelだけを残す。
- **Density/scale:** 最低密度、幅520–680px。常時表示は3pill＋CTA。drawerを開いた時だけ中密度へ戻る。
- **Reuse-vs-scratch note:** 既存controls、same Composition、progress/cancel、audio statusを再利用。scratchはdrawerの開閉とsummary pillの投影だけ。機能を隠して未実装を誤認させない。

## E30 — Creator Control Deck

- **ID:** `E30`
- **Name:** Creator Control Deck
- **Problem solved:** Export panelが「大きなheroを作る面」と「安全に書き出す面」の両方を持つが、一列の行では優先順位を表現できない。
- **Hero/creation role:** Motoliiらしい最終案。中央のheroを見せながら、左で納品面を選び、右で音声・範囲・安全を確認し、最後に一つのLaunchへ進む。
- **Layout/visual hierarchy:** 3カラム構成。左`MAKE`はAspect Atlasの縮小版とQuality、中央`HERO`はStage snapshot＋Preview=Export witness、右`DELIVER`はRange ledger・Destination・Audio mux check。下端を全幅のprogress/Cancel/Delivery result railに固定する。
- **Interaction/entry:** File > Exportで開き、左のpreset変更はComposition要求、右のRange/Destinationは既存message、中央のExport CTAはpreflightを通して実行。実行中は左・右の入力を止め、中央snapshotと下端progressだけを更新。完了時は中央をDelivery Passportへ変える。
- **Density/scale:** 横長・高密度だが階層は明確、幅900–1100px。中央50%、左右25%ずつ。狭い窓では`MAKE → HERO → DELIVER`の3段に折り返す。
- **Reuse-vs-scratch note:** 既存Export paneの意味、Composition、aspect/range/quality、path picker、AudioProgram/mux、progress/Cancellation、chrome grammarを最大限再利用。scratchは3カラムのIcebook composition、hero snapshot、preflightの接続表示だけ。queue、codec増設、別評価経路、audio editor、share/uploadは作らない。
