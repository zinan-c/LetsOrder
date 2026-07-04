export function formatDateTime(value?: string | null) {
  if (!value) {
    return 'Not set';
  }

  return new Intl.DateTimeFormat(undefined, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(new Date(value));
}

export function formatCompactDateTime(value?: string | null) {
  if (!value) {
    return 'Not set';
  }

  const date = new Date(value);
  const datePart = new Intl.DateTimeFormat(undefined, {
    month: 'short',
    day: 'numeric',
    year: 'numeric',
  }).format(date);
  const timePart = new Intl.DateTimeFormat(undefined, {
    hour: '2-digit',
    hour12: false,
    minute: '2-digit',
  }).format(date);

  return `${datePart} ${timePart}`;
}

export function toDateTimeLocalValue(value?: string | null) {
  if (!value) {
    return '';
  }

  const date = new Date(value);
  const offsetMs = date.getTimezoneOffset() * 60 * 1000;
  return new Date(date.getTime() - offsetMs).toISOString().slice(0, 16);
}
