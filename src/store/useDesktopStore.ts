import { create } from "zustand";

import type {
  ActivityEntry,
  ActivityRecord,
  AddPluginInput,
  CacheEntry,
  DesktopSettings,
  DoctorReport,
  PluginStateSnapshot,
  ListedPlugin,
  ValidateSourceInput,
  ValidateSourceResult,
} from "@/types/desktop";
import { desktopApi } from "@/utils/desktopApi";
import { sendCompletionNotification } from "@/utils/nativeDesktop";

type BusyKey = "bootstrap" | "settings" | "plugins" | "doctor" | "activity" | "cache";

type DesktopStore = {
  settings: DesktopSettings | null;
  plugins: ListedPlugin[];
  doctorReport: DoctorReport | null;
  diffSnapshots: PluginStateSnapshot[];
  validation: ValidateSourceResult | null;
  activityRecords: ActivityRecord[];
  cacheEntries: CacheEntry[];
  selectedPluginName: string | null;
  activity: ActivityEntry[];
  errorMessage: string | null;
  busy: Record<BusyKey, boolean>;
  bootstrap: () => Promise<void>;
  saveSettings: (stateDirOverride: string, pluginsDirOverride: string, notificationsEnabled: boolean) => Promise<void>;
  initManifest: (force?: boolean) => Promise<void>;
  refreshPlugins: () => Promise<void>;
  refreshDiff: () => Promise<void>;
  addPlugin: (input: AddPluginInput) => Promise<void>;
  installPlugins: () => Promise<void>;
  validateSource: (input: ValidateSourceInput) => Promise<void>;
  removePlugin: (pluginName: string, keepManifest?: boolean) => Promise<void>;
  reinstallPlugin: (pluginName: string) => Promise<void>;
  updatePlugins: (pluginName?: string, all?: boolean, dryRun?: boolean) => Promise<void>;
  runDoctor: () => Promise<void>;
  refreshActivity: () => Promise<void>;
  refreshCache: () => Promise<void>;
  evictCache: (cacheId: string) => Promise<void>;
  pruneCache: () => Promise<void>;
  selectPlugin: (pluginName: string | null) => void;
};

function buildActivity(title: string, detail: string, tone: ActivityEntry["tone"]): ActivityEntry {
  return {
    id: `${Date.now()}-${Math.random().toString(16).slice(2)}`,
    title,
    detail,
    tone,
    timestamp: new Date().toLocaleTimeString(),
  };
}

async function notifyDesktop(settings: DesktopSettings | null, title: string, detail: string) {
  await sendCompletionNotification(settings, title, detail);
}

export const useDesktopStore = create<DesktopStore>((set, get) => ({
  settings: null,
  plugins: [],
  doctorReport: null,
  diffSnapshots: [],
  validation: null,
  activityRecords: [],
  cacheEntries: [],
  selectedPluginName: null,
  activity: [buildActivity("pyanPM ready", "Ready.", "info")],
  errorMessage: null,
  busy: {
    bootstrap: false,
    settings: false,
    plugins: false,
    doctor: false,
    activity: false,
    cache: false,
  },

  async bootstrap() {
    set((state) => ({
      busy: { ...state.busy, bootstrap: true },
      errorMessage: null,
    }));
    try {
      const [settings, listResult, doctorResult, diffResult, activityResult, cacheResult] = await Promise.all([
        desktopApi.getSettings(),
        desktopApi.listPlugins(),
        desktopApi.runDoctor(),
        desktopApi.diffPlugins(),
        desktopApi.listActivity(),
        desktopApi.listCache(),
      ]);

      set((state) => ({
        settings,
        plugins: listResult.plugins,
        doctorReport: doctorResult.report,
        diffSnapshots: diffResult.snapshots,
        activityRecords: activityResult.records,
        cacheEntries: cacheResult.entries,
        selectedPluginName:
          state.selectedPluginName && listResult.plugins.some((plugin) => plugin.name === state.selectedPluginName)
            ? state.selectedPluginName
            : listResult.plugins[0]?.name ?? null,
      }));
    } catch (error) {
      set({
        errorMessage: error instanceof Error ? error.message : "Failed to load pyanPM state.",
      });
    } finally {
      set((state) => ({
        busy: { ...state.busy, bootstrap: false },
      }));
    }
  },

  async saveSettings(stateDirOverride, pluginsDirOverride, notificationsEnabled) {
    const lastRoute = get().settings?.lastRoute ?? "/";
    set((state) => ({
      busy: { ...state.busy, settings: true },
      errorMessage: null,
    }));
    try {
      const settings = await desktopApi.saveSettings({
        stateDirOverride,
        pluginsDirOverride,
        lastRoute,
        notificationsEnabled,
      });
      set((state) => ({
        settings,
        activity: [
          buildActivity("Settings saved", "pyanPM paths were updated successfully.", "success"),
          ...state.activity,
        ].slice(0, 6),
      }));
      await Promise.all([get().refreshPlugins(), get().refreshDiff(), get().runDoctor()]);
    } catch (error) {
      set({
        errorMessage: error instanceof Error ? error.message : "Failed to save desktop settings.",
      });
    } finally {
      set((state) => ({
        busy: { ...state.busy, settings: false },
      }));
    }
  },

  async initManifest(force = false) {
    set((state) => ({
      busy: { ...state.busy, plugins: true },
      errorMessage: null,
    }));
    try {
      const result = await desktopApi.initManifest(force);
      set((state) => ({
        activity: [
          buildActivity("Manifest initialized", `Created manifest at ${result.manifestPath}.`, "success"),
          ...state.activity,
        ].slice(0, 6),
      }));
      await Promise.all([get().refreshPlugins(), get().runDoctor()]);
    } catch (error) {
      set({
        errorMessage: error instanceof Error ? error.message : "Failed to initialize the global manifest.",
      });
    } finally {
      set((state) => ({
        busy: { ...state.busy, plugins: false },
      }));
    }
  },

  async refreshPlugins() {
    set((state) => ({
      busy: { ...state.busy, plugins: true },
      errorMessage: null,
    }));
    try {
      const result = await desktopApi.listPlugins();
      set((state) => ({
        plugins: result.plugins,
        selectedPluginName:
          state.selectedPluginName && result.plugins.some((plugin) => plugin.name === state.selectedPluginName)
            ? state.selectedPluginName
            : result.plugins[0]?.name ?? null,
      }));
    } catch (error) {
      set({
        errorMessage: error instanceof Error ? error.message : "Failed to load managed plugins.",
      });
    } finally {
      set((state) => ({
        busy: { ...state.busy, plugins: false },
      }));
    }
  },

  async refreshDiff() {
    try {
      const result = await desktopApi.diffPlugins();
      set({ diffSnapshots: result.snapshots });
    } catch (error) {
      set({
        errorMessage: error instanceof Error ? error.message : "Failed to load plugin diff state.",
      });
    }
  },

  async addPlugin(input) {
    set((state) => ({
      busy: { ...state.busy, plugins: true },
      errorMessage: null,
    }));
    try {
      const result = await desktopApi.addPlugin(input);
      await notifyDesktop(get().settings, "pyanPM install complete", result.completion.summary);
      set((state) => ({
        activity: [
          buildActivity(
            "Plugin added",
            `Installed ${result.installed.length} plugin(s) into ${result.pluginsDir}.`,
            "success"
          ),
          ...state.activity,
        ].slice(0, 6),
        validation: null,
      }));
      await Promise.all([get().refreshPlugins(), get().refreshDiff(), get().refreshActivity(), get().runDoctor()]);
    } catch (error) {
      await notifyDesktop(get().settings, "pyanPM install failed", error instanceof Error ? error.message : "Add plugin failed.");
      set({
        errorMessage: error instanceof Error ? error.message : "Failed to add plugin source.",
      });
    } finally {
      set((state) => ({
        busy: { ...state.busy, plugins: false },
      }));
    }
  },

  async installPlugins() {
    set((state) => ({
      busy: { ...state.busy, plugins: true },
      errorMessage: null,
    }));
    try {
      const result = await desktopApi.installPlugins();
      await notifyDesktop(get().settings, "pyanPM install complete", result.completion.summary);
      set((state) => ({
        activity: [
          buildActivity(
            "Install complete",
            `Installed ${result.installed.length} plugin(s) into ${result.pluginsDir}.`,
            "success"
          ),
          ...state.activity,
        ].slice(0, 6),
      }));
      await Promise.all([get().refreshPlugins(), get().refreshDiff(), get().refreshActivity(), get().runDoctor()]);
    } catch (error) {
      await notifyDesktop(get().settings, "pyanPM install failed", error instanceof Error ? error.message : "Install failed.");
      set({
        errorMessage: error instanceof Error ? error.message : "Failed to install managed plugins.",
      });
    } finally {
      set((state) => ({
        busy: { ...state.busy, plugins: false },
      }));
    }
  },

  async validateSource(input) {
    try {
      const result = await desktopApi.validateSource(input);
      set({ validation: result });
    } catch (error) {
      set({
        errorMessage: error instanceof Error ? error.message : "Failed to validate the source reference.",
      });
    }
  },

  async removePlugin(pluginName, keepManifest = false) {
    set((state) => ({
      busy: { ...state.busy, plugins: true },
      errorMessage: null,
    }));
    try {
      const result = await desktopApi.removePlugin(pluginName, keepManifest);
      await notifyDesktop(get().settings, "pyanPM remove complete", result.completion.summary);
      set((state) => ({
        activity: [
          buildActivity("Plugin removed", result.completion.summary, "warning"),
          ...state.activity,
        ].slice(0, 6),
      }));
      await Promise.all([get().refreshPlugins(), get().refreshDiff(), get().refreshActivity(), get().runDoctor()]);
    } catch (error) {
      await notifyDesktop(get().settings, "pyanPM remove failed", error instanceof Error ? error.message : "Remove failed.");
      set({
        errorMessage: error instanceof Error ? error.message : "Failed to remove the managed plugin.",
      });
    } finally {
      set((state) => ({
        busy: { ...state.busy, plugins: false },
      }));
    }
  },

  async reinstallPlugin(pluginName) {
    set((state) => ({
      busy: { ...state.busy, plugins: true },
      errorMessage: null,
    }));
    try {
      const result = await desktopApi.reinstallPlugin(pluginName);
      await notifyDesktop(get().settings, "pyanPM reinstall complete", result.completion.summary);
      set((state) => ({
        activity: [
          buildActivity("Plugin reinstalled", result.completion.summary, "success"),
          ...state.activity,
        ].slice(0, 6),
      }));
      await Promise.all([get().refreshPlugins(), get().refreshDiff(), get().refreshActivity(), get().runDoctor()]);
    } catch (error) {
      await notifyDesktop(get().settings, "pyanPM reinstall failed", error instanceof Error ? error.message : "Reinstall failed.");
      set({
        errorMessage: error instanceof Error ? error.message : "Failed to reinstall the managed plugin.",
      });
    } finally {
      set((state) => ({
        busy: { ...state.busy, plugins: false },
      }));
    }
  },

  async updatePlugins(pluginName, all = false, dryRun = false) {
    set((state) => ({
      busy: { ...state.busy, plugins: true },
      errorMessage: null,
    }));
    try {
      const result = await desktopApi.updatePlugins(pluginName, all, dryRun);
      await notifyDesktop(
        get().settings,
        dryRun ? "pyanPM update preview ready" : "pyanPM update complete",
        result.completion.summary
      );
      set((state) => ({
        activity: [
          buildActivity(dryRun ? "Update preview" : "Update completed", result.completion.summary, "info"),
          ...state.activity,
        ].slice(0, 6),
      }));
      await Promise.all([get().refreshPlugins(), get().refreshDiff(), get().refreshActivity(), get().runDoctor()]);
    } catch (error) {
      await notifyDesktop(get().settings, "pyanPM update failed", error instanceof Error ? error.message : "Update failed.");
      set({
        errorMessage: error instanceof Error ? error.message : "Failed to update managed plugins.",
      });
    } finally {
      set((state) => ({
        busy: { ...state.busy, plugins: false },
      }));
    }
  },

  async runDoctor() {
    set((state) => ({
      busy: { ...state.busy, doctor: true },
      errorMessage: null,
    }));
    try {
      const result = await desktopApi.runDoctor();
      await notifyDesktop(get().settings, "pyanPM doctor complete", result.completion.summary);
      set((state) => ({
        doctorReport: result.report,
        activity: [
          buildActivity(
            "Doctor completed",
            result.report.healthy ? "Managed plugins are healthy." : "Doctor found issues that need attention.",
            result.report.healthy ? "success" : "warning"
          ),
          ...state.activity,
        ].slice(0, 6),
      }));
    } catch (error) {
      await notifyDesktop(get().settings, "pyanPM doctor failed", error instanceof Error ? error.message : "Doctor failed.");
      set({
        errorMessage: error instanceof Error ? error.message : "Failed to run doctor.",
      });
    } finally {
      set((state) => ({
        busy: { ...state.busy, doctor: false },
      }));
    }
  },

  async refreshActivity() {
    set((state) => ({
      busy: { ...state.busy, activity: true },
    }));
    try {
      const result = await desktopApi.listActivity();
      set({ activityRecords: result.records });
    } catch (error) {
      set({
        errorMessage: error instanceof Error ? error.message : "Failed to load activity history.",
      });
    } finally {
      set((state) => ({
        busy: { ...state.busy, activity: false },
      }));
    }
  },

  async refreshCache() {
    set((state) => ({
      busy: { ...state.busy, cache: true },
    }));
    try {
      const result = await desktopApi.listCache();
      set({ cacheEntries: result.entries });
    } catch (error) {
      set({
        errorMessage: error instanceof Error ? error.message : "Failed to load cache entries.",
      });
    } finally {
      set((state) => ({
        busy: { ...state.busy, cache: false },
      }));
    }
  },

  async evictCache(cacheId) {
    await desktopApi.evictCache(cacheId);
    await Promise.all([get().refreshCache(), get().refreshActivity()]);
  },

  async pruneCache() {
    await desktopApi.pruneCache();
    await Promise.all([get().refreshCache(), get().refreshActivity()]);
  },

  selectPlugin(pluginName) {
    set({ selectedPluginName: pluginName });
  },
}));
