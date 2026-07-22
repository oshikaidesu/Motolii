import { Button, PanelHeader } from "../primitives";

export function StageSurface({ composition, plugin }) {
  return (
    <section className="mock-surface mock-stage-shell" aria-label="Stage">
      <header className="stage-tools">
        <Button variant="quiet">Fit</Button>
        <Button variant="quiet">100%</Button>
        <PanelHeader className="stage-heading" title="Stage" wayfinding="stage" />
      </header>
      <div className="stage-canvas">
        <div className="output-frame">
          <span className="scene-orbit orbit-a" />
          <span className="scene-orbit orbit-b" />
          <strong>{composition}</strong>
          <small>{plugin.input} → {plugin.name} → {plugin.output}</small>
        </div>
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
