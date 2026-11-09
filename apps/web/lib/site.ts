/**
 * Site ayarları. GitHub deposu açıldığında `GITHUB_REPO` değerini
 * `kullanıcı/depo` olarak güncelle (masaüstü uygulamasındaki güncelleme
 * adresiyle aynı olmalı: apps/desktop/src-tauri/tauri.conf.json).
 */
export const GITHUB_REPO = "hamzashn/Devrem";
export const VERSION = "0.1.0";

const releases = `https://github.com/${GITHUB_REPO}/releases`;

export const LINKS = {
  releases,
  installer: `${releases}/latest/download/Devrem_${VERSION}_x64-setup.exe`,
  msi: `${releases}/latest/download/Devrem_${VERSION}_x64_en-US.msi`,
};
