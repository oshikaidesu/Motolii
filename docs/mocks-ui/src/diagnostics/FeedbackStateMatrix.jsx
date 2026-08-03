import { Feedback } from "../feedback/Feedback.jsx";
import "./feedback-state-matrix.css";

const retry = {
  kind: "retry-with-changed-input",
  text: "Layerを選択してから、もう一度実行してください。",
};
const anotherAction = {
  kind: "requires-another-action",
  text: "既存の接続を解除してから続行してください。",
};
const unrecoverable = {
  kind: "unrecoverable",
  text: "この項目は現在のprojectでは回復できません。",
};

const cases = [
  {
    id: "inline-neutral",
    placement: "inline",
    tone: "neutral",
    label: "変更内容を確認できます",
  },
  {
    id: "target-valid",
    placement: "target",
    tone: "valid",
    label: "Positionへ接続できます",
  },
  {
    id: "target-invalid",
    placement: "target",
    tone: "warning",
    label: "Positionへ接続できません",
    reason: {
      code: "target.type-mismatch",
      text: "PositionはLayer targetを要求します。",
    },
    recovery: retry,
  },
  {
    id: "disabled-action",
    placement: "inline",
    tone: "disabled",
    label: "選択項目を削除できません",
    reason: {
      code: "action.selection-required",
      text: "削除する項目が選択されていません。",
    },
    recovery: retry,
  },
  {
    id: "warning",
    placement: "inline",
    tone: "warning",
    label: "接続を変更する必要があります",
    reason: {
      code: "connection.definition-in-use",
      text: "Effect Definitionは3件のUseから参照されています。",
    },
    recovery: anotherAction,
  },
  {
    id: "error-unrecoverable",
    placement: "inline",
    tone: "error",
    label: "操作を完了できません",
    reason: {
      code: "operation.unrecoverable",
      text: "利用可能な回復操作がありません。",
    },
    recovery: unrecoverable,
  },
  {
    id: "loading",
    placement: "inline",
    tone: "loading",
    label: "Previewを準備しています",
  },
  {
    id: "semantic-badge",
    placement: "badge",
    tone: "valid",
    label: "移動経路 → 円形パス",
  },
  {
    id: "cursor-context",
    placement: "cursor",
    tone: "warning",
    label: "Audio Trackは対象外です",
    reason: {
      code: "cursor.target-incompatible",
      text: "現在の操作はLayer targetだけを受け入れます。",
    },
    recovery: retry,
  },
];

export function FeedbackStateMatrix() {
  return (
    <main
      className="feedback-state-matrix"
      aria-label="Common feedback state matrix"
    >
      <header className="feedback-state-matrix__header">
        <p>CU-203M · development contract</p>
        <h1>Common feedback states</h1>
      </header>
      <section
        className="feedback-state-matrix__grid"
        aria-label="Feedback fixtures"
      >
        {cases.map(({ id, ...model }) => (
          <article
            className="feedback-state-matrix__case"
            data-feedback-case={id}
            key={id}
          >
            <code>{id}</code>
            <Feedback {...model} />
          </article>
        ))}
      </section>
    </main>
  );
}
