import { Panel } from "@/components/ui/Panel";
import { useDesktopStore } from "@/store/useDesktopStore";

export function CachePage() {
  const cacheEntries = useDesktopStore((state) => state.cacheEntries);
  const busy = useDesktopStore((state) => state.busy);
  const refreshCache = useDesktopStore((state) => state.refreshCache);
  const evictCache = useDesktopStore((state) => state.evictCache);
  const pruneCache = useDesktopStore((state) => state.pruneCache);

  return (
    <Panel
      title="Cache"
      actions={
        <div className="flex gap-2">
          <button
            className="rounded-md border border-zinc-800 bg-zinc-950 px-3 py-1 text-xs text-zinc-300 transition hover:border-zinc-600 hover:bg-zinc-900"
            onClick={() => void refreshCache()}
            type="button"
          >
            {busy.cache ? "Refreshing" : "Refresh"}
          </button>
          <button
            className="rounded-md border border-zinc-800 bg-zinc-950 px-3 py-1 text-xs text-zinc-300 transition hover:border-zinc-600 hover:bg-zinc-900"
            onClick={() => void pruneCache()}
            type="button"
          >
            Prune
          </button>
        </div>
      }
    >
      <div className="space-y-3">
        {cacheEntries.map((entry) => (
          <div key={entry.cacheId} className="rounded-md border border-zinc-800 bg-zinc-950/60 p-3">
            <div className="flex items-center justify-between gap-3">
              <div>
                <p className="text-sm font-semibold text-zinc-100">{entry.pluginName}</p>
                <p className="text-xs text-zinc-500">{entry.sourceSummary}</p>
              </div>
              <button
                className="rounded-md border border-zinc-800 bg-zinc-950 px-3 py-1 text-xs text-zinc-300 transition hover:border-zinc-600 hover:bg-zinc-900"
                onClick={() => void evictCache(entry.cacheId)}
                type="button"
              >
                Evict
              </button>
            </div>
            <div className="mt-3 grid gap-2 text-xs text-zinc-400 md:grid-cols-3">
              <span>{entry.checksum}</span>
              <span>{entry.sizeBytes} bytes</span>
              <span>{entry.referencedByActiveLockfile ? "active lockfile" : "unreferenced"}</span>
            </div>
          </div>
        ))}
        {cacheEntries.length === 0 ? (
          <p className="text-sm leading-6 text-zinc-500">No cache entries are available yet.</p>
        ) : null}
      </div>
    </Panel>
  );
}
