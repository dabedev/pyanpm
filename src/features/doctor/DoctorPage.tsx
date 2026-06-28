import { ShieldAlert } from "lucide-react";

import { Button } from "@/components/ui/Button";
import { Panel } from "@/components/ui/Panel";
import { StatusBadge } from "@/components/ui/StatusBadge";
import { useDesktopStore } from "@/store/useDesktopStore";
import { doctorTone } from "@/utils/status";

export function DoctorPage() {
  const doctorReport = useDesktopStore((state) => state.doctorReport);
  const busy = useDesktopStore((state) => state.busy);
  const runDoctor = useDesktopStore((state) => state.runDoctor);

  return (
    <div className="space-y-4">
      <Panel
        title="Doctor report"
        actions={
          <Button variant="primary" onClick={() => void runDoctor()} disabled={busy.doctor}>
            Run doctor
          </Button>
        }
      >
        <div className="flex items-center gap-3 rounded-md border border-zinc-800 bg-zinc-950/70 px-4 py-3">
          <ShieldAlert className={`h-4 w-4 ${doctorReport?.healthy ? "text-teal-300" : "text-amber-300"}`} />
          <div>
            <p className="text-sm font-semibold text-zinc-100">
              {doctorReport?.healthy ? "Managed state healthy" : "Managed state needs attention"}
            </p>
            <p className="mt-1 text-xs text-zinc-500">
              {doctorReport
                ? `${doctorReport.findings.length} finding(s)`
                : "Run doctor to check the manifest, paths, and installed plugins."}
            </p>
          </div>
        </div>
      </Panel>

      <Panel title="Findings">
        <div className="space-y-3">
          {doctorReport?.findings.map((finding) => (
            <div key={`${finding.code}-${finding.message}`} className="rounded-md border border-zinc-800 bg-zinc-950/60 p-4">
              <div className="flex items-center justify-between gap-3">
                <p className="font-mono text-xs text-zinc-400">{finding.code}</p>
                <StatusBadge tone={doctorTone(finding.level)}>{finding.level}</StatusBadge>
              </div>
              <p className="mt-3 text-sm leading-6 text-zinc-200">{finding.message}</p>
            </div>
          )) ?? <p className="text-sm text-zinc-500">No doctor report loaded yet.</p>}
        </div>
      </Panel>
    </div>
  );
}
