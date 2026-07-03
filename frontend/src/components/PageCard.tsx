import type { PropsWithChildren } from 'react';

interface PageCardProps extends PropsWithChildren {
  eyebrow?: string;
  title: string;
  titleClassName?: string;
  description: string;
}

export default function PageCard({
  eyebrow,
  title,
  titleClassName,
  description,
  children,
}: PageCardProps) {
  return (
    <section className="page-card">
      {eyebrow ? <p className="eyebrow">{eyebrow}</p> : null}
      <h1 className={titleClassName}>{title}</h1>
      <p className="lead">{description}</p>
      {children}
    </section>
  );
}
