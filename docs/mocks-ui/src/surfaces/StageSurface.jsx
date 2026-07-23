import { Button, PanelHeader } from "../primitives";

export function StageSurface({ composition, plugin, frameState = null }) {
  return (
    <section
      className="mock-surface mock-stage-shell"
      aria-label="Stage"
      data-semantic-id={frameState ? "stage" : undefined}
    >
      <header className="stage-tools">
        <Button variant="quiet">Fit</Button>
        <Button variant="quiet">100%</Button>
        <PanelHeader className="stage-heading" title="Stage" wayfinding="stage" />
      </header>
      <div
        className="stage-canvas"
        aria-label={frameState ? `Stage scrim ${frameState.scrimPercent}%` : undefined}
      >
        <div className="output-frame" data-semantic-id={frameState ? "output-frame" : undefined}>
          <span className="scene-orbit orbit-a" />
          <span className="scene-orbit orbit-b" />
          <strong data-semantic-id={frameState ? "inside-object" : undefined}>{composition}</strong>
          <small>{plugin.input} → {plugin.name} → {plugin.output}</small>
          {frameState && <span data-semantic-id="selection">Selected · {frameState.selected}</span>}
        </div>
        {frameState && (
          <>
            <span data-semantic-id="outside-object">Outside frame · {frameState.outside}</span>
            <span data-semantic-id="scrim">Scrim · {frameState.scrimPercent}%</span>
          </>
        )}
      </div>
      <footer className="transport">
        <Button variant="quiet" aria-label="Previous key">|‹</Button>
        <Button variant="quiet" aria-label="Play">▶</Button>
        <Button variant="quiet" aria-label="Next key">›|</Button>
        <strong>00:54.2</strong>
        <span>120 BPM · SNAP BEAT</span>
        <span className="quality">DRAFT · FP16 · 1/2</span>
      </footer>
    </section>
  );
}
