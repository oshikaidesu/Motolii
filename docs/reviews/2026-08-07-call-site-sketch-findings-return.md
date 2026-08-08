# 仮コード照合で出たfinding — 前ownerへの返却

日付: 2026-08-07
状態: **finding / 本文書は修理許可ではない**

## 0. 扱い

`AGENTS.md`「findingは権限ではない」に従う。本文書は**報告と返却先の指定**だけを行う。
M3の接続作業に混ぜず、既存完了条件を阻むscope内原因としても扱わない。
修理の要否・順序・owner は各正本の owner が別途決める。

## 1. 発見の経緯（方法として記録する）

62項目のnode surveyは「その型・ファイルが実在するか」を見る。
仮コード（[器具境界決定](2026-08-07-provisional-call-site-sketch-instrument-decision.md)）は
「**その呼び出しが順に成立するか**」を見る。

両者を照合したところ、**surveyでは出ないclassの欠陥が2件出た**。
いずれも「node単位の存在確認」では見えず、**呼び出しを順に並べて初めて状態の非対称が見える**種類である。

利用者の言によれば、これはM3を「UIを作る工程」ではなく「**製品として成立させる工程**」として
見た結果、確認できた事例である。発見方法として本節を残す。

## 2. FINDING-1: `ReadOnlyNewer` が writable runtime へ入る疑い

### 観察

- `opened.open_mode` が `crates/motolii-ui/src/shell.rs:58-66` で**破棄されている**
- `ReadOnlyNewer` を弾く接続が存在しない
- `ProjectSession::acquire`（`crates/motolii-doc/src/journal/session.rs:88`）は lock 取得のみで、
  product向けの新規project作成経路が無い（`new_project` / `initialize_project` の grep は
  test helper `crates/motolii-ui/tests/cu109_session_backed_edit_entry.rs:72` のみ）

### 既決との関係

[P12-C1 文書ライフサイクル採択決定](2026-08-03-p12-c1-document-lifecycle-adoption-decision.md)は
「**ReadOnlyNewerは閲覧化**」と定めている。現行実装はこの admission を実行していない可能性がある。

### 返却先

P12 project lifecycle の owner。**M3の接続粒として修理しない。**

### 未確認

実際に未来versionのprojectを開いて書き込めるかは**未検証**である。
本findingはコード読解と仮コード照合による疑いであり、再現手順を伴う確認は別途必要。

## 3. FINDING-2: Undo が asset 登録を巻き戻さない疑い

### 観察

- `AssetTable` は `Command` enum のどの variant にも現れない
  （`crates/motolii-doc/src/command.rs` に対する `\.assets\.` grep が該当なし）
- したがって asset 登録は undo / redo の対象外である
- media を配置して Undo すると、`TrackItem` は戻るが **asset 登録は残る**

関連して:

- `UndoHistory::from_restored`（`crates/motolii-doc/src/undo.rs:214`）は**定義のみで呼び出し元ゼロ**
- `DocumentWriter::new`（`crates/motolii-doc/src/lib.rs:379`）は常に `revision=0` で構築する
- 再open後のUndo履歴復元が型としては在るが、誰も呼んでいない

### 既決との関係

絶対規律4（single writer）と journal / Undo の意味に関わる。
「Undo後にDocumentが元へ戻る」という前提が、asset 面で成立していない可能性がある。

### 返却先

M2 Document / journal / Undo の owner。**M3の接続粒として修理しない。**

### 未確認

asset 残留が実害（reopen時の不整合、export時の参照、容量）を生むかは未検証。
意図的な設計（asset は追記のみで参照されなければ無害）である可能性も排除していない。
**どちらであるかを owner が判定するまで、欠陥と断定しない。**

## 4. survey判定の格下げ2件（本文書ではなく地図側で反映済み）

仮コード照合で、node の**契約**が満たされていないことが判明した2件。

| node | survey 当初 | 訂正後 | 理由 |
|---|---|---|---|
| `R3-MENU` | `EXISTS_OLD_ONLY` | `PARTIAL` | 唯一のmenu実装（`app.rs:355-369`）は `LayoutAction` を直接生成し、**CommandId / CommandRegistry / InputRouter を経由しない**。node契約は「同じCommandIdへ投影」 |
| `R3-PROJECT-POLICY` | `EXISTS_WIRED` | `PARTIAL` | 開くことはできるが OpenMode admission が成立していない（FINDING-1） |

これは finding ではなく**状態訂正**であり、[成果駆動統合地図](../outcome-driven-integration-map.md)へ反映する。

## 5. 非目標

- 本文書を根拠にM3で修理を発注すること
- 疑いを確定した欠陥として外向きに扱うこと
- 未検証の実害を前提に設計を変えること
