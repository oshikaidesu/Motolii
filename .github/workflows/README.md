# Stage 5 CIの範囲

`ledger-fences.yml`はmainへのpushまたは手動実行で、現行workspaceの入口と文書整合を検査する。旧app/nextのRustテストや個人checkoutは呼ばない。

`ui-gallery-pages.yml`は`motolii/ui`のGalleryをPRでformat/analyze/Web buildし、mainへの反映時にGitHub Pagesへ公開する。Galleryは製品のWeb版ではなく、native bridgeを起動しないUI確認入口。ローカルの反復はFlutterのhot reloadまたはWidget Previewerを使う。

これらは構成の誤誘導やUI/native境界の漏れに気づくための検査であり、macOS実機・GPU共有・作品書き出しの検収ではない。製品検証は[Stage 5](../../docs/stage5/README.md)のローカル手順で行う。

workflowのrequired checkやbranch protection、マージ権限の設定は変更しない。
