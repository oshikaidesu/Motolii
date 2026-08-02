import { Button } from "../primitives";

export function GlobalTitlebar({ project }) {
  return (
    <header className="mock-titlebar">
      <strong className="wordmark">MOTOLII</strong>
      <span>{project}</span>
      <span className="mock-grow" />
      <Button variant="quiet">Settings</Button>
      <Button>Export</Button>
    </header>
  );
}
