# CU-0B05S Browser lifecycle再投影契約決定

- 日付: 2026-07-29
- 状態: **決定完了 / DONE**
- 対象: `CU-0B05`
- 前提: `CU-0B03`、`CU-0B04N`、`CU-0B04R`

## 1. 決定

BrowserのreloadまたはmacOS Web content process termination後は、同じWebViewの
Host session sequenceを巻き戻さない。古いopaque child WebViewを先にdropし、
同じBrowser一島を新しい`instance_epoch`で再生成する。

- `instance_epoch`はWebView document generationを識別するHost発行の単調値とする。
  initial mount、reload recovery、process termination recoveryごとに進める。
- Web側senderとRust `BrowserHostSession`のsequenceは新instanceごとに0から始める。
  同一instance内では既決どおり単調増加し、巻戻しを許さない。
- BrowserのRectangle source `(scope_ref, item_id)`はHost projection identityとして
  WebView再生成をまたいで維持する。instance epochをsource identityへ流用しない。
- reloadはlive instanceに対する`PageLoadEvent::Started`で検知する。新instanceの
  initial `Started`はreloadとみなさず、`Finished`でprojection readyとする。
- macOSのcontent process termination callbackはevent-loop wakeとrecovery要求だけを
  enqueueし、callback内でWebViewをdrop / build / reloadしない。
- 自動process recoveryはproduct Host lifetime中1回だけとする。二回目はBrowser islandを
  degradedへ止め、native Stage / Timeline、Document、journalを巻き込まない。timer、
  backoff、busy retryは追加しない。利用者向け再試行UIは本粒の非目標である。

再生成は第二の同時WebView islandを追加することではない。Product Hostはold Browserを
dropしてからnew Browserをbuildし、同時所有数を常に0または1に保つ。

## 2. 再投影するHost state

新instanceへ再投影するのは、同じHost所有Browser source、最新layout epoch / logical
bounds、requested focus targetである。initialization scriptは新instance用に
`BrowserHostSession::snapshot_json`から作り直すため、`evaluate_script`、runtime-mutable
script、新しいwire fieldを必要としない。

Document revision、primary selection、Undo/historyはBrowser wireへ足さない。Browser
lifecycle経路は`DocumentEditRuntime`を受け取らず、D2 queueへintentを配送しないことで
不変を保証する。Browserが表示しないrevision/selectionをsurface間sync用に複製しない。

resizeはWebView再生成を起こさず、`CU-0B04R`のlatest layout epochでboundsだけを更新する。
focusはOS focusの推測値でなくHost requested targetを維持し、new instanceがreadyになって
から`focus` / `focus_parent`を一回だけ適用する。

## 3. 負例

- 同じWebViewをreloadして静的initialization snapshotの古いsequenceを再利用する
- Rust sessionの`last_sequence`を0へ巻き戻す
- reload/crash後もold `instance_epoch`を受理する
- old/new WebViewを同時所有する
- snapshotを`evaluate_script`、generic invoke、raw JSON走査で注入する
- Browser wireへDocument revision、selection、Undo/historyを追加する
- callback内でWebViewまたはDocumentを変更する
- 二回目以降も自動再生成を続け、crash loopを作る
- DOM/CSS、React local semantic cache、公開plugin UI契約を変更する

## 4. STOP

既存exact Browser wireを変更する必要がある、WebViewをdrop-before-buildできない、
old instance callbackを新sessionから分離できない、Document / selection / historyを
Browserへ複製しないと合否を証明できない、または一回の自動再生成で通常製品routeへ
戻れない場合は実装を止める。sequence巻戻しや二重WebViewで迂回しない。

## 5. 次

次PRODUCT-ASSET `DO`は実装粒`CU-0B05`。private lifecycle state、wry callback、
drop-before-build、stable source / latest geometry / requested focus再投影、old epoch拒否、
Document/history変更0を自動試験と実Macで閉じる。
