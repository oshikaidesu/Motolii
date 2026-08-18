# icedの出荷実績・維持体制の外部調査

作成日: 2026-08-18

状態: **調査**(観察。決定を含まない)

対象: [icedホスト移行裁定](2026-08-18-iced-host-migration-decision.md)が引き受けたコスト(fork保有・master追随・検証基盤の空白)が、外の世界でどう扱われているかの外部調査。fork運用の中身は[iced fork seam台帳](2026-08-18-iced-fork-seam-ledger.md)が正本。

## 1. 出荷実績

- **COSMIC / Pop!_OS 24.04 LTS**: 安定版が2025-12-11に出荷され半年運用中。ただしpop-os fork=libcosmic上での運用であり、上流へのrebase課題が残る(libcosmic #1089)。
- **Kraken Desktop**: 2024-10-31に「Icedでゼロから構築」と明言してリリース、商用運用中。
- **Sniffnet**: v1.5.0(2026-04-14)でiced 0.14へ移行済み。
- **Halloy**: ほぼ月次リリースで追従継続。
- 旧**Cryptowatch Desktop**は2023年に事業廃止だが、後継も同じicedを使っており離脱例ではない。

「icedを捨てた」有名プロダクトのポストモーテムは**発見できなかった**。観測されたパターンは離脱ではなく「**forkして留まる**」(System76・Kraken・個人プロジェクト)である。

## 2. リリースと1.0

- 最新リリースは0.14.0(2025-12-07)。1.0未達。
- 作者hecrj本人が「1.0はまだ遠い」と明言(SE Radio 713、2026-03-25)。
- minor間隔は7〜15ヶ月で、毎回破壊的変更。
- masterは2026-08-16時点まで高活性。
- icedはwinitをvendoringしている(依存の下層も自前で持つ体質)。

## 3. AccessKit

- upstream未マージ(issue #552は2020年から)。
- 外部の完成度の高いPR #3281(2026-03-14)を、hecrjが**同日クローズ**した — 「Thanks! But I'll work on this myself.」。**fork分が本家に吸われる期待は持てない**ことの直接証拠。
- 0.15の主題はアクセシビリティと予告されている。

## 4. IME / 日本語入力

- PR #2777(kenz-gelsoft)が2025-02にマージされ0.14で出荷=**史上初のIME対応**。ただしover-the-spotのみで、macOS検証。
- WindowsでIME変換ポップアップが画面右下に飛ぶissue #3189(2026-01-09)は放置中。
- on-the-spot未実装。
- 日本語ユーザー向けエディタとしては、IMEエッジケースを**自forkで直す前提**の見積もりが必要。

## 5. focus / キーボードナビゲーション

- 集中focus管理が未整備であることを作者自身が0.15の課題と認めている。
- canvasにフォーカス概念なし([iced再評価調査](2026-08-18-iced-reentry-survey.md)の仮タイムラインspikeで実測した既知穴と整合)。

## 6. ガバナンス

- コミット分布はhecrj 5,557に対し2位107 — **実質バス係数1**。
- ただし本人はフルタイムで、QAも雇用されている。

## 7. 評定

- 移行裁定の迂回を要求する発見は**ゼロ**。
- 不安点は全て「**fork内で自分が持つコードが増える**」という種類に収まり、seam台帳+rev pin運用([iced fork seam台帳](2026-08-18-iced-fork-seam-ledger.md))が既に引き受けた形の延長である。
- 小fork戦略はicedエコシステムの標準的成功パターン(System76・Kraken)だが、**「小forkは小のままでは済まない」** — 特にWindows IME(#3189)とfocus系は、上流が直すまで自forkに実装が積まれる公算が高い。

## 8. 再観測トリガー

- iced 0.15リリース(focus/a11yが主題) — seam・自前実装のどれが上流で解消されるか
- issue #3189(Windows IMEポップアップ位置)の動き
- 1.0工程表の出現

## 9. 出典

- [SE Radio 713: Hector Ramon Jimenez on Building a GUI Library in Rust](https://se-radio.net/2026/03/se-radio-713-hector-ramon-jimenez-on-building-a-gui-library-in-rust/)
- [Accessibility · iced-rs/iced#552](https://github.com/iced-rs/iced/issues/552)
- [AccessKit PR · iced-rs/iced#3281](https://github.com/iced-rs/iced/pull/3281)
- [IME support · iced-rs/iced#2777](https://github.com/iced-rs/iced/pull/2777)
- [Windows IME candidate window position · iced-rs/iced#3189](https://github.com/iced-rs/iced/issues/3189)
- [libcosmic rebase tracking · pop-os/libcosmic#1089](https://github.com/pop-os/libcosmic/issues/1089)
