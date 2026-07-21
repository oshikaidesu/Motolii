import { useEffect, useState } from "react";

export const DEFAULT_FIXTURE = "plugin-browser-candidate";
export const CATALOG_FIXTURE = "catalog";
export const ARCHIVE_CATALOG_FIXTURE = "archive/catalog";

function fixtureFromHash() {
  const fixture = decodeURIComponent(window.location.hash.slice(1)).trim();
  return fixture || DEFAULT_FIXTURE;
}

function Placeholder({ fixture, message }) {
  return (
    <main data-fixture={fixture}>
      <h1>Motolii UI fixture</h1>
      <p>{message}</p>
    </main>
  );
}

function Fixture({ fixture, entry }) {
  const Screen = entry.Component;

  return (
    <section data-fixture={fixture} aria-label={entry.title ?? fixture}>
      <Screen {...entry.props} />
    </section>
  );
}

export function App({ registry = {} }) {
  const [fixture, setFixture] = useState(fixtureFromHash);

  useEffect(() => {
    const onHashChange = () => setFixture(fixtureFromHash());
    window.addEventListener("hashchange", onHashChange);
    return () => window.removeEventListener("hashchange", onHashChange);
  }, []);

  const entries = Object.entries(registry);

  if (
    fixture === CATALOG_FIXTURE ||
    fixture === ARCHIVE_CATALOG_FIXTURE
  ) {
    const wantsArchive = fixture === ARCHIVE_CATALOG_FIXTURE;
    const visibleEntries = entries.filter(
      ([, entry]) => Boolean(entry.archive) === wantsArchive,
    );
    if (visibleEntries.length === 0) {
      return (
        <Placeholder
          fixture={fixture}
          message="画面fixtureはまだ登録されていません。"
        />
      );
    }

    return (
      <div data-fixture={fixture}>
        {visibleEntries.map(([key, entry]) => (
          <Fixture key={key} fixture={key} entry={entry} />
        ))}
      </div>
    );
  }

  const entry = registry[fixture];
  if (!entry) {
    return (
      <Placeholder
        fixture={fixture}
        message={`未登録のfixtureです: ${fixture}`}
      />
    );
  }

  return (
    <div data-fixture={fixture}>
      <Fixture fixture={fixture} entry={entry} />
    </div>
  );
}
