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
          <i style={{ left: "68%" }}>Emitter +.18</i>
          <i style={{ left: "50%" }}>Text 0</i>
          <i style={{ left: "91%" }}>CAM +.42 →</i>
        </div>
      </div>
      <div className="timeline-body">
        <aside className="inbox">
          <strong>INBOX</strong>
          <span>3</span>
          <button>Asset · skyline.png</button>
          <button>Note · Check beat</button>
          <button>Job · Draft ready</button>
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
            </div>
          ))}
          <span className="playhead" />
        </div>
      </div>
    </section>
  );
}
