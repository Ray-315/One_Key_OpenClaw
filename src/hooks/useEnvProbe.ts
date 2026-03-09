import { useEffect } from "react";
import { useEnvStore } from "../store/envStore";

/**
 * Automatically trigger environment probing on mount.
 */
export function useEnvProbe() {
  const probeAll = useEnvStore((s) => s.probeAll);

  useEffect(() => {
    probeAll();
  }, [probeAll]);

  return useEnvStore();
}
