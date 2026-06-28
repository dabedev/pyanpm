import type { DoctorFinding, ListedPlugin } from "@/types/desktop";

export function pluginTone(status: ListedPlugin["status"]): "ok" | "warn" | "error" | "neutral" {
  switch (status) {
    case "installed":
      return "ok";
    case "missing":
      return "error";
    case "manifest-only":
      return "warn";
    default:
      return "neutral";
  }
}

export function doctorTone(level: DoctorFinding["level"]): "ok" | "warn" | "error" | "neutral" {
  if (level === "ok") {
    return "ok";
  }

  if (level === "warn") {
    return "warn";
  }

  if (level === "error") {
    return "error";
  }

  return "neutral";
}
