/** Returns the current platform string */
export function getPlatform(): "macos" | "windows" | "linux" | "unknown" {
  const ua = navigator.userAgent.toLowerCase();
  if (ua.includes("mac")) return "macos";
  if (ua.includes("win")) return "windows";
  if (ua.includes("linux")) return "linux";
  return "unknown";
}
