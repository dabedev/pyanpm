import { HashRouter, Route, Routes } from "react-router-dom";

import { AppShell } from "@/components/layout/AppShell";
import { ActivityPage } from "@/features/activity/ActivityPage";
import { CachePage } from "@/features/cache/CachePage";
import { DoctorPage } from "@/features/doctor/DoctorPage";
import { OverviewPage } from "@/features/overview/OverviewPage";
import { PluginsPage } from "@/features/plugins/PluginsPage";
import { SettingsPage } from "@/features/settings/SettingsPage";

export function AppRoutes() {
  return (
    <HashRouter>
      <AppShell>
        <Routes>
          <Route path="/" element={<OverviewPage />} />
          <Route path="/activity" element={<ActivityPage />} />
          <Route path="/plugins" element={<PluginsPage />} />
          <Route path="/cache" element={<CachePage />} />
          <Route path="/doctor" element={<DoctorPage />} />
          <Route path="/settings" element={<SettingsPage />} />
        </Routes>
      </AppShell>
    </HashRouter>
  );
}
