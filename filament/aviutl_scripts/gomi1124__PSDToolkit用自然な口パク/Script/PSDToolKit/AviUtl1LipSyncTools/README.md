# PSDToolKit Natural LipSync

AviUtl2版PSDToolKit2の立ち絵を、音声の強弱から自然に口パクさせる非公式ランタイムです。AviUtl1版PSDToolKitの音声解析を土台にしていますが、完全再現ではなく、さまざまな声質や口形状で自然に見える動きを目指しています。公式の`PSDToolKit.lua`は変更しません。

## 特徴

- 256サンプルのHamming窓、Ooura RDFT、音量判定を使って、発話中の口形状をネイティブモジュールで解析します。
- 16-bit PCM WAVはAviUtl2のメディアキャッシュを介さず読み取り、安定した24 kHzの解析音声へ変換します。
- 音量がしきい値以上なら1段階開き、未満なら1段階閉じるため、半開きなどの中間口形状を開閉の途中だけに表示します。
- 複数の口形状欄に同じレイヤーが設定されている場合は連続する重複を除き、同じ画像で遷移が停滞するのを防ぎます。
- `.anm2` / `.obj2`を複数まとめてドラッグ＆ドロップ変換できます。

本ツールの通常モードは、音声の強弱に合わせて閉じ口から開き口までを1フレームずつ移動します。16-bit PCM WAV向けの旧定義互換モジュールも`LipSyncAviUtl1Legacy.lua`として同梱していますが、通常モードでは同じ画像が連続する口形状欄を自動的に整理します。

## 必要環境

- Windows 10 / 11 x64
- AviUtl2
- PSDToolKit2

[PSDToolKit2](https://github.com/oov/aviutl2_psdtoolkit2)は先に導入してください。本ツールだけではPSDを表示できません。

## インストール

### AviUtl2カタログまたはパッケージファイル

Releasesから`PSDToolKit_NaturalLipSync_v*.au2pkg.zip`を取得します。AviUtl2カタログではパッケージファイルとして読み込めます。手動の場合はZIPの`Script`フォルダを`C:\ProgramData\aviutl2`へ展開してください。

インストール後は次の3ファイルが同じ場所に置かれます。

```text
C:\ProgramData\aviutl2\Script\PSDToolKit\LipSyncAviUtl1.lua
C:\ProgramData\aviutl2\Script\PSDToolKit\LipSyncAviUtl1Legacy.lua
C:\ProgramData\aviutl2\Script\PSDToolKit\LipSyncAviUtl1.mod2
```

### 立ち絵定義の変換

1. AviUtl2を終了します。
2. `C:\ProgramData\aviutl2\Script\PSDToolKit\AviUtl1LipSyncTools`を開きます。
3. 変換したい`.anm2` / `.obj2`、または定義を含むフォルダを`PSDToolKit口パク変換へドロップ.cmd`へドロップします。
4. 完了後にAviUtl2を起動します。

複数ファイルと複数フォルダを同時にドロップできます。片方だけを指定した場合も、同名の`.anm2` / `.obj2`を対で変換します。元ファイルはその場で更新され、変換前の内容は日時付きフォルダへバックアップされます。

既存エイリアスが変換済みの定義名を参照している場合、エイリアス自体を作り直す必要はありません。AviUtl2の再起動後から新しい口パク処理が使われます。

## 2.0.0の互換性

汎用ツールであることを明確にするため、厳密な旧状態判定モジュールを`LipSyncAviUtl1Legacy.lua`へ改名しました。旧名の`LipSyncAviUtl1MtULegacy.lua`は配布せず、旧名との互換性も提供しません。

旧名を直接`require`している独自定義は、新しいモジュール名へ変更するか、元のPSDToolKit定義から通常モードへ再変換してください。変換ツールが生成した通常モードの定義と、`LipSyncAviUtl1`を参照している既存エイリアスには影響しません。

公開名は2.0.0から`PSDToolKit Natural LipSync`へ変更しました。既存定義と更新導入を維持するため、ランタイムの`LipSyncAviUtl1`というファイル名、カタログID、`AviUtl1LipSyncTools`という導入先フォルダ名は内部互換名として残しています。

## 対応範囲

16-bit PCM WAVのモノラル、およびステレオのトラック0を主な高精度解析対象としています。圧縮音声や24 kHz未満の音声はAviUtl2のデコーダーを経由するため、解析結果が異なる場合があります。

口形状はPSDToolKit定義に登録されている順番を使用します。閉じ・開きの2形状だけでも、半開きなどを含む3から5形状でも動作します。

## ビルド

Visual Studio 2022の「C++によるデスクトップ開発」とWindows SDKが必要です。

```powershell
.\Build-LipSyncAviUtl1.ps1
.\New-ReleasePackage.ps1 -SkipBuild
```

`Build-LipSyncAviUtl1.ps1`はネイティブ単体テストも実行します。配布物は`dist`へ生成されます。

## ライセンス

本プロジェクトは[MIT License](./LICENSE)で公開しています。AviUtl2 SDKとOoura FFTのライセンス・著作権表示は[native/THIRD_PARTY_NOTICES.md](./native/THIRD_PARTY_NOTICES.md)を参照してください。

PSDToolKit2は本リポジトリに含みません。PSDToolKit2本体のライセンスと配布条件は公式リポジトリに従ってください。

## AIの利用について

READMEを含む本リポジトリのコード、文書、配布素材は、AI（OpenAI Codex）を使用して生成・編集されています。

## 謝辞

- AviUtl / AviUtl2: ＫＥＮくん
- PSDToolKit / PSDToolKit2: おおぶ（oov）さん
- General Purpose FFT: Takuya Oouraさん

本プロジェクトは上記各プロジェクトの公式配布物ではありません。
