import { invoke } from "@tauri-apps/api/core";

import type {
  ActivityListResult,
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
import { mockDesktopApi } from "@/utils/mockDesktopApi";

function inTauriRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export const desktopApi = {
  getSettings(): Promise<DesktopSettings> {
    return inTauriRuntime()
      ? invoke<DesktopSettings>("get_settings")
      : mockDesktopApi.getSettings();
  },

  saveSettings(input: SaveSettingsInput): Promise<DesktopSettings> {
    return inTauriRuntime()
      ? invoke<DesktopSettings>("save_settings", { input })
      : mockDesktopApi.saveSettings(input);
  },

  initManifest(force = false): Promise<InitResult> {
    return inTauriRuntime()
      ? invoke<InitResult>("init_manifest", { force })
      : mockDesktopApi.initManifest();
  },

  listPlugins(): Promise<ListResult> {
    return inTauriRuntime()
      ? invoke<ListResult>("list_plugins")
      : mockDesktopApi.listPlugins();
  },

  addPlugin(input: AddPluginInput): Promise<InstallResult> {
    return inTauriRuntime()
      ? invoke<InstallResult>("add_plugin", { input })
      : mockDesktopApi.addPlugin(input);
  },

  installPlugins(): Promise<InstallResult> {
    return inTauriRuntime()
      ? invoke<InstallResult>("install_plugins")
      : mockDesktopApi.installPlugins();
  },

  runDoctor(): Promise<DoctorResult> {
    return inTauriRuntime()
      ? invoke<DoctorResult>("run_doctor")
      : mockDesktopApi.runDoctor();
  },

  validateSource(input: ValidateSourceInput): Promise<ValidateSourceResult> {
    return inTauriRuntime()
      ? invoke<ValidateSourceResult>("validate_source", { input })
      : mockDesktopApi.validateSource(input);
  },

  diffPlugins(): Promise<DiffResult> {
    return inTauriRuntime()
      ? invoke<DiffResult>("diff_plugins")
      : mockDesktopApi.diffPlugins();
  },

  removePlugin(pluginName: string, keepManifest = false): Promise<RemoveResult> {
    return inTauriRuntime()
      ? invoke<RemoveResult>("remove_plugin", { pluginName, keepManifest })
      : mockDesktopApi.removePlugin(pluginName, keepManifest);
  },

  reinstallPlugin(pluginName: string): Promise<InstallResult> {
    return inTauriRuntime()
      ? invoke<InstallResult>("reinstall_plugin", { pluginName })
      : mockDesktopApi.reinstallPlugin(pluginName);
  },

  updatePlugins(pluginName?: string, all = false, dryRun = false): Promise<UpdateResult> {
    return inTauriRuntime()
      ? invoke<UpdateResult>("update_plugins", { pluginName, all, dryRun })
      : mockDesktopApi.updatePlugins(pluginName, all, dryRun);
  },

  listActivity(): Promise<ActivityListResult> {
    return inTauriRuntime()
      ? invoke<ActivityListResult>("list_activity")
      : mockDesktopApi.listActivity();
  },

  listCache(): Promise<CacheListResult> {
    return inTauriRuntime()
      ? invoke<CacheListResult>("list_cache")
      : mockDesktopApi.listCache();
  },

  evictCache(cacheId: string): Promise<CacheMutationResult> {
    return inTauriRuntime()
      ? invoke<CacheMutationResult>("evict_cache", { cacheId })
      : mockDesktopApi.evictCache(cacheId);
  },

  pruneCache(): Promise<CacheMutationResult> {
    return inTauriRuntime()
      ? invoke<CacheMutationResult>("prune_cache")
      : mockDesktopApi.pruneCache();
  },
};
