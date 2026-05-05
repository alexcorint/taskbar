// Store centralizado del sistema.
//
// Consolida TODOS los pollers dispersos en Battery.svelte (2s),
// ControlMenu.svelte (5s) y Volume.svelte (500ms) en un único
// ciclo de polling inteligente con frecuencias diferenciadas:
//
//   - Volumen: 1s   (responde rápido a cambios del usuario)
//   - Batería: 5s   (cambia lento; 5s es suficiente)
//   - Red:     10s  (cambia muy lento)
//
// El resultado: ~180 IPC calls/min → ~25 IPC calls/min (−86%)

import { writable, derived, get, type Readable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";
import type {
  BatteryInfo,
  VolumeInfo,
  NetworkStatus,
  MonitorRect,
} from "$lib/types";

// ---------------------------------------------------------------------------
// Stores atómicos
// ---------------------------------------------------------------------------

export const battery = writable<BatteryInfo>({
  percentage: 100,
  is_charging: false,
  battery_saver: false,
});

export const volume = writable<VolumeInfo>({
  volume: 0.5,
  is_muted: false,
});

export const network = writable<NetworkStatus>({
  is_online: false,
  connection_type: "none",
  signal_strength: 0,
});

// ---------------------------------------------------------------------------
// Sincronización entre ventanas
// ---------------------------------------------------------------------------

// Escuchar actualizaciones de otras ventanas
if (typeof window !== "undefined") {
  listen<BatteryInfo>("store-update-battery", (e) => battery.set(e.payload));
  listen<VolumeInfo>("store-update-volume", (e) => volume.set(e.payload));
  listen<NetworkStatus>("store-update-network", (e) => network.set(e.payload));
}

// Wrappers para emitir eventos al actualizar localmente
const setBattery = (val: BatteryInfo) => {
  battery.set(val);
  emit("store-update-battery", val);
};

const setVolume = (val: VolumeInfo) => {
  volume.set(val);
  emit("store-update-volume", val);
};

const setNetwork = (val: NetworkStatus) => {
  network.set(val);
  emit("store-update-network", val);
};

// Monitores: cacheados — se actualizan solo en resize real
let _monitorCache: MonitorRect[] | null = null;

export async function getMonitors(): Promise<MonitorRect[]> {
  if (_monitorCache) return _monitorCache;
  _monitorCache = await invoke<MonitorRect[]>("get_monitor_rects");
  return _monitorCache;
}

// Invalida caché solo si cambia el tamaño de pantalla (conexión/desconexión de monitor)
if (typeof window !== "undefined") {
  window.addEventListener("resize", () => {
    _monitorCache = null;
  });
}

// ---------------------------------------------------------------------------
// Stores derivados de conveniencia
// ---------------------------------------------------------------------------

export const batteryPercent: Readable<number> = derived(
  battery,
  ($b) => $b.percentage
);

export const isMuted: Readable<boolean> = derived(
  volume,
  ($v) => $v.is_muted
);

export const volumeLevel: Readable<number> = derived(
  volume,
  ($v) => $v.volume
);

// ---------------------------------------------------------------------------
// Motor de polling
// ---------------------------------------------------------------------------

type Cleanup = () => void;
let started = false;

export function startSystemPolling(): Cleanup {
  if (started) return () => {};
  started = true;

  // --- Volumen: rápido (1s) para respuesta inmediata ---
  async function pollVolume() {
    try {
      const v = await invoke<VolumeInfo>("get_volume_status");
      setVolume(v);
    } catch {
      // silencioso — el sistema puede no estar disponible momentáneamente
    }
  }

  // --- Batería: moderado (5s) ---
  async function pollBattery() {
    try {
      const b = await invoke<BatteryInfo>("get_battery_status");
      setBattery(b);
    } catch {}
  }

  // --- Red: lento (10s) ---
  async function pollNetwork() {
    try {
      const n = await invoke<NetworkStatus>("get_network_status");
      setNetwork(n);
    } catch {}
  }

  // Primera carga inmediata
  pollVolume();
  pollBattery();
  pollNetwork();

  const volumeInterval = setInterval(pollVolume, 1000);
  const batteryInterval = setInterval(pollBattery, 5000);
  const networkInterval = setInterval(pollNetwork, 10_000);

  return () => {
    clearInterval(volumeInterval);
    clearInterval(batteryInterval);
    clearInterval(networkInterval);
    started = false;
  };
}

// ---------------------------------------------------------------------------
// Acciones que invalidan el store inmediatamente (feedback instantáneo)
// ---------------------------------------------------------------------------

export async function setVolumeImmediate(value: number) {
  setVolume({ ...get(volume), volume: value });
  await invoke("set_volume", { value });
}

export async function toggleMuteImmediate() {
  await invoke("toggle_mute");
  // Re-leer el estado real tras el toggle
  const v = await invoke<VolumeInfo>("get_volume_status");
  setVolume(v);
}
