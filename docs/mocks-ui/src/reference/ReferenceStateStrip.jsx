export function ReferenceStateStrip({ items, spacing }) {
  return (
    <footer
      className="mock-status"
      aria-label="Reference comparison states"
      style={{ gap: `${spacing}px`, paddingInline: `${spacing}px` }}
    >
      {items.map(({ id, label }) => (
        <span {...(id ? { "data-semantic-id": id } : {})} key={id ?? label}>{label}</span>
      ))}
    </footer>
  );
}
