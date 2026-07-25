import "./key-tools-candidate.css";

const KEY_SCOPE_OPTIONS = [
  ["object", "▤", "Object別"],
  ["channel", "⋮", "Channel別"],
  ["global", "◎", "全選択"],
];

const KEY_SECTIONS = [
  ["align", "┆◆┆", "Align"],
  ["stagger", "◆⋰◆", "Stagger"],
  ["stretch", "←◆→", "Stretch"],
];

const LAYER_SECTIONS = [
  ["align", "┆▤┆", "Layer Align"],
  ["stagger", "▤⋰▤", "Layer Stagger"],
  ["shift", "←▤→", "Layer Shift"],
];

export function KeyToolsCandidate({
  open,
  onOpen,
  onClose,
  mode,
  onModeChange,
  keyCount,
  layerCount,
  scope,
  onScopeChange,
  keySection,
  onKeySectionChange,
  layerSection,
  onLayerSectionChange,
  onKeyOperation,
  onLayerOperation,
}) {
  if (!open) {
    return (
      <button
        type="button"
        className="candidate-key-tools-open"
        aria-label="Key Toolsを開く"
        onClick={onOpen}
      >
        ◆
      </button>
    );
  }

  return (
    <aside className="candidate-key-tools" aria-label="Key Tools">
      <div className="candidate-key-mode">
        <button
          type="button"
          aria-pressed={mode === "keys"}
          onClick={() => onModeChange("keys")}
        >
          KEYS
        </button>
        <button
          type="button"
          aria-pressed={mode === "layers"}
          onClick={() => onModeChange("layers")}
        >
          LAYERS
        </button>
        <button
          type="button"
          aria-label="Key Toolsを閉じる"
          title="閉じる"
          onClick={onClose}
        >
          ×
        </button>
      </div>
      {mode === "keys" ? (
        <>
          <div className="candidate-key-tools-head">
            <b>◆ {keyCount}</b>
            <div className="candidate-key-scope" aria-label="適用単位">
              {KEY_SCOPE_OPTIONS.map(([value, icon, label]) => (
                <button
                  type="button"
                  aria-label={label}
                  aria-pressed={scope === value}
                  key={value}
                  title={label}
                  onClick={() => onScopeChange(value)}
                >
                  {icon}
                </button>
              ))}
            </div>
          </div>
          <div className="candidate-key-sections">
            {KEY_SECTIONS.map(([section, icon, label]) => (
              <button
                type="button"
                aria-label={label}
                aria-expanded={keySection === section}
                key={section}
                title={label}
                onClick={() =>
                  onKeySectionChange(keySection === section ? null : section)
                }
              >
                {icon}
              </button>
            ))}
          </div>
          <div className="candidate-key-actions">
            {keySection === "align" ? (
              <>
                <small>ALIGN</small>
                <button
                  type="button"
                  aria-label="開始へ整列"
                  title="開始へ整列"
                  onClick={() => onKeyOperation("align-start")}
                >
                  │◆
                </button>
                <button
                  type="button"
                  aria-label="Playheadへ整列"
                  title="Playheadへ整列"
                  onClick={() => onKeyOperation("align-playhead")}
                >
                  ◆┆◆
                </button>
                <button
                  type="button"
                  aria-label="終了へ整列"
                  title="終了へ整列"
                  onClick={() => onKeyOperation("align-end")}
                >
                  ◆│
                </button>
              </>
            ) : null}
            {keySection === "stagger" ? (
              <>
                <small>STAGGER</small>
                <svg viewBox="0 0 96 38" aria-hidden="true">
                  <path d="M4 4 C28 4 64 34 92 34" />
                  <circle cx="4" cy="4" r="2" />
                  <circle cx="92" cy="34" r="2" />
                </svg>
                <button
                  type="button"
                  aria-label="等間隔に分布"
                  title="等間隔に分布"
                  onClick={() => onKeyOperation("stagger")}
                >
                  ◆··◆
                </button>
                <button
                  type="button"
                  aria-label="順序を反転"
                  title="順序を反転"
                  onClick={() => onKeyOperation("reverse")}
                >
                  ⇄
                </button>
              </>
            ) : null}
            {keySection === "stretch" ? (
              <>
                <small>STRETCH</small>
                <button type="button" onClick={() => onKeyOperation("stretch-80")}>
                  80%
                </button>
                <button
                  type="button"
                  onClick={() => onKeyOperation("stretch-120")}
                >
                  120%
                </button>
              </>
            ) : null}
          </div>
        </>
      ) : (
        <>
          <div className="candidate-key-tools-head">
            <b>▤ {layerCount}</b>
          </div>
          <div className="candidate-key-sections">
            {LAYER_SECTIONS.map(([section, icon, label]) => (
              <button
                type="button"
                aria-label={label}
                aria-expanded={layerSection === section}
                key={section}
                title={label}
                onClick={() =>
                  onLayerSectionChange(
                    layerSection === section ? null : section,
                  )
                }
              >
                {icon}
              </button>
            ))}
          </div>
          <div className="candidate-key-actions">
            {layerSection === "align" ? (
              <>
                <small>ALIGN</small>
                <button
                  type="button"
                  aria-label="Layerを開始へ整列"
                  onClick={() => onLayerOperation("align-start")}
                >
                  │▤
                </button>
                <button
                  type="button"
                  aria-label="Layerを終了へ整列"
                  onClick={() => onLayerOperation("align-end")}
                >
                  ▤│
                </button>
              </>
            ) : null}
            {layerSection === "stagger" ? (
              <>
                <small>STAGGER</small>
                <button
                  type="button"
                  aria-label="Layerを等間隔に分布"
                  onClick={() => onLayerOperation("stagger")}
                >
                  ▤··▤
                </button>
                <button
                  type="button"
                  aria-label="Layer順序を反転"
                  onClick={() => onLayerOperation("reverse")}
                >
                  ⇄
                </button>
              </>
            ) : null}
            {layerSection === "shift" ? (
              <>
                <small>SHIFT</small>
                <button
                  type="button"
                  aria-label="Layerを左へ移動"
                  onClick={() => onLayerOperation("shift-left")}
                >
                  ≪
                </button>
                <button
                  type="button"
                  aria-label="Layerを右へ移動"
                  onClick={() => onLayerOperation("shift-right")}
                >
                  ≫
                </button>
              </>
            ) : null}
          </div>
        </>
      )}
    </aside>
  );
}
