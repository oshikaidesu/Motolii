import { TimelineSurface } from "../surfaces/TimelineSurface.jsx";
import { BrowserSurface } from "../surfaces/BrowserSurface.jsx";
import { InspectorSurface } from "../surfaces/InspectorSurface.jsx";
import { StageSurface } from "../surfaces/StageSurface.jsx";
import { Button } from "../primitives/index.jsx";
import { referenceFixtures } from "./fixtures.js";
import { loadReferenceFixtures } from "./loadReferenceFixtures.js";
import { ReferenceStateStrip } from "./ReferenceStateStrip.jsx";

export function SharedEffectRelativeReference() {
  const fixture = loadReferenceFixtures("shared-effect-relative", referenceFixtures);
  return (
    <main className="motolii-mock-app" data-reference-screen="shared-effect-relative">
      <header className="mock-titlebar">
        <strong className="wordmark">MOTOLII</strong><span>Shared effect and relative edit</span>
      </header>
      <nav className="mock-commandbar" aria-label="Relative edit commands">
        <Button pressed>SELECT</Button><Button variant="quiet">RELATIVE</Button>
      </nav>
      <section className="mock-workspace">
        <BrowserSurface fixture={fixture.browser} />
        <StageSurface composition={fixture.composition} plugin={fixture.plugin} />
        <InspectorSurface fixture={fixture.inspector} plugin={fixture.plugin} />
      </section>
      <TimelineSurface fixture={fixture.timeline} />
      <ReferenceStateStrip
        spacing={fixture.tokenSpacing}
        items={[{ label: "Shared-use and drag states are projected into the Timeline" }]}
      />
    </main>
  );
}
