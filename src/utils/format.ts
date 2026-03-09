/** Format a Unix timestamp (ms) to locale time string */
export function formatTime(timestamp: number): string {
  return new Date(timestamp).toLocaleTimeString();
}

/** Format a Unix timestamp (ms) to locale date+time string */
export function formatDateTime(timestamp: number): string {
  return new Date(timestamp).toLocaleString();
}
