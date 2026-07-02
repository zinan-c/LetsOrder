interface StatusPillProps {
  children: string;
  tone?: 'warm' | 'green' | 'red' | 'neutral';
}

export default function StatusPill({
  children,
  tone = 'warm',
}: StatusPillProps) {
  return <span className={`status-pill status-pill-${tone}`}>{children}</span>;
}
