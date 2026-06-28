import type { PropsWithChildren } from "react";
import { useEffect } from "react";

import { useDesktopStore } from "@/store/useDesktopStore";

export function AppProviders({ children }: PropsWithChildren) {
  const bootstrap = useDesktopStore((state) => state.bootstrap);

  useEffect(() => {
    void bootstrap();
  }, [bootstrap]);

  return children;
}
