import type { PropsWithChildren } from "react";

import { Activity, Database, FolderTree, Package2, Settings2, Stethoscope } from "lucide-react";
import { NavLink } from "react-router-dom";

import { cn } from "@/lib/utils";

const navItems = [
  { to: "/", label: "Overview", icon: Activity },
  { to: "/activity", label: "Activity", icon: Activity },
  { to: "/plugins", label: "Plugins", icon: Package2 },
  { to: "/cache", label: "Cache", icon: Database },
  { to: "/doctor", label: "Doctor", icon: Stethoscope },
  { to: "/settings", label: "Settings", icon: Settings2 },
];

export function AppShell({ children }: PropsWithChildren) {
  return (
    <div className="grid min-h-screen grid-cols-[220px_minmax(0,1fr)] bg-[radial-gradient(circle_at_top,_rgba(20,184,166,0.15),_transparent_28%),linear-gradient(180deg,#0a0d11_0%,#0b0e13_100%)] text-zinc-100">
      <aside className="border-r border-zinc-800 bg-zinc-950/80 px-4 py-4">
        <div className="mb-6 flex items-center gap-3 border border-zinc-800 bg-zinc-900/80 px-3 py-3">
          <div className="flex h-9 w-9 items-center justify-center rounded-md border border-zinc-700 bg-zinc-950">
            <FolderTree className="h-4 w-4 text-teal-300" />
          </div>
          <div>
            <p className="font-mono text-[10px] uppercase tracking-[0.22em] text-zinc-500">pyanPM</p>
            <h1 className="text-sm font-semibold text-zinc-100">Companion</h1>
          </div>
        </div>

        <nav className="space-y-1">
          {navItems.map((item) => {
            const Icon = item.icon;
            return (
              <NavLink
                key={item.to}
                to={item.to}
                className={({ isActive }) =>
                  cn(
                    "flex items-center gap-3 rounded-md border px-3 py-2 text-xs font-medium transition",
                    isActive
                      ? "border-teal-500/40 bg-teal-500/10 text-teal-100"
                      : "border-transparent text-zinc-400 hover:border-zinc-800 hover:bg-zinc-900 hover:text-zinc-100"
                  )
                }
              >
                <Icon className="h-4 w-4" />
                <span>{item.label}</span>
              </NavLink>
            );
          })}
        </nav>
      </aside>

      <main className="min-w-0 px-5 py-5">{children}</main>
    </div>
  );
}
