# Motolii 固有の決定 — 知らないと再発明する物

**この文書は「AE から離れる話」の既決事項一覧である。**

実装者が引ける審判は3つある(裁定271/272/274):

| 問いの種類 | 審判 |
|---|---|
| **AE に追いつく話** | **Lottie** — `app/reference/lottie.schema.json` と `lottie-coverage.tsv` |
| **UI の分岐** | **意図論** — 「利用者は何を求めてこの操作をするのか」。操作は意図が名指した物だけを変える |
| **AE から離れる話** | **試験** — 「利用者がやりたかったことか、道具ができなかったことの回避か」**+ この文書** |

3番目の試験は**生成規則**であって既決事項ではない。**だから試験を当てた実装者が、既に別の答えが
出ている物を独自に導出し直す。この文書がその穴を埋める。**

**実例**: 2026-08-27 に「調整レイヤーを持たない」を利用者と二人で導出し裁定275 として記録したが、
**2026-07-23 に既に決まっていた**(「Group Core に巨大 enum を作らない = precomp 相当・
調整レイヤー相当を持たない」)。1ヶ月前の決定を再導出していた。

出典は `decision-index.md` の判定語「決定」323件(2026-08-28 時点)。
**新しい決定を足したら、ここにも1行足す。**

---

## モデル(Document / 時間 / identity)

- **単一 writer 原則** — Document への mut は `DocumentWriter::edit` のみ。event-loop-local な
  private edit runtime で prepared request を `apply_macro` へ1回渡し、成功時だけ新 snapshot を配送
- **小さな Core / Controlled Microkernel** — Core は identity・canonical time・revision・
  typed capability・snapshot・atomic commit・authority 多重度・lifecycle だけ。reducer / journal /
  Undo / evaluator / Preview / Export / UI は Host capability module へ分離してよい
- **Group は precomp でも調整レイヤーでもない** — 子 identity と編集可能性を保つ
  「owned set の意味的シャーシ」。Group Core は Ownership・stable ID・lifecycle・scope 参照・
  循環診断だけ。**Composite / Cloner / Depth / Bake / Vism 固有 param を巨大 enum へ集積しない**
- **変換階層は parent が単一の真実** — `LayerAttrs.parent`(循環ガード済)が唯一の正本。
  Group は `LayerSource::Group` マーカーで members 列を持たない。**Group 所属と parent 参照は別の辺**
- **Group の尺は導出、独立値を持たせない** — 子が終われば Group も終わる。
  **ただしキーフレームはその制限の上に居る**(尺 = 素材の事実、キーフレーム時刻 = 作者の決定)
- **単一 Camera・2D = z=0** — 全 Composition に常在する単一 `CompCamera`。2D = `z=0` は Document 意味
- **出力解像度は Composition が所有** — `resolution: Option`(None = 旧導出互換)、既定 1920x1080。
  素材は aspect 保存の fit(contain)。**export の source 由来解像度導出は廃止**
- **トリムの壁は Media 素材だけ** — `source_in + duration × speed ≦ 総フレーム数`。
  Solid / Null / Shape / Text に上限は無い。**壁は硬く止まる**(隠さない)。伸ばしたいなら
  タイムストレッチ / ループという別動詞
- **fps 整合の単位** — `start`/`duration` は comp のフレーム、**`source_in` は素材のフレーム**、
  繋ぐのが `speed`。素材フレーム番号 = `source_in + (comp_frame - start) × speed`
- **`rotation.x/y/z`・多次元 scale は実装する。欄を消さない** — Lottie にある語彙なので削らない
- **Duplicate / Copy-Paste は常に Independent** — 全 ID を新規採番、複製サブツリー内参照だけ再写像。
  **live 同期しない**
- **Asset 識別は source fingerprint と recipe/artifact digest を別 identity へ分離** —
  fingerprint は `motolii-source-v1:sha256:<64桁hex>` + size。worker は生の locator でなく
  Host-private `SourceBinding` を使う
- **Browser 一覧の正本は Document 所有の素材台帳** — AssetId / AssetTable / fingerprint。
  **配置済み layer からの派生一覧は却下**(bin-first ワークフローの意味を再現できない)
- **1 Asset を複数 Clip が別加工で参照できる** — 同じ素材を2箇所へコピーして別々に色調整する
  回避策自体が構造上不要
- **TimeMap は clip-local → source の専用 field** — 画像 Effect stack へ移さない。
  key 追加/移動で Clip 尺を暗黙変更しない
- **Palette の所有は三分割** — Palette 名・順序・登録 Color は User Settings、選択と適用 target は
  Transient、**適用後の RGBA だけが Document**
- **`CommandId` 命名規約** — `motolii.` + `.` 区切りの小文字 ASCII 意味 segment。
  表示名 / 翻訳 / 物理入力 / 画面入口から生成しない(それらが変わっても ID 不変)
- **プロジェクトの保存意味論は NSDocument / Final Cut Pro 型の連続保存** — Save は checkpoint のみ
- **D1n: 外部改変検出は exact-byte revision** — mutation 直前に検出し write 0 の typed reject。
  watch/mtime は hint に留め**自動 merge しない**
- **未知 param は fail-closed、export 長は `nb_frames` 優先** — duration fallback、両方欠落は typed error
- **`projection_generation` 枯渇規則** — `u64::MAX` 到達時は `+1` せず typed 拒否。自動 retry しない

## 描画・空間

- **front-end は Makepad 確定、ゼロコピーは非交渉** — 合成器が表示可能な共有 Surface へ直接描き、
  front は handle を表示するだけ。CPU readback・毎フレーム Texture 再生成・最終 Texture からの
  GPU blit は通常経路で禁止。**プレイヘッドを Stage から切り離さない**
- **Motolii = Rerun の creator-facing wrapper** — 意図・時間・色・座標・seed・Preview/Export policy を
  Motolii が保ち、scene / view / query / camera / picking / renderer は Rerun。
  **direct `re_renderer` scene 操作・per-shape frame・第二 render runtime は禁止**
- **Rerun の内部 cache や GPU resource lifetime を authority にしない** — 完全 key・世代管理・
  部分無効化・pressure 対応は Motolii 側が補完する
- **Stage に独自 post-pass quad / shape 特例 / 近似 bounds preview を作らない** — 通常接続は
  すべて Rerun 経由の同一評価結果を使う。drag preview も Preview/Export と同じ D2 Command 評価
- **camera は document 所有、view camera は出力に出ない** — export は document camera で offscreen 撮影
- **boxcam** — カメラのビューは世界に破線 + ハンドルのボックスとして描き、**世界自体は縮めない**。
  視点は上縁タブ。**初期倍率は fit**
- **clipping mask は Motolii 側 authority のまま** — Rerun の segmentation / opacity へ委譲しない。
  一回使うだけの vector mask は中間 texture を作らない
- **Vism 出力の透明レイヤー表示は `GridMap` → `RectangleRenderer`** — `Mesh3D` は shader が
  texture alpha を捨て、`Image` は 3D view に描かれない。z=0 のまま `draw_order` で順序を作る
- **blend の逐次合成は fork(`re_renderer`)へアクセサを足す経路** — 半透明重なりのバイト不一致の
  真因は gamma 空間。**shader 数学を自 crate へ複製する案は却下**
- **audio device が唯一の clock owner** — 負荷時は video frame を間引き、
  **音声速度・pitch・project fps は変えない**。second clock を作らない
- **GPU RGB→YUV export を採択、CPU swscale は不使用**
- **`ffmpeg-sidecar` クレートは不採用** — 自前 ffprobe / ffmpeg pipe + GPU YUV 変換 + CFR seek を維持
- **`re_video` は fixture decode pattern の利用に限る** — 製品 decoder の authority を置換しない
- **cache 配置 / 追い出しは Host 専権** — 完全 key、透明な miss、hard budget。
  render 内の hidden state は禁止(Host 所有の StateTrack / Bake は可)
- **VRAM 作業セットと RAM/disk 容量は別階層** — admission 前 hard cap、capacity/deadline 別制御
- **中間 target は固定2枚でなく動的 pool** — 直列2枚を下限に必要数へ伸長

## タイムライン・編集の意味

- **BPM グリッドは持つが、モデル上の置き場は LFO 拡張と同じ束で決める**(利用者裁定 2026-08-28):
  hero「BPM グリッドへ手で合わせる」の直訳で、タイムラインは映像でなく**音楽のタイムライン**
  (CANON)。4.7 試験: AE 利用者はビートに置くために秒↔フレーム換算や式で回避してきた —
  やりたかったのは「拍に置く」。**ただし v1 では作らない** — hero 動線はロケータ+耳
  (聞きながら置いて印を打つ)で成立する。BPM が本領を発揮するのは**音同期の自動制御**
  (ParamDriver / DAW の tempo-synced LFO 相当)で、モデル上の置き場はその拡張と同時に決める。
  乗り物は Lottie の車体(marker `cm` / meta の慣習)で行ける見込みがあり、**新 component を
  先に切らない**(総監督が同日「Composition の静的値」と裁定しかけたのを利用者が訂正 —
  拡張側の設計を先取りで縛るため)。テンポ変化(tempo map)も同じ束。先例 = Ableton
- **Timeline 土台は AE 型の絶対時間・自由配置** — gapless packing 前提の trim family
  (ripple / roll / slip / slide / insert / overwrite / lift / extract / sync lock)は**採らない**。
  自由配置の上に個別追加すると既存 move/trim gesture と機構的に衝突する(**漏れではなく設計上の除外**)
- **AM 式の高度イージングは区間の補間型(`Interp` variant)** — Bounce / Elastic / Steps / Elastic Steps。
  式や driver ではない。**適用しても key 数・時刻・値は不変(非破壊差し替え)**。
  データモデルは既に一致 — `Interp::Bezier{x1,y1,x2,y2}` は CSS `cubic-bezier()` = Flow = AM と
  同一表現で **fps・解像度に非依存**、UI は4値を編集する薄いポップアップで足りる
- **Multi-key Graph View(U4e)と Easing Graph(U4b)は別 surface** — 同じ curve 正本を共有するが、
  隣接1区間用の Easing Graph とは別の常設 surface。**Graph View / Interval Easing Editor /
  空間モーションパスの三者を同じ curve state・座標・操作面へ統合しない**
- **区間イージングの対象は「選択されたキー」**(裁定274)。
  **※ 隣接する既決あり** — 「キーフレーム**値**編集は current playhead に exact 一致する key だけを
  対象。Auto Key や値域全体置換は不採用。別時刻の値を変えたいなら先に Add Key」(2026-08-04)。
  値編集とイージング対象は別の問いだが原理が隣接するため、**実装前に突き合わせること**
- **Add Position Key の挿入意味** — 明示 Add Position Key だけが fresh `KeyframeId` を導入。
  同時刻は no-op、Const 値からは1 Linear key、既存 off-key 値からは curve-preserving 挿入
- **キードラッグ(時刻移動)は release 時に1回だけ dispatch** — drag 中は local preview、
  cancel で復元。同時刻の二重 key は型不変条件で禁止
- **Timeline はキーフレーム方式を維持する(エンベロープではない)** — Position/Rotation に値域が無く
  DAW 型エンベロープが成立しない
- **Timeline は時間の操作に集約する** — **キーの縦ドラッグによる値編集は持たない**。
  rail は時間面外のコントロール列としての唯一の例外
- **Timeline の行高は固定・最小 20px** — 可変トラック高 / Optimize Height /
  Waveform Vertical Zoom は不要(**縦は情報を持たない**)
- **object bar は読み取り専用** — 状態変更は Inspector と keymap だけが持つ(誤爆コストの非対称)。
  mute は面を沈めハッチ、solo は他を沈める
- **ホイールの面割当は AE/Premiere 同型** — **素のホイール = 縦スクロール**、Shift = 横パン、
  Cmd/ピンチ = 横ズーム
- **編集操作は再生を止めない(Ableton 型)** — 音は soundtrack のみ由来、絵は毎フレーム snapshot 再読。
  playhead 手動移動と `SetSoundtrack` だけ audio session を開き直す
- **再生パイプラインは単一入口・単一 clock** — ゼロ source composition でも `composition.duration` は
  正準尺として存在し、無音は正規の supply。**second clock や UI 側 offset を作らない**
- **音声ファイルのドロップは soundtrack 既定** — 未設定なら soundtrack として貼る
  (offset 0 / gain 1.0、admit と同一 gesture で Undo 一発)。**動画は常に clip**
- **素材不足時は Freeze 既定を維持し、必ず利用者へ問う** — 黙って処理せず
  Freeze / 引き伸ばし(TimeMap speed)/ Loop を明示選択させる
- **キーフレーム marker/glyph は形状で統一し、色だけが逸脱を3段階で示す** — 文字コードでなく形
- **レイヤーの minimap は作らない** — 行の一覧が既に全体図(死に機能)。
  時間方向のナビゲータ帯だけを置く
- **ロケータは版を上げない** — 空なら書き出さず、旧 reader は未知キーとして往復させる
- **Timeline の比率は実測値が正本** — セル比率(小目盛間隔/行高) = 0.52、bar inset = 0.154 × 行高、
  bar 角丸 = 0.111、ruler 高さ = 0.846 × 行高。**ruler 数字は消さず保持**
- **時間方向の周波数分担** — 帯 = 大目盛(面)、縦線 = 全目盛(線)
- **move/trim は `RationalTime` 厳密演算、payload は絶対値**(差分ではない)。`duration > 0` 必須
- **Depth Rail** — 同一 Z は count 付き stack、**z=0 既定群は個別に描かず灰色1塊へ統合**
  (個別化されること自体が逸脱の合図)
- **Playhead の所有は Project session** — fresh open 時は以前の位置を復元せず決定的な安全初期位置へ。
  ruler primary press 即時採用、Escape / focus loss で press 時刻へ復元
- **Stage 空きクリックは primary selection を解除する** — Document write は発生しない。
  layer 単位の特異 transform は projection 全体を `Err` にせず当該 layer を `Unavailable` に

## UI の語彙・振る舞い

- **UI 動詞は意図を語り機構を語らない** — parent は内部機構として維持しつつ、露出はグループ化動詞のみ
  (⌘G = Group 生成 + parent 付与を1 undo、⌘⇧G = 解除 + world 位置焼き込み)。
  **AE 型の Parent 列 / pick-whip UI は不採用**
- **M と L は直交** — M は出力の話(出ない/触れる)、L は操作の話(出る/触れない)
- **入口が無い操作には UI を発明せず、意図ショートカットを足す**(2026-08-30 利用者裁定)。
  「ショートカットのハードコードはせず意味ショートカットなので後からいくらでも config 設定できる、
  **ない入口を作るより安上がり**」。キーマップは**意図名**に結んだ表(裁定174)なので、
  割り当ては後から差し替えられる。**ボタンやメニューを増やす前に、まずショートカットで届かせる**
- **品質バー Q0(触達性)が最優先** — 見えて触れそうな要素は必ず実機能へ接続、未実装 chrome は撤去。
  数値予算: 定常 p99 ≤ 8ms、gesture 連続フレーム落ち禁止、preview ≤ 1フレーム / 往復 ≤ 2フレーム、
  Stage 評価 p95 ≤ 16ms、50ms 超スパイク禁止、UI thread 4ms 超同期禁止
- **Inspector の key 追加は Rive パターン** — animatable property の隣に key button。
  灰色 outline(未アニメート)/ accent outline(他時刻に key あり)/ accent fill(現在時刻に key あり)。
  Position/Scale は Vec2 1property として header + X/Y 子値へ。
  **accept された snapshot だけが表示を進め、local optimistic state を持たない**
- **Inspector は parameter の意味を勝手に推測しない** — `param_id` から意味 icon を決め打ちせず、
  **明示 min/max のない値へ正規化 meter/rail/arc を描かず、見かけの上限を発明しない**。
  数値面は endless encoder 相当の連続 tick + 中心 pointer。
  **Object 選択・clip 操作・key 追加/削除/補間/playhead は Stage/Timeline が所有し Inspector へ複製しない**
- **Timeline / Inspector / key 編集は Godot(MIT)を全面 PORT** — 最低限の移植ではない。
  Godot の chrome / theme / Node 型は持ち込まない。**GPL 先例(Blender)は clean-room 必須**
  (copy / 翻訳 / port / vendor 禁止)、**MIT 先例(Godot)は PORT 可**
- **Browser は4面固定 + Palette 独自階層** — `Media / Effects / Create / Palettes`。
  上段 tab は横 scroll で幅を潰さない。**固定標準 swatch は作らない**
- **Browser 結果表示は thumbnail-only / thumbnail+name / list の共通 View toggle** —
  card 内は item 名を tag より優先、選択表示は常に1 item だけ
- **UI label は1行固定・折り返さず elide** — 全文と詳細は下部 Info / tooltip / focus へ。
  **文字列長で layout を変えない**
- **値セルの表示は6文字以内へ丸める** — `MAX_VALUE_CELL_CHARS=6` から1桁ずつ精度を落とす。
  **編集 draft は全精度を保つ**(表示は丸め、編集は真値)
- **余白は比率で定数化** — `MARGIN_RATIO=0.30` を頂点に半分ずつ折る梯子 {0.30, 0.15, 0.075} × 行高。
  **段の中間値の発明は禁止**
- **文字余白は em 基準** — 横 0.6em / 縦 0.3em、行送り 1.5(読み物)/ 1.35(密ラベル)、
  文字寸 = 0.42 × 行高。**文字は隣の箱に決して入らず、あふれは省略記号で切る**
- **UI トンマナは Ableton 風・情報密度重視。新しい視覚言語は発明しない** — テーマは
  `Motolii Dark` / `Motolii Light` 同格、**初回既定は Dark**。UI 文字は英語
- **S 空間スコアの定数** — Fitts 到達のヒット下限 12×12px(Motolii 実測由来)、
  S2 は KLM 予測秒数(P1.1 / K0.2 / H0.4 / M1.35)、S4 視覚重みは WCAG コントラスト比
  (弱 3:1 / 中 4.5:1 / 強 7:1)。**F/Z パターンと黄金比の特定値は根拠なしとして棄却**
- **共通 interaction 状態機械は6状態のみ** — Discover / Target / Preview / Commit / Cancel /
  Inspect / Transient。Preview は任意、Commit は queue 受理後の不可逆点、
  **Cancel は target / Preview からのみ**
- **Keymap は toolkit 非依存 Gesture の閉集合** — Keyboard / ModifierPointer / KeyToggle のみ。
  **scancode / toolkit 型は持たない**。Primary は OS command modifier へ展開
- **File 系の path 選択は `rfd` native dialog** — **自前 file browser を作らない**
- **Timeline / Stage / Browser / Inspector は bundled first-party Host module であり plugin ではない**
- **マスコットは 8×8・2色のカササギ** — 立つ / 沈む / 浮くの3フレーム、移動は跳躍。ペット機能は延期
- **panel 配置は一般モデル** — dock stack / detached top-level / hidden。矩形計算だけ Taffy へ委ねる。
  **Host snapshot / selection / playhead は単一 owner で全 window が読み取り専用投影**
- **未配置 control は Host 所有 staging surface へ一時配置** — 新しい ControlId / registry /
  plugin UI / Document field は作らない

## 拡張(Vism / plugin)

- **Vism は配布規格でなく LLM 並列生産の道路** — 要求は「配布できること」でなく
  **「LLM が単独で書けること」**(契約が小さく明示的・ホスト内部を知らずに書ける・単体で試せる・
  並行して衝突しない)。marketplace / カタログ / ガバナンスは後回し
- **plugin 信頼境界(TCB)** — TCB は Controlled Microkernel と明示 admit された Host capability
  module だけ。**「core plugin」分類は作らない**。公開 plugin 境界を通る code は first/third を問わず
  非信頼。**任意 native dylib を editor process へ load しない**
- **悪性 Vism は ambient authority 0・default deny で判定** — 供給元表示ではなく
  明示 capability・Host 所有 hard budget・atomic install の authority 境界で
- **外部の「plugin」概念は責任分離される** — 値評価 = ParamDriver、0-input 生成 = LayerSource、
  Document 変更 = Authoring Tool/Kit、逐次状態 = Simulation/Bake、外部 DCC = 明示 bridge。
  **1 trait へ潰さない**
- **作者 program の公式言語は TypeScript**(ECMAScript 2024 + TypeScript 7.0.2)— closed allowlist で
  決定論と ambient authority 0。**Node / npm / DOM 互換は契約しない**。WGSL は段階開示の GPU kernel 席
- **Vism SDK は typed domain object 契約** — opaque ID・hidden state・raw particle array・
  万能 namespace・Cavalry 互換は拒否
- **Vism parameter の `List(ElementType)` は list 全体で1キー** — 要素ごとの独立タイミングは不採用
  (添字 identity は Blender の既知の破綻例)。ただし添字を identity へ焼き込まない
- **Vism は 1 Vism = 1独立 crate / contract / fixture / artifact** — 共有公開 API・Document・
  永続形式・plugin kind・Host resource 契約は先に直列締結し、以降の追加が他 Vism を変更しない
- **BPM / 拍リズムの所有者は Core でなく「BPM Rhythm Vism」** — Core は時刻・型・接続だけ。
  既存 `Document.bpm` は pre-Vism 互換入力として当面保持するのみ
- **創作者と開発者を固定身分にしない** — Use → Tune → Compose → Inspect → Fork → Author →
  Publish → Reuse の一経路。通常 UI に言語 / seat / payload / runtime 選択を出さない
- **Document raw recipe / Contract / Catalog / Executor Registry は別レイヤー** — 保存可・
  意味検証可・実行可を同一視しない。**open は install / network / build / arbitrary plugin code を
  起こさない**
- **Shared Effect の削除ライフサイクル** — 参照中 Definition の削除は Reject、Unlink は Use だけ除去、
  Copy Local は未知 field / nested ID を再採番、最後の Use 後も orphan として保持。
  **Undo は counter を巻き戻さない**
- **`RenderCtx` は `#[non_exhaustive]`、予約 field は実装を意味しない**
- **Vism 拡張子は `.vism` で確定** — container / file 形式・loader は未決(**実装未許可**)
- **External Authoring Bridge は first-party 専用にしない** — Core に対応 app allowlist や
  vendor 型を置かない
- **Expression はコアに入れず拡張として戸を残す**

## 「しない」と決めた物

- **調整レイヤーを持たない** — 代わりに普通のシェイプへ**「背景を読む」エフェクト**
  (CSS `backdrop-filter` 型)。シェイプ自体が範囲を表すのでマスクの二重化が消える。
  全体にかけたいなら**全画面の矩形を描く**(見えている物が作用する)
- **プリコンポを持たない** — Group は入れ子コンポではなく同一タイムライン上のグループ
- **3D レイヤーのチェックボックスを持たない** — 空間は最初から3D、z=0 はモードではない。
  AE の切替は2Dから始まった歴史の請求書で、利用者に「3Dにする」を覚えさせている
- **ストップウォッチは AE 方式のまま**(離れない側の記録) — 押すまでキーにならない。
  値を試すたびにキーが打たれる方が当たり前から外れる(2026-08-29 利用者裁定)
- **gapless packing 前提の trim family を採らない**(ripple / roll / slip / slide / insert /
  overwrite / lift / extract / sync lock)
- **AE 型 Parent 列・pick-whip UI は不採用**
- **キーの縦ドラッグによる値編集を持たない**
- **レイヤーの minimap は作らない**
- **固定標準 swatch は作らない**
- **自前 file browser を作らない**
- **ギズモはスクラッチのまま実装しない**
- **ペイント族はコア完成後の拡張**
- **Fast Previews / メモリ・ディスクキャッシュは保留** — ゼロコピー下で何が低解像度を要求するのか未確定
- **プリレンダー族は DAW 由来の freeze 動詞で代用** — 明示固定 + 同期保持
- **v1 の外部出力は ExportJob の音声 mux 込み完成映像だけ** — **Lottie / animated SVG / OTIO /
  別 Host project / Web runtime package / 外部 service publish は完成条件外**。
  将来のため Delivery Adapter capability の席だけ残す
- **中央 marketplace・決済・購入者管理・DL ランキング・公式必須セットは運営しない** —
  価格 / license を品質・trust・公式順位へ変換しない
- **M4 の cache は Salsa、全面 HybridCache、独自 DB / WAL / runtime / parser を作らない**
- **Generator は p5.js 互換を撤回、ShapeScript + SVG adapter が正準**
- **`ffmpeg-sidecar` クレートは不採用**
- **D2 macro は現状 Delete + RemoveTrackItem のみ** — target 推測や適用後 Cancel はしない
- **Text Motion 縦切りは Random Entrance + Character Score + timing read-only 投影に限定** —
  override 書き戻しは gate 通過まで発注不可

---

## 上書きされている記述(引かないこと)

抽出元の台帳に残っているが、後の裁定で覆されている物:

- **「UI 基盤スタックの最終形 = shell:React Native / Timeline・Curve・Stage overlay:rust-skia /
  Stage preview:wgpu」**(2026-08-07)→ **裁定251(2026-08-20)で front = Makepad に確定**。
  RN も rust-skia も現行ではない
- **「Rerun 関連の `re_ui` / `egui_tiles` / egui callback の vendoring 凍結(UI runtime 再選定まで)」**
  (2026-07-20)→ 再選定は完了している
- **「旧 UI スタック(egui / direct-wgpu / Vello / WebView)は凍結」**(2026-08-07)→ 方針は生きているが、
  2026-08-27 の世界分断で `crates/` `next/` ごと生きた世界の外に出た

## 判定できなかった物(抽出担当4名が挙げた)

- **Document 実体を rerun store の pin fork にする決定**(2026-08-20)と、三本柱の整理
  (モデル = Lottie、2026-08-27)の関係。上書きされたか未確認
- **裁定54「ログと構造の強制」**(UiIntent journal・単一書き込みゲートウェイ・replay oracle)—
  原則(型付き journal・単一 writer)が Makepad 移行後も生きているか、iced 世代限定の死んだ機構か
- **P04-C2 の strict-interior Position 読み取り規則**(Easing trigger の `ACTIVE-INTERVAL` 判定)—
  Easing 実装者が独自の区間判定を再発明しかねない領域だが、一般化した1文へ圧縮できなかった
