import { useEffect, useState } from "react";

import { Button } from "@/components/ui/Button";
import { Panel } from "@/components/ui/Panel";
import { useDesktopStore } from "@/store/useDesktopStore";
import { pickDirectoryPath } from "@/utils/nativeDesktop";

export function SettingsPage() {
  const settings = useDesktopStore((state) => state.settings);
  const busy = useDesktopStore((state) => state.busy);
  const saveSettings = useDesktopStore((state) => state.saveSettings);

  const [stateDir, setStateDir] = useState("");
  const [pluginsDirOverride, setPluginsDirOverride] = useState("");
  const [notificationsEnabled, setNotificationsEnabled] = useState(true);
  const defaultStateDir = settings?.defaultStateDir ?? "";
  const detectedPluginsDir = settings?.detectedPluginsDir ?? "";

  useEffect(() => {
    if (!settings) {
      return;
    }

    setStateDir(settings.stateDir ?? "");
    setPluginsDirOverride(settings.pluginsDirOverride ?? "");
    setNotificationsEnabled(settings.notificationsEnabled);
  }, [settings]);

  return (
    <div className="grid gap-4 xl:grid-cols-[1.1fr_0.9fr]">
      <Panel title="Settings">
        <div className="space-y-4">
          <label className="block">
            <span className="mb-2 block text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500">
              Global Config
            </span>
            <p className="mb-2 truncate font-mono text-xs text-zinc-500">
              {defaultStateDir || "No default Global Config path is available."}
            </p>
            <div className="flex gap-2">
              <input
                className="h-9 min-w-0 flex-1 rounded-md border border-zinc-800 bg-zinc-950 px-3 font-mono text-sm text-zinc-100 outline-none transition focus:border-teal-400"
                onChange={(event) => setStateDir(event.target.value)}
                placeholder={defaultStateDir || ""}
                value={stateDir}
              />
              <Button
                variant="secondary"
                type="button"
                onClick={async () => {
                  const selected = await pickDirectoryPath(stateDir || defaultStateDir);
                  if (selected) {
                    setStateDir(selected);
                  }
                }}
              >
                Browse
              </Button>
            </div>
            {defaultStateDir ? (
              <div className="mt-2 flex items-center justify-between gap-3 rounded-md border border-zinc-800 bg-zinc-950/60 px-3 py-2">
                <div className="min-w-0">
                  <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-zinc-500">Default path</p>
                  <p className="mt-1 truncate font-mono text-xs text-zinc-300">{defaultStateDir}</p>
                </div>
                <Button variant="ghost" type="button" onClick={() => setStateDir(defaultStateDir)}>
                  Use default
                </Button>
              </div>
            ) : null}
          </label>

          <label className="block">
            <span className="mb-2 block text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500">
              Roblox plugins directory
            </span>
            <p className="mb-2 truncate font-mono text-xs text-zinc-500">
              {detectedPluginsDir || "No default Roblox Plugins directory was detected on this machine."}
            </p>
            <div className="flex gap-2">
              <input
                className="h-9 min-w-0 flex-1 rounded-md border border-zinc-800 bg-zinc-950 px-3 font-mono text-sm text-zinc-100 outline-none transition focus:border-teal-400"
                onChange={(event) => setPluginsDirOverride(event.target.value)}
                placeholder={detectedPluginsDir || ""}
                value={pluginsDirOverride}
              />
              <Button
                variant="secondary"
                type="button"
                onClick={async () => {
                  const selected = await pickDirectoryPath(pluginsDirOverride || detectedPluginsDir || stateDir || defaultStateDir);
                  if (selected) {
                    setPluginsDirOverride(selected);
                  }
                }}
              >
                Browse
              </Button>
            </div>
            {detectedPluginsDir ? (
              <div className="mt-2 flex items-center justify-between gap-3 rounded-md border border-zinc-800 bg-zinc-950/60 px-3 py-2">
                <div className="min-w-0">
                  <p className="text-[10px] font-semibold uppercase tracking-[0.18em] text-zinc-500">Detected path</p>
                  <p className="mt-1 truncate font-mono text-xs text-zinc-300">{detectedPluginsDir}</p>
                </div>
                <Button variant="ghost" type="button" onClick={() => setPluginsDirOverride(detectedPluginsDir)}>
                  Use detected
                </Button>
              </div>
            ) : null}
          </label>

          <label className="flex items-center justify-between rounded-md border border-zinc-800 bg-zinc-950/60 px-3 py-3">
            <div>
              <span className="block text-[11px] font-semibold uppercase tracking-[0.18em] text-zinc-500">
                Background notifications
              </span>
              <span className="mt-1 block text-xs text-zinc-400">
                Show notifications when the window is unfocused.
              </span>
            </div>
            <input
              checked={notificationsEnabled}
              className="h-4 w-4 accent-teal-400"
              onChange={(event) => setNotificationsEnabled(event.target.checked)}
              type="checkbox"
            />
          </label>

          <div className="flex justify-end">
            <Button
              variant="primary"
              onClick={() => void saveSettings(stateDir, pluginsDirOverride, notificationsEnabled)}
              disabled={busy.settings || !((stateDir.trim() || defaultStateDir) && (pluginsDirOverride.trim() || detectedPluginsDir))}
            >
              Save settings
            </Button>
          </div>
        </div>
      </Panel>

    </div>
  );
}
