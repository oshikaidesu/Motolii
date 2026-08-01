import { useId } from "react";
import "./feedback.css";

const placements = new Set(["inline", "target", "badge", "cursor"]);
const tones = new Set([
  "neutral",
  "valid",
  "warning",
  "error",
  "loading",
  "disabled",
]);
const recoveryKinds = new Set([
  "retry-with-changed-input",
  "requires-another-action",
  "unrecoverable",
]);
const contextRequiredTones = new Set(["warning", "error", "disabled"]);

function requireText(value, field) {
  if (typeof value !== "string" || value.trim() === "") {
    throw new TypeError(`Feedback ${field} must be a non-empty string`);
  }
}

export function validateFeedbackModel({
  placement,
  tone,
  label,
  reason,
  recovery,
}) {
  if (!placements.has(placement)) {
    throw new TypeError(`Feedback placement is unsupported: ${placement}`);
  }
  if (!tones.has(tone)) {
    throw new TypeError(`Feedback tone is unsupported: ${tone}`);
  }
  requireText(label, "label");

  const hasReason = reason !== undefined;
  const hasRecovery = recovery !== undefined;
  if (hasReason !== hasRecovery) {
    throw new TypeError(
      "Feedback reason and recovery must be supplied together",
    );
  }
  if (contextRequiredTones.has(tone) && !hasReason) {
    throw new TypeError(
      `Feedback ${tone} requires typed reason and recovery`,
    );
  }

  if (hasReason) {
    requireText(reason?.code, "reason.code");
    requireText(reason?.text, "reason.text");
    if (!recoveryKinds.has(recovery?.kind)) {
      throw new TypeError(
        `Feedback recovery kind is unsupported: ${recovery?.kind}`,
      );
    }
    requireText(recovery?.text, "recovery.text");
  }
}

export function Feedback({
  placement = "inline",
  tone = "neutral",
  label,
  reason,
  recovery,
  className,
  ...feedbackProps
}) {
  validateFeedbackModel({ placement, tone, label, reason, recovery });

  const descriptionId = useId();
  const hasContext = reason !== undefined;
  const role =
    tone === "error" ? "alert" : tone === "loading" ? "status" : "group";

  return (
    <div
      {...feedbackProps}
      className={["motolii-feedback", className].filter(Boolean).join(" ")}
      role={role}
      aria-label={label}
      aria-describedby={hasContext ? descriptionId : undefined}
      aria-busy={tone === "loading" ? "true" : undefined}
      tabIndex={hasContext ? 0 : undefined}
      data-feedback-placement={placement}
      data-feedback-tone={tone}
      data-feedback-reason={reason?.code}
      data-feedback-recovery={recovery?.kind}
    >
      <span className="motolii-feedback__marker" aria-hidden="true" />
      <span className="motolii-feedback__body">
        <span className="motolii-feedback__label">{label}</span>
        {hasContext && (
          <span id={descriptionId} className="motolii-feedback__context">
            <span className="motolii-feedback__reason">{reason.text}</span>
            <span className="motolii-feedback__recovery">
              {recovery.text}
            </span>
          </span>
        )}
      </span>
    </div>
  );
}
