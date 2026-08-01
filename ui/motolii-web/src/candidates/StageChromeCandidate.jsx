export function StageHeaderCandidate({ mode }) {
  return (
    <div className="stage-tools">
      <button className="toolbtn" type="button">Fit</button>
      <button className="toolbtn" type="button">100%</button>
      <button className="toolbtn on" type="button" aria-label="Stage grid">▦</button>
      <span className="stage-mode">
        STAGE / <b id="stage-mode">{mode}</b>
      </span>
    </div>
  );
}

export function StageTransportCandidate({
  timecode,
  barPosition,
  tempoStatus,
  qualityStatus,
  easingTrigger,
}) {
  return (
    <div className="transport">
      <button className="toolbtn" id="step-prev" type="button" aria-label="前のkeyへ">|‹</button>
      <button className="toolbtn" id="play" type="button" aria-label="再生">▶</button>
      <button className="toolbtn" id="step-next" type="button" aria-label="次のkeyへ">›|</button>
      {easingTrigger}
      <span className="time" id="time">{timecode}</span>
      <span>{barPosition}</span>
      <span>{tempoStatus}</span>
      <span className="quality">{qualityStatus}</span>
    </div>
  );
}
