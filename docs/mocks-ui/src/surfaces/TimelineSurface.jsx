import { IconButton, PanelHeader } from "../primitives";

export function TimelineSurface({ fixture }) {
  return (
    <section className="mock-surface mock-timeline" aria-label="Timeline">
      <PanelHeader
        title="譜面 / Timeline"
        detail={fixture.timecode}
        wayfinding="timeline"
        actions={<IconButton label="Open Depth Rail" glyph="≋" />}
      />
      <div className="depth-rail">
        <strong>DEPTH</strong>
        <span>ROOT</span>
        <div className="depth-axis">
          {(fixture.depthLabels ?? ["Emitter +.18", "Text 0", "CAM +.42 →"]).map((label, index, labels) => (
            <i
              key={label}
              style={{ left: `${((index + 1) / (labels.length + 1)) * 100}%` }}
            >
              {label}
            </i>
          ))}
        </div>
        {fixture.statuses?.map((status) => (
          <span data-semantic-id={status.id} key={status.id}>{status.label}</span>
        ))}
      </div>
      <div className="timeline-body">
        <aside className="inbox">
          <strong>INBOX</strong>
          <span>{(fixture.inbox ?? ["Asset · skyline.png", "Note · Check beat", "Job · Draft ready"]).length}</span>
          {(fixture.inbox ?? ["Asset · skyline.png", "Note · Check beat", "Job · Draft ready"]).map((entry) => (
            <button key={entry}>{entry}</button>
          ))}
        </aside>
        <div className="time-plane">
          {fixture.bars.map((bar, index) => (
            <div
              className={`time-bar object-${index + 1}`}
              key={bar.name}
              style={{ left: `${bar.left}%`, width: `${bar.width}%`, top: `${24 + index * 48}px` }}
            >
              <strong>{bar.kind} · {bar.name}</strong>
              <span>{bar.depth}</span>
              {bar.flow && <small>{bar.flow}</small>}
              {bar.states?.map((state) => (
                <small data-semantic-id={state.id} key={state.id}>{state.label}</small>
              ))}
            </div>
          ))}
          <span className="playhead" />
        </div>
      </div>
    </section>
  );
}
