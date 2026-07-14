# イシュー候補台帳(バックログ)

最終更新: 2026-07-13

このファイルは**今後のイシュー候補を1枚で俯瞰する台帳**。個々のマイルストーン仕様(`specs/M*.md`)のタスク表と重複させず、**それらを束ねる観点・横断的関心事・まだ仕様化されていないギャップ・v2の明示的な先送り**を追跡する。GitHub issue を起こす際の原本にする。

- 各行の「関連」は[落とし穴カタログ](pitfalls-and-roadmap.md)や仕様書のIDを指す。issue本文には必ず該当IDを引く(台帳とチケットを相互リンクする)。
- 完了条件は**自動判定(cargo test / ゴールデン / プロパティテスト)**を原則にする([AGENTS.md](../AGENTS.md)の規律)。
- これは生きたドキュメント。状態が変わったら更新する。

---

## 現在地サマリ(2026-07-10)

- **フェーズ**: **凍結ゲート宣言済み**(2026-07-10)。M2〜M5並列レーン解禁可。宣言文: [reviews/2026-07-10-freeze-gate-declaration.md](reviews/2026-07-10-freeze-gate-declaration.md)。
- **M2**: **コア締結**(2026-07-14)。宣言=[reviews/2026-07-14-m2-core-closure.md](reviews/2026-07-14-m2-core-closure.md)。残チケット=D5(#144)。PP-GateはM3前追跡(M2 blockerではない)。
- **M1のゴール(出口デモ)**: 達成。`samples/exit-demo/` + E2Eゴールデン緑。詳細は[M1仕様](specs/M1-vertical-slice.md)。
- **キーフレームUI決定(2026-07-09)**: AE式グラフビューは作らず、**Flow/アライトモーション式の区間イージングポップアップ**(cubic-bezier 4値、fps非依存)。空間モーションパスは別概念でv1コア外。
- **スコープ決定(2026-07-09)**: **解析駆動は最終フェーズに後回し**。DataTrack/ParamDriverの“口”は凍結ゲートで予約。
- **クレート**: `motolii-core` / `motolii-media` / `motolii-gpu` / `motolii-eval` / `motolii-nodes` / `motolii-plugin` / `motolii-render` / `motolii-export` / `motolii-cli` / `motolii-testkit` / `motolii-doc`。
- **未検証の最大リスク**: ~~S1 Slint実機(INF-1)~~ → **完了**(2026-07-11・人間OK・Slint採用確定)。次のリスクはM3実装ガード(深いIME/タイムライン描画)側。
- **凍結ゲート状態**: **宣言**。改訂は宣言文書の解凍手続き(理由+migrate+ゴールデン)を通す。
- **コントリビュータ導線(2026-07-10追記)**: 「乗ってもいいか」の最大欠落は視覚的証拠(GAP-11)と成功までの摩擦(GAP-9)。LLM委任の成否は**人間差し戻しをCIに移す**(INF-7)に依存。[plugin-authoring.md](plugin-authoring.md)§7の目視チェックリスト→機械判定が最安の一手。
- **M2恒久焼き込みの予防(2026-07-12追記)**: 出戻りが最小の窓で予防を第一選択にする(H-4)。手順正本=[reviews/2026-07-12-m2-permanence-prevention.md](reviews/2026-07-12-m2-permanence-prevention.md)。先人対比=[rework-prior-art](reviews/2026-07-12-rework-prior-art.md)。運用入口=[AGENTS.md](../AGENTS.md)。

### 優先度の目安
- **P0**: これが崩れると前提が失われる/多数のチケットの前提。今すぐ着手可能なものは最優先。
- **P1**: 後付けが最も高くつく基礎。凍結ゲート前後で位置を確定させる。
- **P2**: 実用に必要だが、垂直スライス確立後で間に合う。

---

## ① 凍結ゲート トラッキング(Epic)

M1完了後、**実際に動いたインターフェースだけ**を凍結して並列開発を解禁する関門([pitfalls: 凍結ゲート](pitfalls-and-roadmap.md))。1エピック+チェックリストで管理する。

| ID | タイトル | コード実証による完了条件 | 状態 | 関連 |
|---|---|---|---|---|
| FG-1 | [Epic] 凍結ゲート: G-1入場条件をコード実証で凍結 | [残件表](reviews/2026-07-10-freeze-gate-remaining.md) FG-C1〜C6全緑 | **宣言済み** | [宣言](reviews/2026-07-10-freeze-gate-declaration.md) |
| FG-1a / FG-C5 | F-1 正準座標 + Draft/Final一致 | Overlay解像度横断 + 重心一致ゴールデン | **完了** | F-1 |
| FG-1b / FG-C3 | F-2 単一writer骨格 | `DocumentWriter`のみ`&mut Document`、読み手は`Arc<Document>` | **完了** | F-2 |
| FG-1c | F-3 単一評価モデル(M1部分集合) | ソース→オーバーレイ→合成ゴールデン達。マスク/グループはM2 | 部分達(M1分・許容) | F-3 |
| FG-1d / FG-C2 | F-4 TimeMap製品経路 | export/`BackgroundTextureRequest`が`TimeMap::map`経由 | **完了** | F-4 |
| FG-1e / FG-C1 | G-1 プラグイン種別レジストリ経由 | Filter/ParamDriver/CompositeのPluginステップゴールデン | **完了** | G-1 |
| FG-C4 | param移行枠+旧JSON roundtrip | migrateで`amp`→`amplitude`を吸収 | **完了** | G-1 param |
| FG-C6 | InstanceIndex / CompLookbehind 型予約 | Rust型+serde | **完了** | F-7/F-11 |
| FG-2 | F-12 時間軸自由度の口を予約(`PluginKind::Simulation`済 / `SimulationPlugin` trait叩き台確定 / `TemporalFootprint`フィールドセット / スキーマにシムノードの席 / K1キー整合)。**凍結宣言後のため解凍手続き(3点セット)を通す** | 口のroundtrip保存テスト+設計文書の凍結(コード実証=参照パーティクルはv1.x、SIM-1) | 残(設計済([simulation-model.md](simulation-model.md))・enum予約済) | F-12 |

---

## ② 横断・インフラ(マイルストーン表に無い/薄い)

| ID | タイトル | なぜ必要か | 完了条件(自動判定寄り) | 優先 | 状態 | 関連 |
|---|---|---|---|---|---|---|
| INF-1 | **S1 Slintスパイクを実機GUIで実走**しIME/スレッド分離/Manualデバイス共有の合否を記録 | ヘッドレスで未実走=**UI基盤(Slint)前提が未検証**。IME不合格なら基盤が覆る | `docs/spikes/s1-slint.md`に合否記録(特にIME)。M3仕様を確定 | **P0** | **完了**(2026-07-11・人間OK・Slint採用確定。証拠=`docs/spikes/s1-evidence/`) | M0-S1 |
| INF-2 | 性能回帰ハーネス(1080p×40レイヤ目標のフレーム時間をCI計測) | performance-modelの目標に**CIガードが無い**。VRAM常駐破壊の混入を数値検出 | 基準比で閾値超過をCIが検出 | P1 | 未着手 | performance-model |
| INF-3 | 実GPUベンダ差の方針(golden=lavapipe固定、実機Final出力の許容/非再現を明文化) | 出荷Finalはユーザ実GPUで走る。再現性方針が未定義 | 方針をdocs化+許容誤差の根拠 | P1 | 未着手 | INF/color |
| INF-4 | device lost / VRAM OOM 復帰の系統設計 | 長尺・4Kで必ず発生。全リソース再生成の契約 | device lost注入で復帰するテスト | P1 | 部分(R1でGPU復帰に言及) | robustness |
| INF-5 | キャッシュ並行契約をloomで検証(参照カウント遅延解放/ロック1段) | **Natronの死因**(cache deadlock)の予防 | loomでデッドロック無しを確認 | P1 | 未着手 | F-2, M4-K1 |
| INF-6 | 常時保存(コマンドジャーナル+定期スナップショット)の復元テスト | プロセスkillでも作業を失わない | kill→再起動→復元の統合テスト | P1 | 未着手 | M2, B-1追記 |
| INF-7 | **[Epic] plugin-authoringチェックリストの機械化** — 人間差し戻しをCIに移し、LLM委任の往復を「マージ前の最後の1回」にする | 目視チェックリストのままではLLMが検証を回せない。人間リターンが3回続くと貢献者が去る(D-2の裏返し) | 下表INF-7a〜7fが緑。`AGENTS.md`の提出前1コマンドで§7相当が機械判定される | **P1** | **a〜g完了**(Epic達成) | D-2, F-8, F-9, plugin-authoring §7 |
| INF-7a | ベンダー/OS固有API deny(`cargo-deny` / 依存・ソースgrep) | CUDA/Metal/DX系crate・製品経路のベンダーAPI参照をCIで落とす | deny設定+違反負例がCI赤、参照プラグインは緑 | **P1**(容易・先) | **完了**(conformanceスキャナ。GPUベンダー系のみ。`windows*`は対象外=F-9本命に合わせる) | F-9, §3-1 |
| INF-7b | 公開APIの`assert!`/panic禁止をCI化 | `motolii-plugin`公開面と参照実装でclippy/`unwrap`方針を機械判定 | lint設定+違反負例が赤。入力起因は`PluginError`経路のみ | **P1**(容易・先) | **完了**(`[lints.clippy]`+conformance。allowは`mod tests`のみ) | AGENTS実装規約, §3-7 |
| INF-7c | `NodeDesc`必須欄の検証関数をテストで強制 | `validate_node_desc(&NodeDesc) -> Result`を置き、全参照プラグイン+レジストリ登録時に呼ぶ | 欠けたdescの負例が赤、参照実装が緑。§7「メタデータ完備」が目視不要 | **P1**(容易・先) | **完了** | F-8, §2 |
| INF-7d | AGENTS.mdに**提出前1コマンド**を明記し、checklist検証をそのコマンドに含める | LLMは指示された検証は回すが、散文チェックリストは回せない | `cargo test -p motolii-plugin`(+deny/lint)が§7の機械化分をカバーする旨をAGENTS.mdに1行で書く。ドキュメントとCIが一致 | **P1**(容易・先) | **完了** | AGENTS, D-2 |
| INF-7e | `new-plugin`スケルトン生成(規約準拠の型紙を吐く) | ClearFilterコピーより「正しい状態から開始」させる。LLMの初期状態を規約準拠に固定 | スクリプト1発でFilter/ParamDriver等のスケルトン+空`desc`+テストスタブが生成され、INF-7c検証を通る | P1 | **完了**(`scripts/new-plugin.sh` + `tests/new_plugin_scaffold.rs`) | plugin-authoring §4/§5 |
| INF-7f | 純関数契約のプロパティテストをtestkit標準装備 | 同じ`t`+入力で2回呼び→同一出力。隠れた`&self`状態の検出器 | testkitヘルパー+参照プラグイン1つ以上で緑。新規プラグインの推奨完了条件に明記 | P1(中程度) | **完了**(`motolii_testkit::purity` + Clear/Tint/Sine緑、stateful負例) | §3-3, 純関数契約 |
| INF-7g | (実演) LLMにプラグイン1個を書かせ、**人間レビュー無しでCI緑まで**通し、記録を残す | READMEの「LLM-driven」宣言の証拠=バス係数への答え。INF-7a〜dが揃った後 | レビュー記録(プロンプト・差し戻し回数=CIのみ・マージPR)を`docs/reviews/`に残す | P2 | **完了**(`core.filter.opacity` + [記録](reviews/2026-07-11-INF-7g-llm-plugin-demo.md)) | INF-7, concept |
| INF-8 | **DX: WGSLホットリロード(開発ビルド限定)+高速起動計測+Slint live-preview評価** — AE型「再起動地獄」の予防([dev-experience.md](dev-experience.md))。**非ブロッキング評価**: どれが不合格でも基盤採否には影響しない | プラグイン作者(人間・LLM目視局面)の反復速度は採用の入口。パイプラインはホスト所有なのでホスト単独で差し替え可能 | (a) devビルドでWGSL編集→次描画に反映(該当キャッシュ無効化込み)の統合テスト、コンパイルエラー時に直前パイプラインで描画継続 (b) 起動→INF-6復元の所要時間を計測記録 (c) Slint live-previewを**プロパティ/モデル/コールバック保持・反映遅延・構文エラー時の復旧**の3基準で評価記録(実走はINF-1と同じ実機作業でよい) | P2 | 未着手(a/bはINF-6・M4キャッシュ後が安い。cはINF-1実走時) | dev-experience, INF-6, F-10, M4, INF-1 |

補足: ループ内GPU生成の検出は**INF-2(性能ハーネス)**が実質の機械判定器。正準座標のpx禁止は型で縛る設計変更込みのため**GAP-10**へ分離。

---

## ③ まだ未ドキュメントの新規ギャップ

既存ドキュメントに見当たらず、後で負債化する基礎観点。**ここが優先的にissue化すべき本命。** 行が「決定済み/実装待ち」と明記されているものは再issue化せず、関連Issueの実装完了で閉じる。

| ID | タイトル | なぜ後で痛いか | 完了条件(方向) | 優先 | 関連 |
|---|---|---|---|---|---|
| GAP-1 | **フォント/テキスト基盤**の実装(M5-P6)。分界・スタックは決定済(fontique+harfrust+Vello `draw_glyphs`、組版はプラグイン) | 歌詞組版=主用途の第1号前提。未実装だと文字レイヤーが存在しない | P6ゴールデン(かな漢字・フォールバック)緑 | **P1** | F-6, M5-P6, [references.md](references.md) |
| GAP-8 | **シェイプ間リンク(レイヤー参照付きParamSource)** — LookAt/Follow/ParentRef。AEエクスプレッション非採用の代替 | **M横断最大ギャップ**。現行ParamSourceに別レイヤー参照が無く「向ける・追従」が式か手キーフレームに戻る | M2スキーマ+motolii-eval評価+F-3順序+M3ターゲットピッカー+M4無効化伝播の一括設計 | **P1** | concept, M2-D1/D3, M3, M4-K2, F-3 |
| GAP-2 | **プラグインのパラメータ同一性&バージョニング**(param IDは位置でなく安定ID、effect version + param移行) | doc全体のversion/migrationはあるが、**組込エフェクトのparam追加/改名/型変更で旧プロジェクトが壊れる**経路が未定義(AE/Premiereの版間破壊の定番) | param安定ID+effect versionのスキーマ、移行関数枠、roundtripテスト | **P1** | C-2, G-1 |
| GAP-3 | **メディア再リンク/オフライン素材**(相対/絶対パス、素材移動、欠落時UI) | NLEの基礎。プロジェクト移動で素材ロスト→再リンク導線が無いと実用不可 | パス解決規約+欠落検出+再リンクのモデル | P2 | M2(Asset) |
| GAP-4 | **Undoの粒度/coalescing**(ドラッグ=多数コマンドの結合)。ジャーナル整合はD1d(#105)担当で別レーン | **coalescing決定済み**(#103⑨): プロパティ単位atomic、1 gesture=1 macro、同一対象+同一propertyのdragをmerge。未ドキュメントではなく**実装待ち** | D2(#109)のgesture merge+apply/revertプロパティテスト | P2 | **実装待ち** / #109 / M2-D2 |
| GAP-5 | 書き出し色の実プレイヤー検証(内部sRGB近似 vs 出力bt709タグの既知ズレを実測・明文化) | 「書き出したら色が違う」の最終境界。近似の許容範囲を線引き | 実測レポート+許容範囲のdocs化 | P2 | F-5, B-3 |
| GAP-6 | 入力/ショートカット&アクセシビリティ(SlintのAccessKit、IME前提の入力設計) | 日本語IMEを一級にしたのに、キーマップ/a11yの土台が未計画 | 入力マップ方針+AccessKit有効化の確認 | P2 | M3 |
| GAP-7 | プロジェクト/素材のパッケージ化・可搬性(collect files相当) | 納品・バックアップ・別マシン移行で必要。スキーマに絡む | パッケージ形式の素性をスキーマに予約 | P2 | M2, F-5 |
| GAP-11 | **README冒頭の視覚的証拠**(M1出口デモのGIF/短尺動画)(※旧番号GAP-8はシェイプ間リンクと重複していたため振り直し) | モーショングラフィックスツールなのに動く証拠が無い=「難しそう」の最大シグナル。文章で乗る人は少数 | README最上部に出口デモのGIF/動画。生成手順をdocsかsamplesに1コマンドで再現可能 | **P1** | D-4, M1出口デモ |
| GAP-9 | **clone→1コマンド→mp4**の摩擦ゼロ化(`samples/exit-demo`) | ユーザー顔の15分成功体験が無いと、規律の壁だけが先に見える。ffmpeg/GPU/素材準備が脱落点 | 素材同梱・依存の明示・失敗時メッセージ(日英)。CIまたはドキュメント手順で「1コマンド成功」を再現 | **P1** | D-4, M1出口デモ |
| GAP-10 | `ParamDef`に単位型を持たせ正準座標(px禁止)を型で縛る | 散文+レビューではLLMが破り続ける。設計変更込みなのでINF-7の容易枠から外す | 空間paramが正準単位以外をコンパイル/検証で拒否。既存参照プラグイン移行+テスト | P2 | F-1, §3-4 |
| GAP-13 | **プラグインUIモデルの採否判断(宣言語彙 vs 自由描画)** — 設計仮説([plugin-ui-model.md](plugin-ui-model.md))は**現行M3§拡張方式(.slintカスタムパネル+plugin描画wgpu)と競合中**。**M3仕様確定の明示依存**(M3-2/M3-3は判断まで着手禁止、M3-1は着手可) | 「プラグインにUIをどこまで開けるか」は後戻り不能な公開契約で、両論が現存する矛盾状態(READMEの「矛盾はバグ」)。空間パラメータ(x/y同時操作)のUI形はM3設計の前提 | ①能力コーパス(代表有料プラグインUI 5〜10件の分解表)②AM実機確認 ③判定語付き採否(採用/縮小/延期/棄却)④採用ならM3§拡張方式を同時改訂+`ParamDef`拡張の**解凍手続き(必須。公開struct literal互換が壊れるため)**とconstructor/builder化の互換計画 | **P1** | 仮説文書化済・批判レビュー7点反映済・採否判断待ち | plugin-ui-model, M3, F-8, GAP-2, F-1 |
| GAP-12 | **パス演算子スタック(パス→パス)** — パンク・膨張/ジグザグ/パスのオフセット/角丸/トリムパス/ツイスト/パスのウィグル(+リピーター=F-7)。concept 2026-07-10決定でv1コア要件 | 「AEを選ぶ理由」そのもの(AM含む競合に無い)なのに、現行契約はテクスチャ語彙のみでパス→パスの口が無い。放置するとラスタライズ後の画像歪みFilterで代用され品質が死ぬ | M2シェイプスキーマに順序付き演算子スタック予約(Lottie `pb`/`zz`/`op`/`rd`/`tm`/`tw`/`rp`が前例)+v1ファーストパーティ実装(シェイプ/SVG/テキストパス共通)。`PluginKind::PathOp`化はv2判断 | **P1** | F-13, F-7, F-10, M2-D1, references(Lottie) |

---

## ④-0 最終フェーズ: 解析駆動(v1コア完成後、2026-07-09決定で後回し)

「映像解析→DataTrack→パラメータ駆動」の解析プロデューサ群。v2の「今やらない」とは別で、**v1コアの最後に実装する**位置づけ(このツール唯一の差別化=長期的な強みなので放棄はしない)。

| ID | タイトル | 関連 |
|---|---|---|
| ANA-1 | 色解析(支配色/色マスク重心)プラグイン → DataTrack生成 | 旧M4-K5, B-6 |
| ANA-2 | 時系列解析の区間キャッシュ + 部分再解析 | 旧M4-K3, B-5 |
| ANA-3 | オプティカルフロー/トラッキング(wgpu compute自前 vs OpenCV/ONNXを評価) | 旧M4-K5, B-6 |
| ANA-4 | 解析DataTrackでオーバーレイ/エフェクトを駆動するE2E(Traceryライク) | concept #2 |

## ④-1 v1.x: シミュレーションと時間窓(凍結ゲートで口を予約、実装はM4のK1/K7後)

物理シミュレーション(布・液体・パーティクル)と前後フレーム参照を、レンダ経路の純関数契約を壊さずに一級対応する(2026-07-10決定、落とし穴F-12)。設計は[simulation-model.md](simulation-model.md)に一元化。**コミュニティ先導のプラグイン開発が本格化する領域**なので、境界の凍結を最優先し、実装はコミュニティと並走できる形にする。

| ID | タイトル | 完了条件(方向) | 関連 |
|---|---|---|---|
| SIM-1 | StateTrack機構+標準パーティクルの最小L3(重力+風+平面衝突、wgpu compute)でコード実証(参照実装=製品第1弾) | ベイク→スクラブ→パラメータ変更→チェックポイントからの部分再シムのE2Eテスト、同一seed再現性テスト(lavapipe) | F-12, M4-K1/K7 |
| SIM-2 | 時間窓フィルタの実装(TemporalFootprint解決+キャッシュキーの窓拡張+TimeMap写像) | エコー/フレームブレンドのゴールデン、時間窓×TimeMap(逆再生)の整合テスト | F-12, F-4 |
| SIM-3 | モーションブラー(サブフレームサンプル型 or ベクター型を評価) | `Quality::effect_samples`でDraft/Fullが切り替わるゴールデン | concept決定, F-12 |
| SIM-4 | 布(バネ質点系)/2D流体(安定化ソルバ)プラグイン — コミュニティ/ファーストパーティ | 各ゴールデン+状態予算(`state_budget_bytes`)の遵守テスト | F-12 |
| SIM-5 | **標準搭載パーティクル(ファーストパーティ第2号)**: L0閉形式(重力+風+抗力の解析解、curlノイズ乱流、状態ゼロでスクラブ自由)+ L3昇格(衝突等のオプトイン)+ 音楽同期エミッション(BPMグリッド/DataTrack駆動)+ ライフカーブ(サイズ/色/不透明度) | L0の任意時刻アクセス性テスト(シークとシーケンシャルで同一出力)、L0↔L3切替のUI/スキーマ整合、ビート同期バーストのE2Eゴールデン | F-12, simulation-model§8, concept決定 |
| SIM-6 | **コライダー入力(他シェイプとの相互作用)**: `colliders: [LayerRef]`のスキーマ実装+ホスト正規化(シェイプ→SDFラスタライズ(JFA)+解析プリミティブ高速経路)+ 形状解釈`fill: そのまま\|外縁`(既定=そのまま。外縁=外部flood fillで穴を塗り潰してからSDF化)+ キャッシュキーへの参照レシピハッシュ算入+循環拒否 | 「動くシェイプでパーティクルが跳ねる」E2Eゴールデン、ドーナツSVGで「そのまま=穴に粒が溜まる/外縁=穴を通らない」の両モードテスト、コライダーレイヤー編集→影響時刻以降のみ再シムのテスト、循環参照のロード時拒否テスト | F-12, simulation-model§3.7 |

## ④ v2 明示バックログ(今やらない・スコープ膨張の可視化)

「やらないことリスト」を明示追跡し、スコープ膨張(D-4)を防ぐ。

| ID | タイトル | 関連 |
|---|---|---|
| V2-1 | 動的プラグインロード(C ABI / `abi_stable`)+配布 | A-2, G-1 |
| V2-2 | WASMパラメータプラグインのサンドボックス実運用 | 5-1 |
| V2-3 | ハードウェアデコード→wgpuゼロコピー(Recが先行例) | B-2, references |
| V2-4 | HDR/10bit + OCIO統合 | F-5, B-3 |
| V2-5 | OTIO書き出し | F-5, references |
| V2-6 | マルチOS動作保証(v1は開発主機1つに固定) | E |
| V2-7 | ディスクキャッシュ(解析結果の永続化) | M4未決事項 |
| V2-8 | **プラグインカスタムUIの解凍判断**: `.slint`実行時ロード / wgpu自由描画 / 宣言レイアウト(Blender UILayout型) / ギズモ。v1は`NodeDesc`自動生成のみで確定済み。解凍時も「標準パネルで全パラメータ操作可能」が不変条件 | [plugin-ui-v1-boundary](reviews/2026-07-12-plugin-ui-v1-boundary.md), M3 |
| V2-9 | 有料/公開プラグイン5〜10件の**UI能力コーパス**(カスタムパネル・ギズモ・スコープ等が実製品で何を要求するか)。v1縮小判断には不要 — **V2-8のどの口を開けるか**を決める調査 | V2-8 |

補足(2026-07-13): V2-1/V2-2の方式選定では**開発体験(実行時スワップによるホットリロード)**も評価軸に含める。純関数契約により差し替え=キャッシュ無効化で意味論が閉じるため、WASM案の追加根拠になる。Rust dylibのホットリロードは恒久不採用。詳細と先例(未カウンターレビュー)は[dev-experience.md](dev-experience.md)。

---

## ラベル体系(GitHub issue用)

- **優先度**: `P0` / `P1` / `P2`
- **マイルストーン**: `M1` / `M2` / `M3` / `M4` / `M5` / `v2` / `freeze-gate`
- **種別**: `foundation` / `perf` / `color` / `concurrency` / `plugin-api` / `text` / `assets` / `undo` / `ux` / `ci` / `robustness` / `spike` / `epic` / `contributor-loop` / `contributor-loop`
- **その他**: `blocker`(前提を塞ぐ) / `data-safety`

issue本文には必ず該当する落とし穴ID(`F-2`等)や仕様書ID(`M4-K1`等)を引用し、この台帳と相互リンクする。
