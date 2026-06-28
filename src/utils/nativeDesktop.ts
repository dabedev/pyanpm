import type { DesktopSettings } from "@/types/desktop";

function inTauriRuntime() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function windowIsBackgrounded() {
  if (typeof document === "undefined") {
    return false;
  }

  return document.visibilityState === "hidden" || !document.hasFocus();
}

export async function pickPluginSourcePath(kind: "file" | "path"): Promise<string | null> {
  if (!inTauriRuntime()) {
    return null;
  }

  const { open } = await import("@tauri-apps/plugin-dialog");
  const selected = await open({
    directory: kind === "path",
    multiple: false,
    filters:
      kind === "file"
        ? [
            {
              name: "Roblox plugin models",
              extensions: ["rbxm", "rbxmx"],
            },
          ]
        : undefined,
  });

  return Array.isArray(selected) ? selected[0] ?? null : selected;
}

export async function pickDirectoryPath(defaultPath?: string): Promise<string | null> {
  if (!inTauriRuntime()) {
    return null;
  }

  const { open } = await import("@tauri-apps/plugin-dialog");
  const selected = await open({
    directory: true,
    multiple: false,
    defaultPath: defaultPath || undefined,
  });

  return Array.isArray(selected) ? selected[0] ?? null : selected;
}

export async function sendCompletionNotification(
  settings: DesktopSettings | null,
  title: string,
  body: string
): Promise<void> {
  if (!inTauriRuntime() || !settings?.notificationsEnabled || !windowIsBackgrounded()) {
    return;
  }

  const { isPermissionGranted, requestPermission, sendNotification } = await import("@tauri-apps/plugin-notification");
  const granted = (await isPermissionGranted()) || (await requestPermission()) === "granted";
  if (!granted) {
    return;
  }

  await sendNotification({ title, body });
}
