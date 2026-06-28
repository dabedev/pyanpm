import type { PropsWithChildren, ReactNode } from "react";

import { cn } from "@/lib/utils";

type PanelProps = PropsWithChildren<{
  title?: string;
  subtitle?: string;
  actions?: ReactNode;
  className?: string;
}>;

export function Panel({ title, subtitle, actions, className, children }: PanelProps) {
  return (
    <section className={cn("rounded-md border border-zinc-800 bg-zinc-950/90", className)}>
      {(title || subtitle || actions) && (
        <header className="flex items-start justify-between gap-3 border-b border-zinc-800 px-4 py-3">
          <div className="min-w-0">
            {title ? <h2 className="text-sm font-semibold text-zinc-100">{title}</h2> : null}
            {subtitle ? <p className="mt-1 text-xs text-zinc-500">{subtitle}</p> : null}
          </div>
          {actions ? <div className="flex shrink-0 items-center gap-2">{actions}</div> : null}
        </header>
      )}
      <div className="p-4">{children}</div>
    </section>
  );
}
