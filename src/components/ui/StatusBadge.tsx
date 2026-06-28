import type { ReactNode } from "react";

import { cn } from "@/lib/utils";

type StatusBadgeProps = {
  tone: "ok" | "warn" | "error" | "neutral";
  children: ReactNode;
};

const toneClasses: Record<StatusBadgeProps["tone"], string> = {
  ok: "border-teal-500/40 bg-teal-500/10 text-teal-200",
  warn: "border-amber-500/40 bg-amber-500/10 text-amber-200",
  error: "border-red-500/40 bg-red-500/10 text-red-200",
  neutral: "border-zinc-700 bg-zinc-900 text-zinc-300",
};

export function StatusBadge({ tone, children }: StatusBadgeProps) {
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-md border px-2 py-1 text-[10px] font-semibold uppercase tracking-[0.18em]",
        toneClasses[tone]
      )}
    >
      {children}
    </span>
  );
}
