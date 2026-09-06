# Stage 5 CIの範囲

`ledger-fences.yml`はmainへのpushまたは手動実行で、現行workspaceの入口と文書整合を検査する。旧app/nextのRustテストや個人checkoutは呼ばない。

これは構成の誤誘導に気づくための検査であり、macOS実機・GPU共有・Flutter UI・作品書き出しの検収ではない。製品検証は[Stage 5](../../docs/stage5/README.md)のローカル手順で行う。

今回の更新はworkflow内容の修正だけであり、required checkやbranch protection、マージ権限の設定は変更しない。
