# U0e-2 発注失敗の原因と機械ガード

日付: 2026-07-22  
状態: **決定 / GR-D1〜R3実装・検収済み / U0e-2再開可**
対象: CU-0A02 / U0e-2 の却下差分、React source asset を含む以後の発注、
`scripts/delegate-cursor-supervised.sh`

## 1. 結論

U0e-2 の発注書には、既存 `candidates/surfaces` の表示合成、三層 fixture の投影、
diagnostic route の隔離、部分更新禁止が書かれていた。それでも差分は、既存 React 資産を使わない
5画面の縮約 stub、戻り値を捨てる fixture load、生色値を持つ専用 CSS、通常 catalog への混入、
非原子的な captures / manifest 交換になった。

主因は散文不足ではない。次の三つが同時に成立したことにある。

1. `CODEX PRECHECK: APPROVED` が marker の存在だけで、Codex が選んだ正本 ref、対象 worktree の base、粒の
   `DO / WAIT`、authority file の同一性を機械照合していなかった。
2. 発注書の再利用・三層投影・route 隔離・原子性が、実装途中で失敗する因果 oracle になって
   いなかった。semantic ID と画像集合だけなら stub でも満たせた。
3. 検収は実装後だけにあり、300秒 timeout と一時 directory の無条件削除により、timeout 時の
   証跡と再開点を失った。

Claude Fable 5 の read-only 原因レビューと発注書原文を含む再監査を行った。初回レビューが
発注書原文を読めなかった部分は採用せず、上記は Codex が実ファイルと却下差分で再確認した事実だけに
限定する。

## 2. 即時停止線

次のガードが実装・負例検証されるまで、却下済み U0e-2 差分を修理発注・採用せず、React source
asset を含む次粒も実装発注しない。

- 対象 worktree の `HEAD`、発注書の `BASE_REF` が指すcommit、`BASE_SHA`、authority file の
  path + SHA-256 が一致しない
- `BASE_REF` / `BASE_SHA` で固定した対象 worktree 内の粒度台帳で対象粒が `DO` でない、または
  依存粒が `DONE` でない
- authority file が対象 worktree 内に存在せず、main の絶対 path だけを参照する
- React mandatory 8 labels の欠落、順序逆転、固定 source SHA/path の不一致
- 変更許可外 path が tracked / untracked のどちらかに一つでもある

## 3. 採択する機械ガード

### GR-D1: dispatch gate

通常発注の正規入口である `scripts/delegate-cursor-supervised.sh` は、発注実行前に発注書と正本を
機械照合する。Codex CLI または Cursor Grok が利用不能で Claude fallback を明示した場合も、代替 runner は同等の gate を
通過しなければならない。

- `BASE_REF` はCodexが選んだ完全な `refs/heads/...`、`BASE_SHA` はそのcommitと対象 worktree
  `HEAD`の両方に完全一致
- `AUTHORITY path sha256` の全件が対象 worktree 内に存在し、byte hash が一致
- `GRAIN` と明示 `DEPENDENCY` を、同じbaseに含まれる粒度化台帳から引き、状態 `DO` と依存
  `DONE` を確認
- React対象なら mandatory 8 labels を指定順で確認
- 不一致は実装担当を起動せず fail closed。実装者用の抑制・例外・環境変数 bypass は作らない

期待する代表エラー:

```text
ORDER-GATE NG: worktree HEAD != BASE_SHA
ORDER-GATE NG: BASE_REF does not resolve to BASE_SHA
ORDER-GATE NG: authority hash mismatch: <path>
ORDER-GATE NG: CU-0A02 is WAIT; dispatch is forbidden
ORDER-GATE NG: React guard label missing or out of order: <label>
```

### GR-D2: scope closure と証跡永続

実装後、検収前に `git status --porcelain` を含む全変更を変更許可閉集合へ照合する。許可外 path は
検収へ進めない。order / implementation / inspection / stderr / timeout / 前後 status / diff を発注書に
対応する evidence directory へ残す。検収 timeout は実装・prepare と分離し、timeout 後も同じ段階から
再開できるようにする。検収者が worktree を変更した場合は verdict を無効化する。

### GR-R1: React provenance と route

- reference screen は指定された既存 candidate / surface / primitive の export を import し、
  provenance manifest の closure と一致する
- product/mock の二重 copy、legacy/archive runtime import、reference leaf の自己登録で再利用を偽装しない
- `#catalog` は candidate route だけを列挙し、reference / diagnostic / archive は別入口とする
- screenshot 類似や semantic ID の貼付を source reuse の証明にしない

### GR-R2: fixture 因果と token

三層 fixture の値を一時 copy で一つずつ変え、該当 normal capture が変化しなければ失敗させる。
これは画像類似ではなく `fixture -> projection` の因果 oracle である。戻り値を捨てる
`loadReferenceFixtures(...)` と reference tree 内の生 hex / `rgb()` / inline color も拒否する。

GR-R1/R2の機械境界は`docs/mocks-ui/scripts/reference-guard.mjs`、負例は
`docs/mocks-ui/guard-tests/reference-guard.test.mjs`を正本とする。manifestのpath/export/SHA-256を
Babel AST / PostCSSで実import closureへ照合し、central registryの`catalogKind`を
`candidate / reference / diagnostic / archive`へ閉じる。三層因果は製品rendererを複製せず、
`verifyFixtureCausality`はmanifestのtest evidenceとしてpath/export/SHA-256を固定したnormal capture
rendererだけをloadする。原fixtureを不変にした一時copyの同じ三pathへbaselineと各一層変異を二つの
異なる順序で再生し、同じ状態のcapture決定性と`document / scenes / tokens`それぞれの因果を判定する。
U0e-2は実5画面とrendererをこの境界へ接続するが、guard自身へscreen固有stubや色値を追加しない。

### GR-R3: 原子性と負例行列

captures と manifest を同一世代として交換し、各 I/O 境界への失敗注入で旧世代または新世代の
どちらか一方だけが残ることを検証する。以下を独立負例として固定する。

- manifest source SHA と現 fixture の不一致
- 実行時 `browser.version()` と固定 Chromium version の不一致
- 派生画像が normal と同一
- schema 外 / 壊れた Document、未知 screen、非有限 token
- reference / diagnostic の catalog 混入
- fixture load 戻り値破棄、既存 source import 0、生色値
- 許可外 untracked file、検収者による1 byte書込み、各交換段階の失敗注入

GR-R3の機械境界は`docs/mocks-ui/scripts/reference-generation.mjs`、負例は
`docs/mocks-ui/guard-tests/reference-generation.test.mjs`を正本とする。captureとmanifestは一意な
staging generationで全fileとdirectoryをsyncしてからimmutable generationへrenameし、readerが参照する
`CURRENT`一ファイルだけを原子的に交換する。manifestはbrowser version、source SHA、screen/variant順、
全PNGのbyte SHAを閉じ、PNGをdecodeした寸法+RGBAでもnormal/derivedとscreen間normalを比較する。
17個の全mutation checkpointで失敗注入し、`CURRENT`交換前は旧世代、交換後は新世代の全captureとmanifestが
一致することを固定する。同名generationの上書き、generation rootの余分file、壊PNG、余分captureも拒否する。

負例行列の機械証跡は次の分担とする。別testへ分かれていても`npm run test:reference-guard`が全件を一括実行する。

| 負例 | 機械証跡 |
|---|---|
| source SHA / browser version / derived同一 / unknown screen / schema外 | `reference-generation.test.mjs`のclosed generation負例 |
| 壊れたfixture root / 非有限probe | `reference-guard.test.mjs`のmalformed/non-finite負例 |
| route混入 / loader戻り値破棄 / source import 0 / 生色 | `reference-guard.test.mjs`のroute/provenance/fixture/raw-color負例 |
| 許可外untracked / 検収者1 byte書込み | `scripts/test-delegate-cursor-supervised.sh`のGR-D2 scope/fingerprint負例 |
| 各交換段階の失敗 | `reference-generation.test.mjs`の全checkpoint old/new世代負例 |

## 4. Fable案から採用しなかった短絡

再監査は「派生25枚が normal から再計算されるため dataflow guard は不要」としたが、これは
`normal -> derived` しか証明せず、今回破れた `三層 fixture -> normal` を証明しないため採用しない。

同様に、現行 generator は一時 directory で生成した後に live captures を `rm` し、captures と
manifest を別々に `rename` する。rename 間の失敗で世代混在が起きるため、原子性 guard も維持する。
`TEST_VECTORS` は色変換の既知値だけであり、壊した Document、source SHA、route、fixture投影の
負例行列の代わりにはならない。

## 5. 最小実装順

1. **GR-D1**: dispatch gate。古い正本・WAIT粒を実装担当へ渡さない。
2. **GR-D2**: scope closure、検収証跡永続、timeout分離。
3. **GR-R1/R2**: React provenance、route隔離、fixture因果、生token拒否。
4. **GR-R3**: atomic swap、失敗注入、負例行列。
5. 全ガード通過後にだけ、却下済みU0e-2を既存React資産の直接合成として修理発注する。

各粒は一つの契約境界だけを変更する。Document、plugin契約、製品公開API、golden threshold、既存
visual期待値は変更しない。正当な差分がガードへ落ちても実装者がallowlistや抑制を追加せず、
`ORDER: STOP`でCodexへ戻し、ガード改訂を独立変更として反対側レビューする。
