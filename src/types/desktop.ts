export type DesktopSettings = {
  stateDir: string;
  defaultStateDir?: string | null;
  pluginsDirOverride?: string | null;
  detectedPluginsDir?: string | null;
  lastRoute: string;
  notificationsEnabled: boolean;
};

export type CommandMeta = {
  commandId: string;
  statePath: string;
  activityId?: string | null;
};

export type CompletionSummary = {
  affectedPluginCount: number;
  warningCount: number;
  errorCount: number;
  durationMs: number;
  summary: string;
};

export type ProgressEvent = {
  commandId: string;
  statePath: string;
  pluginName?: string | null;
  stage: string;
  message: string;
  current?: number | null;
  total?: number | null;
  severity: "info" | "warn" | "error";
  timestamp: string;
};

export type ListedPlugin = {
  name: string;
  requestedVersion?: string | null;
  installedVersion?: string | null;
  source: string;
  status: string;
  targetPath?: string | null;
};

export type ListResult = {
  plugins: ListedPlugin[];
};

export type InstalledPluginSummary = {
  name: string;
  source: string;
  checksum: string;
  targetPath: string;
};

export type InstallResult = {
  meta: CommandMeta;
  installed: InstalledPluginSummary[];
  lockfilePath: string;
  pluginsDir: string;
  progress: ProgressEvent[];
  completion: CompletionSummary;
};

export type InitResult = {
  meta: CommandMeta;
  manifestPath: string;
  completion: CompletionSummary;
};

export type DoctorFinding = {
  level: "ok" | "warn" | "error";
  code: string;
  message: string;
};

export type DoctorReport = {
  healthy: boolean;
  findings: DoctorFinding[];
};

export type DoctorResult = {
  meta: CommandMeta;
  report: DoctorReport;
  completion: CompletionSummary;
};

export type ValidationIssue = {
  field: string;
  sourceRef: string;
  code: string;
  severity: "info" | "warning" | "error";
  message: string;
  suggestedFix?: string | null;
};

export type ValidateSourceResult = {
  meta: CommandMeta;
  valid: boolean;
  normalizedSourceRef?: string | null;
  pluginName?: string | null;
  errors: ValidationIssue[];
};

export type AddPluginInput = {
  pluginRef: string;
  version?: string;
  gitRefKind?: "branch" | "tag" | "commit";
  gitRef?: string;
  gitSubdir?: string;
};

export type ValidateSourceInput = {
  sourceRef: string;
  gitRefKind?: "branch" | "tag" | "commit";
  gitRef?: string;
  gitSubdir?: string;
};

export type SaveSettingsInput = {
  stateDirOverride?: string;
  pluginsDirOverride?: string;
  lastRoute: string;
  notificationsEnabled: boolean;
};

export type ActivityEntry = {
  id: string;
  title: string;
  detail: string;
  timestamp: string;
  tone: "info" | "success" | "warning";
};

export type ActivityRecord = {
  id: string;
  commandId: string;
  statePath: string;
  commandName: string;
  commandArgsSummary: string;
  startedAt: string;
  finishedAt: string;
  durationMs: number;
  status: "success" | "failed" | "partial";
  pluginNames: string[];
  summary: string;
  errorCode?: string | null;
  errorMessage?: string | null;
  errorStage?: string | null;
};

export type ActivityListResult = {
  meta: CommandMeta;
  records: ActivityRecord[];
};

export type PluginStateSnapshot = {
  pluginName: string;
  manifestSource?: string | null;
  manifestVersionConstraint?: string | null;
  lockVersion?: string | null;
  lockChecksum?: string | null;
  installedPath?: string | null;
  installedChecksum?: string | null;
  state:
    | "synced"
    | "manifest_only"
    | "lock_only"
    | "missing_install"
    | "stale_install"
    | "checksum_drift"
    | "source_unreadable"
    | "unmanaged_installed_plugin"
    | "target_write_risk";
  recommendedAction: string;
  lastActivityId?: string | null;
};

export type DiffResult = {
  meta: CommandMeta;
  snapshots: PluginStateSnapshot[];
};

export type RemovedPluginSummary = {
  name: string;
  keepManifest: boolean;
  removedManifestEntry: boolean;
  removedTargetPath?: string | null;
};

export type RemoveResult = {
  meta: CommandMeta;
  removed: RemovedPluginSummary;
  progress: ProgressEvent[];
  completion: CompletionSummary;
};

export type UpdateCandidate = {
  pluginName: string;
  currentVersion?: string | null;
  candidateVersion?: string | null;
  currentChecksum?: string | null;
  candidateChecksum: string;
  willChange: boolean;
  reason: string;
};

export type UpdateResult = {
  meta: CommandMeta;
  dryRun: boolean;
  candidates: UpdateCandidate[];
  updated: InstalledPluginSummary[];
  progress: ProgressEvent[];
  completion: CompletionSummary;
};

export type CacheEntry = {
  cacheId: string;
  pluginName: string;
  version?: string | null;
  sourceKind: string;
  sourceSummary: string;
  checksum: string;
  sizeBytes: number;
  createdAt: string;
  lastUsedAt: string;
  referencedByActiveLockfile: boolean;
  path: string;
};

export type CacheListResult = {
  meta: CommandMeta;
  totalSizeBytes: number;
  entries: CacheEntry[];
};

export type CacheMutationResult = {
  meta: CommandMeta;
  removed: string[];
  protected: string[];
  reclaimedSizeBytes: number;
  completion: CompletionSummary;
};
