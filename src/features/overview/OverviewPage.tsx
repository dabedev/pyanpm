import { AlertTriangle, Boxes, FolderRoot, RefreshCcw, ShieldCheck } from "lucide-react";

import { Button } from "@/components/ui/Button";
import { Panel } from "@/components/ui/Panel";
import { StatusBadge } from "@/components/ui/StatusBadge";
import { useDesktopStore } from "@/store/useDesktopStore";

function Metric({
  label,
  value,
  hint,
}: {
  label: string;
  value: string;
  hint: string;
}) {
  return (
    <div className="rounded-md border border-zinc-800 bg-zinc-950/70 p-4">
      <p className="text-[10px] font-semibold uppercase tracking-[0.2em] text-zinc-500">{label}</p>
      <p className="mt-3 font-mono text-xl text-zinc-100">{value}</p>
      <p className="mt-2 text-xs text-zinc-500">{hint}</p>
    </div>
  );
}

export function OverviewPage() {
  const settings = useDesktopStore((state) => state.settings);
  const plugins = useDesktopStore((state) => state.plugins);
  const doctorReport = useDesktopStore((state) => state.doctorReport);
  const activity = useDesktopStore((state) => state.activity);
  const busy = useDesktopStore((state) => state.busy);
  const initManifest = useDesktopStore((state) => state.initManifest);
  const installPlugins = useDesktopStore((state) => state.installPlugins);
  const runDoctor = useDesktopStore((state) => state.runDoctor);
  const refreshPlugins = useDesktopStore((state) => state.refreshPlugins);

  const issueCount = doctorReport?.findings.filter((finding) => finding.level !== "ok").length ?? 0;
  const healthy = doctorReport?.healthy ?? false;

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between gap-4">
        <div>
          <p className="font-mono text-[10px] uppercase tracking-[0.22em] text-zinc-500">Overview</p>
          <h1 className="mt-2 text-2xl font-semibold text-zinc-100">pyanPM</h1>
        </div>
        <div className="flex items-center gap-2">
          <StatusBadge tone={healthy ? "ok" : "warn"}>{healthy ? "Healthy" : "Attention needed"}</StatusBadge>
          <Button variant="ghost" onClick={() => void refreshPlugins()} disabled={busy.plugins}>
            Refresh list
          </Button>
        </div>
      </div>

      <div className="grid gap-4 xl:grid-cols-4">
        <Metric
          label="Managed plugins"
          value={plugins.length.toString().padStart(2, "0")}
          hint="From the current manifest and lockfile."
        />
        <Metric
          label="Doctor findings"
          value={issueCount.toString().padStart(2, "0")}
          hint="Non-OK checks needing review or action."
        />
        <Metric
          label="Global Config"
          value={settings?.stateDir ? "global" : "unset"}
          hint={settings?.stateDir ?? "Unavailable"}
        />
        <Metric
          label="Plugins directory"
          value={settings?.pluginsDirOverride ? "selected" : "unset"}
          hint={
            settings?.pluginsDirOverride ??
            settings?.detectedPluginsDir ??
            "Pick the Roblox Plugins folder in Settings for this user installation."
          }
        />
      </div>

      <div className="grid gap-4 xl:grid-cols-[1.4fr_1fr]">
        <Panel title="Quick actions">
          <div className="grid gap-3 md:grid-cols-2">
            <button
              className="rounded-md border border-zinc-800 bg-zinc-950/70 p-4 text-left transition hover:border-zinc-600 hover:bg-zinc-900"
              onClick={() => void initManifest()}
              type="button"
            >
              <div className="flex items-center justify-between">
                <FolderRoot className="h-4 w-4 text-teal-300" />
                <StatusBadge tone="neutral">init</StatusBadge>
              </div>
              <h2 className="mt-4 text-sm font-semibold text-zinc-100">Initialize manifest</h2>
              <p className="mt-2 text-xs leading-5 text-zinc-500">Create the global `pyanpm.toml` store for this user.</p>
            </button>

            <button
              className="rounded-md border border-zinc-800 bg-zinc-950/70 p-4 text-left transition hover:border-zinc-600 hover:bg-zinc-900"
              onClick={() => void installPlugins()}
              type="button"
            >
              <div className="flex items-center justify-between">
                <Boxes className="h-4 w-4 text-teal-300" />
                <StatusBadge tone="neutral">install</StatusBadge>
              </div>
              <h2 className="mt-4 text-sm font-semibold text-zinc-100">Install locked plugins</h2>
              <p className="mt-2 text-xs leading-5 text-zinc-500">Stage and deploy cached artifacts into Studio.</p>
            </button>

            <button
              className="rounded-md border border-zinc-800 bg-zinc-950/70 p-4 text-left transition hover:border-zinc-600 hover:bg-zinc-900"
              onClick={() => void runDoctor()}
              type="button"
            >
              <div className="flex items-center justify-between">
                <ShieldCheck className="h-4 w-4 text-teal-300" />
                <StatusBadge tone="neutral">doctor</StatusBadge>
              </div>
              <h2 className="mt-4 text-sm font-semibold text-zinc-100">Run doctor checks</h2>
              <p className="mt-2 text-xs leading-5 text-zinc-500">Validate manifests, plugin paths, and checksum drift.</p>
            </button>

            <button
              className="rounded-md border border-zinc-800 bg-zinc-950/70 p-4 text-left transition hover:border-zinc-600 hover:bg-zinc-900"
              onClick={() => void refreshPlugins()}
              type="button"
            >
              <div className="flex items-center justify-between">
                <RefreshCcw className="h-4 w-4 text-teal-300" />
                <StatusBadge tone="neutral">list</StatusBadge>
              </div>
              <h2 className="mt-4 text-sm font-semibold text-zinc-100">Refresh managed plugins</h2>
              <p className="mt-2 text-xs leading-5 text-zinc-500">Refresh the plugin list.</p>
            </button>
          </div>
        </Panel>

        <Panel title="Recent activity">
          <div className="space-y-3">
            {activity.map((entry) => (
              <div key={entry.id} className="rounded-md border border-zinc-800 bg-zinc-950/60 p-3">
                <div className="flex items-center justify-between gap-3">
                  <p className="text-xs font-semibold text-zinc-100">{entry.title}</p>
                  <span className="font-mono text-[10px] text-zinc-500">{entry.timestamp}</span>
                </div>
                <p className="mt-2 text-xs leading-5 text-zinc-500">{entry.detail}</p>
              </div>
            ))}
          </div>
        </Panel>
      </div>

      {!healthy ? (
        <Panel
          title="Health attention"
          actions={<AlertTriangle className="h-4 w-4 text-amber-300" />}
        >
          <p className="text-sm leading-6 text-zinc-300">
            Visit the Doctor page for full finding codes and remediation guidance before shipping plugin updates.
          </p>
        </Panel>
      ) : null}
    </div>
  );
}
