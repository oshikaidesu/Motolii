import { BrowserSurface } from "../surfaces/BrowserSurface.jsx";
import { Button } from "../primitives/index.jsx";
import { referenceFixtures } from "./fixtures.js";
import { loadReferenceFixtures } from "./loadReferenceFixtures.js";
import { ReferenceStateStrip } from "./ReferenceStateStrip.jsx";

export function EmptyBrowserReference() {
  const fixture = loadReferenceFixtures("empty-browser", referenceFixtures);
  return (
    <main className="motolii-mock-app" data-reference-screen="empty-browser">
      <header className="mock-titlebar">
        <strong className="wordmark">MOTOLII</strong>
        <span data-semantic-id="empty-project">Untitled project · no media</span>
        <span className="mock-grow" />
        <Button>Import media</Button>
      </header>
      <nav className="mock-commandbar" aria-label="Empty project commands">
        <Button pressed>SELECT</Button>
        <span data-semantic-id="context-explanation">Import or create an item to begin</span>
      </nav>
      <section className="mock-workspace" data-semantic-id="asset-browser">
        <BrowserSurface fixture={fixture.browser} />
        <section className="mock-surface mock-stage-shell" aria-label="Empty Stage" />
        <aside className="mock-surface mock-inspector" aria-label="Empty Inspector" />
      </section>
      <section className="mock-surface mock-timeline" aria-label="Empty Timeline" />
      <ReferenceStateStrip
        spacing={fixture.tokenSpacing}
        items={[{ id: "transport", label: "Transport · 00:00.0 · stopped" }]}
      />
    </main>
  );
}
