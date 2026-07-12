# 反対側レビュー: M3/プラグイン生態系の先例所見を最小化する(2026-07-12)

ステータス: **独立批判レビュー**(一次資料を確認できた範囲だけで判定。元調査の完遂・検証を引き継がない)

対象: [先例調査メモ](2026-07-12-prior-art-gap-survey.md)のA-1〜A-4 / B-1〜B-2 / C-1〜C-4

## 結論

先例が示す問題の多くは実在する。しかし、元調査は「将来起こり得る互換問題」を見つけると、すぐに恒久契約・バージョン交渉・validatorへ翻訳する傾向がある。Motolii v1は静的リンクで、プラグインパネルの9割を`NodeDesc`から自動生成する計画であり、動的な第三者配布はv2である。この現在地では、第一選択は互換機構の先行実装ではなく、**安定公開する境界を狭くして問題をまだ持ち込まないこと**である。

M3に対する推奨は次の4点に縮む。

1. **v1の安定パネル契約は`NodeDesc`からの自動生成だけ**とする。すべてのプラグインはこれだけで完全操作可能でなければならない
2. `.slint`カスタムパネルはファーストパーティ限定の実験機能、またはv2候補とする。v1の互換保証・言語バージョン交渉は作らない
3. DPIはSlintの論理ピクセルとホストUI層で処理し、`scale_factor`をプラグイン契約へ追加しない。マルチモニタ実機テストは残す
4. スレッド契約はv1ではホスト呼び出し側とtrait docで明示する。CLAP型の実行時thread-check/validatorは動的ロード境界を作るv2で再評価する

キーマップはM3の恒久物なので、`press/release/click/drag`のイベント種別、安定`CommandId`、不変な既定値+ユーザーデルタ、原本を壊さない移行だけを初期スキーマへ入れる。一般化されたhealing基盤や複数互換プリセットは、実際の移行事例が生じるまで作らない。

## 判定方法

各所見に対し、次の順で反対尋問した。

1. 元調査の歴史的事実を一次資料で確認できるか
2. 同じ失敗条件がMotolii v1にも存在するか
3. 提案対策より小さい「境界を公開しない」「ホスト側に閉じる」が使えないか
4. 今決めないとユーザーデータやプラグイン数に掛け算されるか

判定語:

- **採用**: M3初期契約に残す
- **縮小**: 問題は認めるが、より小さい対策へ置き換える
- **延期**: v2等、失敗条件が実在する時点まで送る
- **棄却**: 現行設計ですでに満たす、または根拠が不足するため新規作業にしない

## A. プラグインUI契約

### A-1. `.slint`バージョンスキュー — **縮小**

確認できたこと:

- Suilがホストと異なるUIツールキットを包む専用ライブラリであり、長期間にわたりGtk/Qt/X11/Cocoa等の組合せを保守していることは公式[README](https://gitlab.com/lv2/suil)と[NEWS](https://gitlab.com/lv2/suil/-/blob/main/NEWS)で確認できる
- 一方、「2023年にツールキット直埋め込みを全撤去」は不正確。0.10.20で削除されたのは**Gtk in Qt / Qt in Gtk wrappers**であり、現行SuilはX11等のネイティブUIをGtk/Qtへ埋め込む役割を継続している。2025年にはQt6上のX11対応も追加されている
- Slintは互換を無視しているわけではない。`slint-interpreter`には既定有効の`compat-1-2`があり、pre-1.0構文も互換目的で残された履歴がある([interpreter公式](https://docs.slint.dev/latest/docs/rust/slint_interpreter/)、[legacy syntax公式](https://releases.slint.dev/1.1.0/docs/slint/src/reference/legacy_syntax))

Motoliiへの転移条件は「異なるSlint世代で書かれた第三者`.slint`を、更新されたホストが長期にロードすること」である。v1静的リンク+ファーストパーティではまだ成立しない。

対処:

- v1はホストと同じロックファイルでビルド・チェックされた`.slint`だけを許可
- `slint-viewer --check`相当のコンパイル確認を同梱時に行い、失敗時は自動生成パネルへ戻す
- 言語バージョン宣言・互換範囲マトリクスはv2動的配布の設計時まで延期

### A-2. UI技術ハードコード — **縮小**

OpenFXがinteract描画をOpenGL前提にし、1.5でホスト定義の`OfxDrawSuiteV1`を追加した事実は[OpenFX公式仕様](https://openfx.readthedocs.io/en/latest/Reference/ofxInteracts.html)で確認できる。ただし、そこから直ちに「`.slint`言語バージョンネゴシエーションを初日から持つ」は導けない。OFXの問題はプラグインに低レベル描画APIと共有GL状態を直接渡したことであり、宣言的UIをホストが解釈するMotoliiとは故障面が異なる。

またSlint公式は、実行時interpretを事前コンパイルより遅く、メモリも多く使う方式と明記する([Slint C++ API](https://docs.slint.dev/latest/docs/cpp/))。Motoliiの「軽さ」目標では、互換機構より先に実行時カスタムUI自体の費用を測る必要がある。

対処:

- `slint`依存をUIシェルへ限定する既存M3ガード4は維持
- `.slint`をプラグインの**安定必須契約にしない**
- v1で試す場合もファーストパーティ限定、失敗可能な上乗せ機能として扱う

### A-3. カスタムUIによる体験分裂 — **採用**

OpenFXはhostがcustom interactをサポートしない場合を仕様化しており、同一プラグインのUI能力がhost capabilityで変わる([OpenFX公式仕様](https://openfx.readthedocs.io/en/latest/Reference/ofxInteracts.html))。Motoliiは単一ホストだが、カスタムパネルのロード失敗・旧版・無効化でも同じ分裂が起こる。

対処は小さい。**カスタムUIを持つプラグインも`NodeDesc`自動生成パネルだけで全パラメータを編集できること**を不変条件にする。カスタムUI固有にしか存在しない必須操作は禁止する。これはv1から固定してよい。

### A-4. DPI/スケールと処理状態の分離 — **縮小**

Cubase 11のDPI互換シムがOpenGL等のプラグインUIを正しく拡大できず、設定切替後に音声のノイズ/無音が起こり得ることは[Steinberg公式トラブルシュート](https://helpcenter.steinberg.de/hc/en-us/articles/360017509919-Cubase-11-Using-DPI-unaware-plug-ins-on-Windows)で確認できる。UI設定が処理側へ波及してはならない、という境界テストの動機は妥当である。

ただしMotoliiの`.slint`は別ツールキットのネイティブウィンドウを埋め込む方式ではない。Slintの`px`は論理ピクセルで、device pixel ratioへ自動追従する([Slint positioning公式](https://docs.slint.dev/latest/docs/slint/guide/language/coding/positioning-and-layouts/))。`scale_factor`をプラグイン契約へ追加すると、むしろUIツールキット都合を恒久境界へ漏らす。

対処:

- DPI/論理↔物理px変換はUIシェル所有
- M3ガード9の別モニタ実機スパイクは維持
- 「パネル再生成・ウィンドウ移動・scale変更でDocument/評価結果が変わらない」テストをホストUI側に置く
- プラグインへは本当にレイアウト判断が必要になった場合だけ論理viewport寸法を渡し、物理scaleは渡さない

## B. スレッド契約と規格統治

### B-1. CLAP型の関数単位スレッド注釈+validator — **縮小**

CLAPが各関数へ`[main-thread]`、`[audio-thread & active & processing]`等を記し、`thread-check` extensionと`clap-validator`を提供することは[公式plugin.h](https://github.com/free-audio/clap/blob/main/include/clap/plugin.h)、[thread-check.h](https://github.com/free-audio/clap/blob/main/include/clap/ext/thread-check.h)、[公式README](https://github.com/free-audio/clap)で確認できる。

ただしCLAPは、独立配布されたバイナリ同士をC ABIで接続し、リアルタイム音声スレッド制約まで持つ規格である。Motolii v1は同一workspaceの静的Rust traitで、呼び出し箇所をホストが所有する。doc-commentの文言をconformance走査する仕組みは、型でも実行時保証でもなく、LLMが注釈を満たすだけの新しい形式作業になり得る。

対処:

- v1: trait単位で「host render workerから呼ぶ」「UIスレッドから直接呼ばない」をdocと`plugin-authoring.md`へ1回明記
- UI callbackはDocumentコマンドを発行するだけとし、render traitを直接呼べない依存方向にする
- v2動的ロード時: thread roleの型、runtime check、validatorをセットで設計する
- 現時点で「注釈の無い公開関数を文字列走査でfail」は導入しない

### B-2. ライセンス・統治宣言 — **棄却(重複)**

CLAPがMITであることと、安定ABI/extension versioningを掲げることは[公式リポジトリ](https://github.com/free-audio/clap)で確認できる。一方、元調査のVST2廃止からCLAP誕生までの因果・発言は、今回取得できた一次資料だけでは独立確認できなかった。

Motoliiはすでにworkspace全体を`MIT OR Apache-2.0`とし、READMEでfork継続可能性と商用プラグイン許可を明記している。新たなconcept級宣言を足しても法的能力は増えない。「単一主体が契約を廃止できない」は、財団化や商標・著作権移管まで設計しない限り実効的な完了条件にならない。

対処:

- 新規ゲート項目にしない
- v2 SDKを別配布物にする時、そのmanifest/header/exampleにも同じSPDXとライセンスファイルが含まれることだけCIで確認する

## C. キーマップ/入力

### C-1. `press/click/drag`の区別 — **採用(根拠を縮小)**

Blenderの100–200ms遅延やissue #68970の因果は、今回アクセス可能な一次資料では再検証できなかった。ただしBlender公式マニュアル自体がKeymap Eventの値としてpress/release/click/dragを区別し、Tweakを独立した入力型として扱う([Blender Keymap Manual](https://docs.blender.org/manual/en/2.83/editors/preferences/keymap.html))。Motoliiのタイムライン操作もclick選択とdrag移動を同じ物理ボタンで使うため、この区別は外部失敗例なしでも必要である。

対処:

- GAP-6スキーマに`press/release/click/drag`を入れる
- click判定閾値・drag開始距離はユーザーデータでなくホスト入力ポリシーとして持つ
- 100–200msという数値をM3性能根拠には使わず、自身の入力レイテンシを測る

### C-2. 他アプリ互換プリセット — **縮小**

Blenderは現在、既定・旧2.7系・Industry Compatibleを提供しており、互換プリセットを全面放棄してはいない([Blender 5.0 Manual](https://docs.blender.org/manual/fr/5.0/editors/preferences/keymap.html))。したがって「Maya/3ds Maxプリセット削除→互換プリセットは成立しない」という一般化は強すぎる。

一方、Blenderの歴史的変更にはプリセットを最小にして競合を避ける意図が明記され、非Blender key configuration固有の修正も発生している([interaction preset commit](https://projects.blender.org/archive/blender-archive/commits/commit/24eedb2175896dd5d7e145486f3f3c6455511fca/source/blender/windowmanager)、[constraint fix](https://projects.blender.org/archive/blender-archive/commit/278fce1170fc095bebf4c7feae1570761359ff01))。保守費用の警告としては使える。

対処:

- M3ではMotolii標準だけを出荷し、全コマンドを再割当可能にする
- AE/AM互換は名称を先に約束せず、実操作モデルが揃った後に1つだけ評価

### C-3. 設定マイグレーションhealing — **縮小**

Blender 4.5.4で「一部キーマップ設定が正しくimportされない」修正(#146670)があったことはリリース情報から追跡でき、Blender公式マニュアルも個別keymap変更が新バージョンと衝突し得ると警告する([Blender Keymap Manual](https://docs.blender.org/manual/en/2.83/editors/preferences/keymap.html))。ただし「修正版は破損済み状態を治せずconfig削除しかなかった」という詳細は、今回アクセス可能な一次資料で確認できなかったため設計根拠にしない。

対処:

- 設定ファイルにformat versionを持つ
- 移行前原本を保持し、tempへ書いて検証後にatomic replaceする
- migrationは冪等、未知`CommandId`/未知フィールドは保持する
- 一般化した再マイグレーションhealing基盤は、実際に破損版を出した時点まで作らない

### C-4. 不変ベース+ユーザーデルタ — **採用**

「全プリセットへ無言伝播し4.5年以上未修正」という個別バグの期間は再検証できなかった。しかしBlenderは2011年に、ユーザー編集が既定keymap全体を置き換えず差分だけを保存する方式へ変更し、その目的を「builtin変更へ追従しやすくする」と明記している([KEYMAP REFACTORING commit](https://projects.blender.org/archive/blender-archive/commits/commit/c94fe5e2995873536cbdb180652b1aa027e4ef8d/source/blender/windowmanager))。公式マニュアルも変更分だけのexportを「keymap delta」と説明する。

対処:

- builtin presetは不変でversion管理
- user configは`CommandId -> Gesture`の追加/置換/無効化デルタ
- プリセット切替は別々のbaseに同じuser deltaを暗黙適用せず、baseごとのoverlayとして識別

## ゲートへの差分

### M3ゲートへ残す

- `slint`依存をUIシェルに限定するCI
- 全プラグインが`NodeDesc`自動生成パネルだけで完全操作可能、というフォールバック契約
- UI再生成/DPI/別モニタ移動がDocument・評価結果へ波及しないテスト
- GAP-6の最小スキーマ: 安定`CommandId`、イベント種別、不変base+user delta、version、原本保全migration
- UI callbackからrender traitを直接呼ばない依存方向

### M3ゲートから外す/延期する

- `.slint`言語バージョン宣言と互換テストマトリクス → v2動的配布前
- DPI `scale_factor`のプラグイン必須フィールド → **採用しない**
- 全plugin APIへのCLAP型スレッド注釈と文字列conformance走査 → v2境界で型/runtime checkとして再設計
- 新たなライセンス・統治宣言 → 現行MIT/Apache+READMEと重複
- 一般化された設定healing基盤、複数他アプリ互換プリセット → 実需要発生後

## 未確認として残すもの

- Slintが1.x期間の`.slint`言語互換をどこまで正式保証するか。`compat-1-2`の存在は確認したが、将来全1.xを受理するという包括保証ではない
- `slint-interpreter`で第三者リソースを読む場合のsandbox/許可import設計。`Compiler::set_file_loader`でロードを仲介できるが、v2配布形式と一緒に脅威モデルを決める必要がある
- Blender #68970、#146670、プリセット間伝播バグの元issue全文。robots制限で今回直接取得できず、元調査の詳細な数値・期間・healing不能の主張は未検証
- メディア再リンク/可搬性とGPUベンダ差は本レビューの対象外。別ラウンドが必要

## このレビューの使い方

元調査メモの所見をゲートや仕様へ採用する時は、本レビューの判定を併記する。元調査単独の「検証済み」表示や、出典URLのない歴史的詳細を完了条件の根拠に使わない。判断が割れた場合は、ユーザーデータまたは公開プラグイン契約へ不可逆に焼くかどうかで決める。焼かない選択が可能なら、v1では小さい方を選ぶ。
