import type {
  ActivityListResult,
  ActivityRecord,
  AddPluginInput,
  CacheListResult,
  CacheMutationResult,
  DesktopSettings,
  DiffResult,
  DoctorResult,
  InitResult,
  InstallResult,
  ListResult,
  RemoveResult,
  SaveSettingsInput,
  UpdateResult,
  ValidateSourceInput,
  ValidateSourceResult,
} from "@/types/desktop";

let settings: DesktopSettings = {
  stateDir: "C:\\Users\\Example\\AppData\\Roaming\\pyanpm\\state",
  defaultStateDir: "C:\\Users\\Example\\AppData\\Roaming\\pyanpm\\state",
  pluginsDirOverride: "C:\\Users\\Example\\AppData\\Local\\Roblox\\Plugins",
  detectedPluginsDir: "C:\\Users\\Example\\AppData\\Local\\Roblox\\Plugins",
  lastRoute: "/",
  notificationsEnabled: true,
};

let manifestInitialized = false;
let plugins: ListResult["plugins"] = [];
let activityRecords: ActivityRecord[] = [];
let cacheEntries: CacheListResult["entries"] = [];

function delay() {
  return new Promise((resolve) => window.setTimeout(resolve, 120));
}

export const mockDesktopApi = {
  async getSettings(): Promise<DesktopSettings> {
    await delay();
    return settings;
  },

  async saveSettings(input: SaveSettingsInput): Promise<DesktopSettings> {
    await delay();
    settings = {
      stateDir: input.stateDirOverride ?? settings.defaultStateDir ?? settings.stateDir,
      defaultStateDir: settings.defaultStateDir ?? settings.stateDir,
      pluginsDirOverride: input.pluginsDirOverride ?? settings.detectedPluginsDir ?? null,
      detectedPluginsDir: settings.detectedPluginsDir ?? "C:\\Users\\Example\\AppData\\Local\\Roblox\\Plugins",
      lastRoute: input.lastRoute,
      notificationsEnabled: input.notificationsEnabled,
    };
    return settings;
  },

  async initManifest(): Promise<InitResult> {
    await delay();
    manifestInitialized = true;
    return {
      meta: meta(),
      manifestPath: `${settings.stateDir}\\pyanpm.toml`,
      completion: completion("Initialized the global pyanPM manifest."),
    };
  },

  async listPlugins(): Promise<ListResult> {
    await delay();
    return { plugins };
  },

  async addPlugin(input: AddPluginInput): Promise<InstallResult> {
    await delay();
    manifestInitialized = true;
    const normalized = input.pluginRef.replace(/^file:|^path:|^git:/, "");
    const guessedName =
      input.pluginRef.startsWith("git:")
        ? normalized.split(/[/:\\/]/).pop()?.replace(/\.git$/i, "") || "git-plugin"
        : normalized.split(/[\\/]/).pop()?.replace(/\.(rbxm|rbxmx)$/i, "") || "plugin";
    const targetPath = `${settings.pluginsDirOverride ?? "C:\\Roblox\\Plugins"}\\${guessedName}.rbxm`;

    plugins = [
      {
        name: guessedName,
        requestedVersion: input.version ?? null,
        installedVersion: input.version ?? "local",
        source: input.pluginRef,
        status: "installed",
        targetPath,
      },
      ...plugins.filter((plugin) => plugin.name !== guessedName),
    ];

    return {
      meta: meta(),
      installed: [
        {
          name: guessedName,
          source: input.pluginRef,
          checksum: "mock-checksum",
          targetPath,
        },
      ],
      lockfilePath: `${settings.stateDir}\\pyanpm.lock`,
      pluginsDir: settings.pluginsDirOverride ?? "C:\\Roblox\\Plugins",
      progress: [],
      completion: completion(`Installed ${guessedName}.`),
    };
  },

  async installPlugins(): Promise<InstallResult> {
    await delay();
    const installed = plugins.map((plugin) => ({
      name: plugin.name,
      source: plugin.source,
      checksum: "mock-checksum",
      targetPath: plugin.targetPath ?? `${settings.pluginsDirOverride ?? "C:\\Roblox\\Plugins"}\\${plugin.name}.rbxm`,
    }));
    plugins = plugins.map((plugin) => ({ ...plugin, status: "installed" }));

    return {
      meta: meta(),
      installed,
      lockfilePath: `${settings.stateDir}\\pyanpm.lock`,
      pluginsDir: settings.pluginsDirOverride ?? "C:\\Roblox\\Plugins",
      progress: [],
      completion: completion(`Installed ${installed.length} plugin(s).`),
    };
  },

  async runDoctor(): Promise<DoctorResult> {
    await delay();
    return {
      meta: meta(),
      report: {
        healthy: manifestInitialized,
        findings: manifestInitialized
          ? [
              {
                level: "ok",
                code: "manifest",
                message: "Manifest parsed successfully.",
              },
              {
                level: "ok",
                code: "plugins_dir.access",
                message: `Studio plugins directory is accessible at ${
                  settings.pluginsDirOverride ?? "C:\\Roblox\\Plugins"
                }.`,
              },
            ]
          : [
              {
                level: "error",
                code: "manifest.parse",
                message: "Manifest is missing. Run init to create pyanpm.toml.",
              },
            ],
      },
      completion: completion("Doctor completed."),
    };
  },

  async validateSource(input: ValidateSourceInput): Promise<ValidateSourceResult> {
    await delay();
    const sourceRef = input.sourceRef;
    const gitValid =
      sourceRef.startsWith("git:") &&
      sourceRef.length > 4 &&
      (!input.gitRefKind || Boolean(input.gitRef?.trim())) &&
      (!input.gitRef || Boolean(input.gitRefKind));
    const valid = sourceRef.startsWith("file:") || sourceRef.startsWith("path:") || gitValid;
    return {
      meta: meta(),
      valid,
      normalizedSourceRef: valid ? sourceRef : null,
      pluginName: valid
        ? sourceRef.startsWith("git:")
          ? sourceRef.replace(/^git:/, "").split(/[/:\\/]/).pop()?.replace(/\.git$/i, "") ?? "git-plugin"
          : sourceRef.split(/[\\/]/).pop()?.replace(/\.(rbxm|rbxmx)$/i, "") ?? "plugin"
        : null,
      errors: valid
        ? []
        : [
            {
              field: "sourceRef",
              sourceRef,
              code: "source.prefix.unsupported",
              severity: "error",
              message: "Use a `file:`, `path:`, or `git:` source.",
              suggestedFix: "Provide a valid source and complete any required Git ref fields.",
            },
          ],
    };
  },

  async diffPlugins(): Promise<DiffResult> {
    await delay();
    return {
      meta: meta(),
      snapshots: plugins.map((plugin) => ({
        pluginName: plugin.name,
        manifestSource: plugin.source,
        manifestVersionConstraint: plugin.requestedVersion,
        lockVersion: plugin.installedVersion,
        lockChecksum: "mock-checksum",
        installedPath: plugin.targetPath,
        installedChecksum: "mock-checksum",
        state: plugin.status === "installed" ? "synced" : "missing_install",
        recommendedAction: plugin.status === "installed" ? "none" : "install",
        lastActivityId: activityRecords[0]?.id ?? null,
      })),
    };
  },

  async removePlugin(pluginName: string, keepManifest: boolean): Promise<RemoveResult> {
    await delay();
    const removedPlugin = plugins.find((plugin) => plugin.name === pluginName);
    plugins = plugins.filter((plugin) => plugin.name !== pluginName);
    return {
      meta: meta(),
      removed: {
        name: pluginName,
        keepManifest,
        removedManifestEntry: !keepManifest,
        removedTargetPath: removedPlugin?.targetPath ?? null,
      },
      progress: [],
      completion: completion(`Removed ${pluginName}.`),
    };
  },

  async reinstallPlugin(pluginName: string): Promise<InstallResult> {
    await delay();
    const plugin = plugins.find((entry) => entry.name === pluginName);
    return {
      meta: meta(),
      installed: plugin
        ? [
            {
              name: plugin.name,
              source: plugin.source,
              checksum: "mock-checksum",
              targetPath: plugin.targetPath ?? `${settings.pluginsDirOverride ?? "C:\\Roblox\\Plugins"}\\${plugin.name}.rbxm`,
            },
          ]
        : [],
      lockfilePath: `${settings.stateDir}\\pyanpm.lock`,
      pluginsDir: settings.pluginsDirOverride ?? "C:\\Roblox\\Plugins",
      progress: [],
      completion: completion(`Reinstalled ${pluginName}.`),
    };
  },

  async updatePlugins(pluginName?: string, all = false, dryRun = false): Promise<UpdateResult> {
    await delay();
    const target = all ? plugins : plugins.filter((plugin) => plugin.name === pluginName);
    return {
      meta: meta(),
      dryRun,
      candidates: target.map((plugin) => ({
        pluginName: plugin.name,
        currentVersion: plugin.installedVersion,
        candidateVersion: plugin.installedVersion,
        currentChecksum: "mock-checksum",
        candidateChecksum: "mock-checksum",
        willChange: false,
        reason: "already up to date",
      })),
      updated: dryRun
        ? []
        : target.map((plugin) => ({
            name: plugin.name,
            source: plugin.source,
            checksum: "mock-checksum",
            targetPath: plugin.targetPath ?? `${settings.pluginsDirOverride ?? "C:\\Roblox\\Plugins"}\\${plugin.name}.rbxm`,
          })),
      progress: [],
      completion: completion(dryRun ? "Computed update candidates." : "Updated managed plugins."),
    };
  },

  async listActivity(): Promise<ActivityListResult> {
    await delay();
    return {
      meta: meta(),
      records: activityRecords,
    };
  },

  async listCache(): Promise<CacheListResult> {
    await delay();
    return {
      meta: meta(),
      totalSizeBytes: cacheEntries.reduce((total, entry) => total + entry.sizeBytes, 0),
      entries: cacheEntries,
    };
  },

  async evictCache(cacheId: string): Promise<CacheMutationResult> {
    await delay();
    cacheEntries = cacheEntries.filter((entry) => entry.cacheId !== cacheId);
    return {
      meta: meta(),
      removed: [cacheId],
      protected: [],
      reclaimedSizeBytes: 0,
      completion: completion(`Evicted cache entry ${cacheId}.`),
    };
  },

  async pruneCache(): Promise<CacheMutationResult> {
    await delay();
    const removed = cacheEntries.map((entry) => entry.cacheId);
    cacheEntries = [];
    return {
      meta: meta(),
      removed,
      protected: [],
      reclaimedSizeBytes: 0,
      completion: completion("Pruned cache entries."),
    };
  },

};

function meta() {
  return {
    commandId: crypto.randomUUID(),
    statePath: settings.stateDir,
    activityId: null,
  };
}

function completion(summary: string) {
  return {
    affectedPluginCount: 0,
    warningCount: 0,
    errorCount: 0,
    durationMs: 120,
    summary,
  };
}
