import "./easing-trigger-candidate.css";

export function EasingTriggerCandidate({ activeInterval, pressed, onOpen }) {
  return (
    <button
      id="interval-easing"
      className={`interval-easing${activeInterval ? " on" : ""}`}
      aria-label={
        activeInterval
          ? `${activeInterval.objectName} · ${activeInterval.channel}のInterval Easing Editorを開く`
          : "key間へ移動するとInterval Easing Editorを開けます"
      }
      aria-pressed={pressed ? "true" : "false"}
      aria-controls="easing-panel"
      data-info="Easing Graph|key間にいる時だけ開けます|"
      disabled={!activeInterval}
      onClick={(event) => {
        if (!activeInterval || typeof onOpen !== "function") return;
        const rect = event.currentTarget.getBoundingClientRect();
        onOpen({ x: rect.x, y: rect.y, width: rect.width, height: rect.height });
      }}
    >
      <svg viewBox="0 0 20 14" aria-hidden="true">
        <path d="M2 12 C7 12 8 2 18 2" />
        <circle cx="2" cy="12" r="1.6" />
        <circle cx="18" cy="2" r="1.6" />
      </svg>
    </button>
  );
}
