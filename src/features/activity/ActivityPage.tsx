import { Panel } from "@/components/ui/Panel";
import { StatusBadge } from "@/components/ui/StatusBadge";
import { useDesktopStore } from "@/store/useDesktopStore";

export function ActivityPage() {
  const activityRecords = useDesktopStore((state) => state.activityRecords);
  const busy = useDesktopStore((state) => state.busy);
  const refreshActivity = useDesktopStore((state) => state.refreshActivity);

  return (
    <Panel
      title="Activity history"
      actions={
        <button
          className="rounded-md border border-zinc-800 bg-zinc-950 px-3 py-1 text-xs text-zinc-300 transition hover:border-zinc-600 hover:bg-zinc-900"
          onClick={() => void refreshActivity()}
          type="button"
        >
          {busy.activity ? "Refreshing" : "Refresh"}
        </button>
      }
    >
      <div className="space-y-3">
        {activityRecords.map((record) => (
          <div key={record.id} className="rounded-md border border-zinc-800 bg-zinc-950/60 p-3">
            <div className="flex items-center justify-between gap-3">
              <div>
                <p className="text-sm font-semibold text-zinc-100">{record.commandName}</p>
                <p className="text-xs text-zinc-500">{record.summary}</p>
              </div>
              <StatusBadge tone={record.status === "failed" ? "error" : record.status === "partial" ? "warn" : "ok"}>
                {record.status}
              </StatusBadge>
            </div>
            <div className="mt-3 grid gap-2 text-xs text-zinc-400 md:grid-cols-3">
              <span>{record.statePath}</span>
              <span>{record.durationMs} ms</span>
              <span>{new Date(record.finishedAt).toLocaleString()}</span>
            </div>
          </div>
        ))}
        {activityRecords.length === 0 ? (
          <p className="text-sm leading-6 text-zinc-500">No activity yet.</p>
        ) : null}
      </div>
    </Panel>
  );
}
