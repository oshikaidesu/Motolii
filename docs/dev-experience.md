# 開発体験(DX): ホットリロードと反復速度

作成日: 2026-07-13
更新日: 2026-07-25
ステータス: **設計ノート**(hot reloadの段階とcrash recoveryへの接続を固定するが、runtime／ABI／sandboxの公開契約は変更しない。先例表は未カウンターレビューの仮説 — [reviews/README.md](reviews/README.md)規律3に従い、v2口の解凍時に反対側レビューを通してから設計根拠化する)
関連: [plugin-authoring.md](plugin-authoring.md)§3-3(純関数契約)、[plugin-resources.md](plugin-resources.md) D1(ホスト所有PipelineCache)、[Controlled Microkernel決定](reviews/2026-07-25-controlled-microkernel-host-module-parallelism-decision.md#6-pluginという語と信頼境界の分離)、[backlog.md](backlog.md) INF-6/INF-8/V2-1/V2-2、[ae-pain-points.md](ae-pain-points.md)

## 1. 問題(なぜこの文書が要るか)

AEのプラグイン開発は「1行直すたびにAE再起動+プロジェクト開き直し」だった。プラグイン作者(人間・LLMとも)の反復速度は採用の入口であり、コンセプトの「LLMがプラグインを量産する」ワークフローでも人間が目視確認する局面では同じ摩擦を踏む。

AEがホットリロードできない構造要因は3つ:

1. dylibを起動時に1回ロードして握りっぱなし(差し替えの口がない)
2. プラグインが内部状態を所有できる(差し替え時に何を捨ててよいかホストが知らない)
3. リロード後にどのキャッシュが無効かホストが判定できない

**Motoliiは§3-3の純関数契約と「キャッシュはホスト専権」(M4キャッシュキー完全性)で②③を既に排除している。** 出力は`t + 入力 + params`だけで決まるため、プラグイン差し替えの意味論は「該当プラグインidが寄与するキャッシュキーを無効化して再評価」で閉じる。状態移行問題が存在しない。ホットリロードのために設計した規律ではないが、副産物としてリロード可能性が手に入っている。残るのは①(差し替えの口)だけで、これはレベル別に安い順で解ける。

## 2. 先人の解決策(仮説台帳。未カウンターレビュー)

| 困りごと | 先人(仮説) | 機構 |
|---|---|---|
| シェーダ調整のたびに再ビルド | Bevy(asset hot-reload)、TouchDesigner、シェーダライブコーディング環境一般 | シェーダソースをファイル監視し、変更時にパイプラインだけ再コンパイル。アプリ本体は生きたまま |
| ネイティブコードの差し替え | Zed / Lapce / Extism(WASMプラグイン) | ネイティブdylibのABI地獄を避け、wasmモジュールをランタイムでスワップ。エディタ再起動なし |
| Rust dylibのホットリロード | hot-lib-reloader系 / Unreal Live Coding | Rustは型レイアウト・ABIが不安定で、実用化には(Unrealが払ったような)巨額の継続投資が要る。ZedらがWASMへ行ったのはこの回避が動機のひとつ(要一次資料確認) |
| そもそも再起動を苦にしない | Blender(スクリプトreload+高速起動)ほか | 起動が数秒+セッション完全復元なら、再起動は反復手段として成立する |
| エディタUI自体の反復 | React component HMR + Host session再投影 | product-owned React componentはdev bundleのHMR、native workspaceと製品結合は高速再起動＋INF-6 session復元、headless interaction test、固定reference screenで反復する。HMR成立をproduction reload契約やplugin UI公開許可へ外挿しない |

## 3. はしご(欲しい反復速度に対して最も安いレベルを選ぶ)

| レベル | 手段 | 時期 | 根拠・依存 |
|---|---|---|---|
| 0 | **WGSLホットリロード(開発ビルド限定)**。シェーダファイル監視→ホストのPipelineCacheへ再コンパイル要求→該当ノードのキャッシュ無効化 | v1(INF-8) | パイプラインはホスト所有([plugin-resources.md](plugin-resources.md) D1)なので、プラグイン側の協力なしにホスト単独で差し替えられる。プラグイン反復の大半はシェーダ調整 |
| 1 | **再起動を安くする**: 高速起動+ジャーナル復元でセッションが完全に戻る | v1(INF-6の副産物) | INF-6「kill→再起動→復元」の完了条件がそのままDX要件を兼ねる。Rustロジック変更(静的リンク)はこの経路で回す |
| 2 | **WASMモジュールの実行時スワップ候補** | v2(V2-1/V2-2の方式spike後) | 純関数契約により差し替え=キャッシュ無効化で意味論を閉じやすい。WASMを採る場合は「配布sandbox」に加えてDXも比較軸になるが、runtime方式は未決 |
| ✗ | Rust dylibのホットリロード | **恒久にやらない** | ABI/型レイアウト不安定で博打。レベル1(安い再起動)とレベル2(WASM)で同じ欲求を満たす |

### 3.1 hot reloadとcrash recoveryを同じ交換路へ畳む

hot reloadは「開発中だから信頼して同一processへcodeを差し込む」機能にしない。編集による新artifact、
compile失敗、WASM trap、panic、worker process停止は原因こそ違うが、Hostから見れば
**現在のruntime instanceを無効化し、検証済みinstanceへ交換するlifecycle event**である。

```text
source／artifact change または runtime failure
  → 旧generationへの新規投入を停止
  → in-flight結果をgenerationで破棄
  → capability／version／resource budgetを再検証
  → sandbox／worker instanceを生成
  → 該当plugin idが寄与するcacheを無効化
  → Host所有recipeと同じrevisionから再評価
```

HostはDocument、recipe、identity、revision、cache key、resource admissionを保持する。plugin runtimeへ
編集正本や復旧に必要な隠れstateを持たせないため、開発時の交換と障害後の再起動を同じprimitiveで
実装できる。compile error時に直前の正常artifactを維持するpolicyと、crash時に該当instanceを停止して
欠落診断を出すpolicyは分けても、交換路そのものを二重実装しない。

この共通化はfirst-party codeを信用する根拠ではない。公開plugin境界を通るcodeは供給元を問わず
非信頼であり、将来は同じsandbox／worker境界内で交換する。Controlled Microkernelとadmitted Host
capability moduleだけがTCBである。現行v1のstatic first-party pluginにはruntime交換口もprocess隔離も
まだ無いため、現時点のRust変更は再build＋製品再起動を正規経路とする。journal／session完全復元を
伴うレベル1はINF-6完了後の目標であり、現行実装済みとして扱わない。

GPU pluginのprocess外worker交換は長期候補だが、process間texture共有、OS handle、device-lost、
in-flight GPU workの取消は未決である。Bitwig型の段階隔離は比較候補に留め、先例の名称からMotoliiの
公開protocolや安全性を逆算しない。

### 3.2 Core／Host moduleでhot reloadが輝く範囲

Controlled Microkernel自身のidentity、revision、atomic commit、authority多重度を実行中に交換しない。
ここをreload対象にすると、交換を裁定する不動点が消える。

一方、admitted Host capability moduleの**具体実装**は、authority stateをkernel／Host所有のまま保ち、
旧generationの停止、in-flight結果の破棄、候補実装のconformance、atomic activate、失敗時rollbackを
一つの交換transactionにできるなら、開発時hot reloadの候補になる。Document reducer、journal、
Undo等のauthorityをmodule private stateへ移してreload可能に見せるのは禁止する。

交換後の出力が変わり得るproviderでは、cache keyへmodule実装世代／artifact identityを寄与させるか、
activate時に影響範囲を全無効化する。plugin IDだけの無効化をHost module交換へ流用しない。この意味と
負例はM4 cache key完全性の解凍時に締結し、本書から永続version fieldを先行追加しない。

これはproductionのdynamic module ABIを決定しない。現行Rust moduleの正規経路は引き続き
製品processの高速再起動＋journal／session再投影である。将来、二実装とconformanceが成立したseatだけを
開発時provider swapへ上げる。つまりCore側で得る価値は、kernelを差し替えることではなく、
**kernelを固定したまま周辺実装を捨てて再構築できること**にある。

## 4. 実装の注意(レベル0)

- **開発ビルド限定の機能とし、製品経路・ゴールデンテストに影響させない**。ファイル監視はdevフラグ配下、CIでは無効
- キャッシュ整合: パイプラインキーはWGSLソース(またはその内容ハッシュ)を含むdescriptorなので、ソース変更=別キー=自然に再コンパイルされる。**古いソースで描いたフレームキャッシュの無効化**だけ明示処理が要る(該当プラグインidの寄与するキーを落とす。M4キャッシュキー完全性の枠内)
- コンパイルエラー時は直前の正常パイプラインで描き続け、エラーをUI/ログに出す(黒画面でループを止めない)
- LLM量産ワークフローとの関係: エージェントの検証ループは既にヘッドレス(ゴールデン+purity)で回る。レベル0/1は**人間が目視したい局面**のためのもの

## 5. 決めないこと

- レベル2(WASMスワップ)の具体設計はv2口の解凍時。その際は§2の先例表を一次資料で確認し反対側レビューを通す(規律1・2・6)
- エディタUIの実行時ホットリロードはv1契約にしない。製品React componentのHMR／production bundle再読込、session再投影、構文エラーからの復旧をINF-8の非ブロッキング計測へ置く。既存egui shellの再buildはbaseline計測に限り、新しい製品開発経路にしない
- v1 plugin UIはHost自動生成panelだけなので、UIの反復はHost側の通常開発経路へ畳む。plugin所有のegui/native codeとwgpu自由UIは[M3着手前決定](reviews/2026-07-16-m3-preflight-decisions.md)どおり公開しない。将来の宣言語彙は型ごとの解凍判断に従う
