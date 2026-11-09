/**
 * Site ayarları. GitHub deposu açıldığında `GITHUB_REPO` değerini
 * `kullanıcı/depo` olarak güncelle (masaüstü uygulamasındaki güncelleme
 * adresiyle aynı olmalı: apps/desktop/src-tauri/tauri.conf.json).
 */
export const GITHUB_REPO = "hamzashn/Devrem";
export const VERSION = "0.1.0";

const repo = `https://github.com/${GITHUB_REPO}`;

export const LINKS = {
  repo,
  releases: `${repo}/releases`,
  /** README'deki ayrıntılı kurulum rehberi. */
  guideTr: `${repo}#-kurulum`,
  guideEn: `${repo}/blob/main/README.en.md#-installation`,
  clone: `${repo}.git`,
};
