import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

/**
 * Generic hook to subscribe to a Tauri event.
 * Automatically unsubscribes on cleanup.
 */
export function useTauriEvent<T>(
  eventName: string,
  handler: (payload: T) => void
) {
  useEffect(() => {
    const unlisten = listen<T>(eventName, (event) => {
      handler(event.payload);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [eventName, handler]);
}
