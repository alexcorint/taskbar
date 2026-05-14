import { writable } from "svelte/store";
import { emit, listen } from "@tauri-apps/api/event";

// Store para el estado de expansión de la tarjeta de medios
export const isExpanded = writable(false);

// Sincronización entre ventanas
if (typeof window !== "undefined") {
  listen<boolean>("store-update-is-expanded", (e) => {
    isExpanded.set(e.payload);
  });
}

export const setIsExpanded = (val: boolean) => {
  isExpanded.set(val);
  emit("store-update-is-expanded", val);
};

// Store para la metadata de medios (para evitar fetch duplicados si es necesario)
export const mediaData = writable({
  title: "Now listening...",
  artist: "",
  album: "",
  app_id: "Generic",
  thumbnail_base64: "",
  is_playing: false,
  position_ms: 0,
  duration_ms: 0,
  slideDirection: "right" as "left" | "right",
});

if (typeof window !== "undefined") {
  listen<any>("store-update-media-data", (e) => {
    mediaData.set(e.payload);
  });
}

export const setMediaData = (val: any) => {
  mediaData.set(val);
  emit("store-update-media-data", val);
};
