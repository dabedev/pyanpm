import type { ButtonHTMLAttributes, PropsWithChildren } from "react";

import { cn } from "@/lib/utils";

type ButtonProps = PropsWithChildren<
  ButtonHTMLAttributes<HTMLButtonElement> & {
    variant?: "primary" | "secondary" | "ghost" | "danger";
  }
>;

const variantClasses: Record<NonNullable<ButtonProps["variant"]>, string> = {
  primary:
    "border-teal-500/60 bg-teal-500/10 text-teal-100 hover:bg-teal-500/20 disabled:border-zinc-800 disabled:bg-zinc-900 disabled:text-zinc-600",
  secondary:
    "border-zinc-700 bg-zinc-900 text-zinc-100 hover:border-zinc-500 hover:bg-zinc-800 disabled:text-zinc-600",
  ghost:
    "border-transparent bg-transparent text-zinc-300 hover:border-zinc-800 hover:bg-zinc-900",
  danger:
    "border-red-500/50 bg-red-500/10 text-red-100 hover:bg-red-500/20 disabled:border-zinc-800 disabled:bg-zinc-900 disabled:text-zinc-600",
};

export function Button({
  className,
  children,
  variant = "secondary",
  ...props
}: ButtonProps) {
  return (
    <button
      className={cn(
        "inline-flex h-8 items-center justify-center rounded-md border px-3 text-[11px] font-semibold uppercase tracking-[0.18em] transition focus:outline-none focus:ring-1 focus:ring-teal-400 disabled:cursor-not-allowed",
        variantClasses[variant],
        className
      )}
      {...props}
    >
      {children}
    </button>
  );
}
