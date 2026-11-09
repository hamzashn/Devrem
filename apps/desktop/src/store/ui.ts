import { create } from "zustand";

/** Geçici arayüz durumu (kaydedilmez). */
export const useUi = create<{ galleryOpen: boolean }>()(() => ({ galleryOpen: false }));

export const openGallery = () => useUi.setState({ galleryOpen: true });
export const closeGallery = () => useUi.setState({ galleryOpen: false });
