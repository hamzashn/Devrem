/**
 * Otomatik güncelleme. Açılışta GitHub Releases'taki `latest.json` kontrol
 * edilir; yeni sürüm varsa durum çubuğunda bildirim gösterilir. Geliştirme
 * modunda ve tarayıcıda kontrol yapılmaz.
 */
import { relaunch } from "@tauri-apps/plugin-process";
import { check, type Update } from "@tauri-apps/plugin-updater";
import { create } from "zustand";
import i18n from "../i18n";
import { showNotice } from "../store/notice";
import { isTauri } from "./ipc";

interface UpdateState {
  update: Update | null;
  installing: boolean;
}

export const useUpdate = create<UpdateState>()(() => ({ update: null, installing: false }));

export async function checkForUpdates() {
  if (!isTauri() || import.meta.env.DEV) return;
  try {
    const update = await check();
    if (update) useUpdate.setState({ update });
  } catch (e) {
    // Depo henüz yayımlanmamışsa veya çevrimdışıysak sessizce geç.
    console.info("Güncelleme kontrolü yapılamadı:", e);
  }
}

export async function installUpdate() {
  const { update } = useUpdate.getState();
  if (!update) return;
  useUpdate.setState({ installing: true });
  try {
    await update.downloadAndInstall();
    await relaunch();
  } catch (e) {
    useUpdate.setState({ installing: false });
    showNotice("error", i18n.t("update.failed", { message: String(e) }));
  }
}
