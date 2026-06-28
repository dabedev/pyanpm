import { PackageSearch } from "lucide-react";

import { Panel } from "@/components/ui/Panel";
import { StatusBadge } from "@/components/ui/StatusBadge";
import { AddPluginForm } from "@/features/plugins/AddPluginForm";
import { useDesktopStore } from "@/store/useDesktopStore";
import { pluginTone } from "@/utils/status";

export function PluginsPage() {
  const plugins = useDesktopStore((state) => state.plugins);
  const diffSnapshots = useDesktopStore((state) => state.diffSnapshots);
  const validation = useDesktopStore((state) => state.validation);
  const selectedPluginName = useDesktopStore((state) => state.selectedPluginName);
  const busy = useDesktopStore((state) => state.busy);
  const addPlugin = useDesktopStore((state) => state.addPlugin);
  const installPlugins = useDesktopStore((state) => state.installPlugins);
  const validateSource = useDesktopStore((state) => state.validateSource);
  const removePlugin = useDesktopStore((state) => state.removePlugin);
  const reinstallPlugin = useDesktopStore((state) => state.reinstallPlugin);
  const updatePlugins = useDesktopStore((state) => state.updatePlugins);
  const selectPlugin = useDesktopStore((state) => state.selectPlugin);

  const selectedPlugin = plugins.find((plugin) => plugin.name === selectedPluginName) ?? null;
  const selectedSnapshot = diffSnapshots.find((snapshot) => snapshot.pluginName === selectedPluginName) ?? null;

  return (
    <div className="grid gap-4 xl:grid-cols-[1.45fr_0.9fr]">
      <Panel
        title="Managed plugins"
        actions={<StatusBadge tone="neutral">{plugins.length} rows</StatusBadge>}
      >
        {plugins.length === 0 ? (
          <div className="flex min-h-64 flex-col items-center justify-center rounded-md border border-dashed border-zinc-800 bg-zinc-950/60 p-6 text-center">
            <PackageSearch className="h-6 w-6 text-zinc-600" />
            <h2 className="mt-4 text-sm font-semibold text-zinc-100">No managed plugins yet</h2>
            <p className="mt-2 max-w-sm text-xs leading-5 text-zinc-500">
              Add a local file or folder source on the right to create a manifest entry and install it into Studio.
            </p>
          </div>
        ) : (
          <div className="overflow-hidden rounded-md border border-zinc-800">
            <div className="grid grid-cols-[1.3fr_0.8fr_0.8fr_0.9fr] border-b border-zinc-800 bg-zinc-950/80 px-3 py-2 text-[10px] font-semibold uppercase tracking-[0.18em] text-zinc-500">
              <span>Name</span>
              <span>Status</span>
              <span>Version</span>
              <span>Source</span>
            </div>
            <div className="max-h-[30rem] overflow-auto">
              {plugins.map((plugin) => (
                <button
                  key={plugin.name}
                  className={`grid w-full grid-cols-[1.3fr_0.8fr_0.8fr_0.9fr] items-center gap-3 border-b border-zinc-900 px-3 py-3 text-left text-sm transition ${
                    selectedPluginName === plugin.name
                      ? "bg-teal-500/10"
                      : "bg-zinc-950/40 hover:bg-zinc-900/70"
                  }`}
                  onClick={() => selectPlugin(plugin.name)}
                  type="button"
                >
                  <span className="truncate font-medium text-zinc-100">{plugin.name}</span>
                  <span>
                    <StatusBadge tone={pluginTone(plugin.status)}>{plugin.status}</StatusBadge>
                  </span>
                  <span className="truncate font-mono text-xs text-zinc-400">
                    {plugin.installedVersion ?? plugin.requestedVersion ?? "-"}
                  </span>
                  <span className="truncate text-xs text-zinc-500">{plugin.source}</span>
                </button>
              ))}
            </div>
          </div>
        )}
      </Panel>

      <div className="space-y-4">
        <Panel title="Add plugin source">
          <AddPluginForm
            busy={busy.plugins}
            onValidate={async (input) => {
              await validateSource(input);
            }}
            onSubmit={async (input) => {
              await addPlugin(input);
            }}
            validationMessage={validation?.valid === false ? validation.errors[0]?.message ?? null : null}
          />
        </Panel>

        <Panel title="Install action">
          <button
            className="flex w-full items-center justify-between rounded-md border border-zinc-800 bg-zinc-950 px-4 py-3 text-left transition hover:border-zinc-600 hover:bg-zinc-900"
            onClick={() => void installPlugins()}
            type="button"
          >
            <div>
              <p className="text-sm font-semibold text-zinc-100">Install managed plugins</p>
              <p className="mt-1 text-xs text-zinc-500">Deploy cached artifacts into the active Studio plugins directory.</p>
            </div>
            <StatusBadge tone="neutral">{busy.plugins ? "running" : "ready"}</StatusBadge>
          </button>
        </Panel>

        <Panel title="Diff state">
          {selectedSnapshot ? (
            <dl className="grid gap-3 text-sm">
              <div>
                <dt className="text-[10px] font-semibold uppercase tracking-[0.18em] text-zinc-500">State</dt>
                <dd className="mt-1 text-zinc-100">{selectedSnapshot.state}</dd>
              </div>
              <div>
                <dt className="text-[10px] font-semibold uppercase tracking-[0.18em] text-zinc-500">Recommended action</dt>
                <dd className="mt-1 text-zinc-100">{selectedSnapshot.recommendedAction}</dd>
              </div>
              <div>
                <dt className="text-[10px] font-semibold uppercase tracking-[0.18em] text-zinc-500">Lock checksum</dt>
                <dd className="mt-1 break-all font-mono text-xs text-zinc-400">{selectedSnapshot.lockChecksum ?? "-"}</dd>
              </div>
              <div>
                <dt className="text-[10px] font-semibold uppercase tracking-[0.18em] text-zinc-500">Installed checksum</dt>
                <dd className="mt-1 break-all font-mono text-xs text-zinc-400">{selectedSnapshot.installedChecksum ?? "-"}</dd>
              </div>
            </dl>
          ) : (
            <p className="text-sm leading-6 text-zinc-500">Select a plugin to view its diff state.</p>
          )}
        </Panel>

        <Panel title="Plugin details">
          {selectedPlugin ? (
            <dl className="grid gap-3 text-sm">
              <div>
                <dt className="text-[10px] font-semibold uppercase tracking-[0.18em] text-zinc-500">Name</dt>
                <dd className="mt-1 text-zinc-100">{selectedPlugin.name}</dd>
              </div>
              <div>
                <dt className="text-[10px] font-semibold uppercase tracking-[0.18em] text-zinc-500">Source</dt>
                <dd className="mt-1 break-all font-mono text-xs text-zinc-400">{selectedPlugin.source}</dd>
              </div>
              <div className="grid grid-cols-2 gap-3">
                <div>
                  <dt className="text-[10px] font-semibold uppercase tracking-[0.18em] text-zinc-500">Requested</dt>
                  <dd className="mt-1 font-mono text-xs text-zinc-300">{selectedPlugin.requestedVersion ?? "-"}</dd>
                </div>
                <div>
                  <dt className="text-[10px] font-semibold uppercase tracking-[0.18em] text-zinc-500">Installed</dt>
                  <dd className="mt-1 font-mono text-xs text-zinc-300">{selectedPlugin.installedVersion ?? "-"}</dd>
                </div>
              </div>
              <div>
                <dt className="text-[10px] font-semibold uppercase tracking-[0.18em] text-zinc-500">Target path</dt>
                <dd className="mt-1 break-all font-mono text-xs text-zinc-400">{selectedPlugin.targetPath ?? "-"}</dd>
              </div>
              <div className="grid gap-2 pt-2">
                <button
                  className="rounded-md border border-zinc-800 bg-zinc-950 px-3 py-2 text-left text-xs text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-900"
                  onClick={() => void reinstallPlugin(selectedPlugin.name)}
                  type="button"
                >
                  Reinstall plugin
                </button>
                <button
                  className="rounded-md border border-zinc-800 bg-zinc-950 px-3 py-2 text-left text-xs text-zinc-200 transition hover:border-zinc-600 hover:bg-zinc-900"
                  onClick={() => void updatePlugins(selectedPlugin.name, false, false)}
                  type="button"
                >
                  Update selected
                </button>
                <button
                  className="rounded-md border border-amber-700/40 bg-amber-500/5 px-3 py-2 text-left text-xs text-amber-100 transition hover:border-amber-500/60 hover:bg-amber-500/10"
                  onClick={() => void removePlugin(selectedPlugin.name, false)}
                  type="button"
                >
                  Remove managed plugin
                </button>
              </div>
            </dl>
          ) : (
            <p className="text-sm leading-6 text-zinc-500">Select a plugin to view its details.</p>
          )}
        </Panel>
      </div>
    </div>
  );
}
