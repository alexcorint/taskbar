// Acción Svelte: sincroniza la posición de un elemento de la barra de tareas
// con la ventana de control_menu, usando coordenadas lógicas cross-DPI.
//
// Optimización: los monitor rects se obtienen del store cacheado (getMonitors)
// en lugar de llamar a get_monitor_rects por IPC cada segundo.

import { emit, listen } from "@tauri-apps/api/event";
import { getMonitors } from "$lib/stores/system";

export function anchor(node: HTMLElement, widgetId: string) {
  let interval: ReturnType<typeof setInterval>;

  async function syncPosition() {
    try {
      const rect = node.getBoundingClientRect();
      const monitors = await getMonitors(); // cacheado — sin IPC si ya se fetched
      const monitor = monitors[0];
      if (!monitor) return;

      const scale = monitor.scale_factor;
      const logicalMonitorX = monitor.x / scale;
      const logicalMonitorY = monitor.y / scale;
      const logicalMonitorWidth = monitor.width / scale;
      const logicalMonitorHeight = monitor.height / scale;

      const centerX = logicalMonitorX + rect.left + rect.width / 2;
      const taskbarBottom = logicalMonitorY + logicalMonitorHeight;

      await emit("sync-widget-anchor", {
        widgetId,
        centerX,
        taskbarBottom,
        monitorWidth: logicalMonitorWidth,
        scale,
      });
    } catch (e) {
      console.error(`[Anchor: ${widgetId}] Error syncing position:`, e);
    }
  }

  // Handshake inicial
  setTimeout(syncPosition, 500);

  // Escuchar peticiones explícitas
  const unlisten = listen("request-sync", () => syncPosition());

  // Sincronización periódica reducida a 2s (el monitor no cambia frecuentemente)
  interval = setInterval(syncPosition, 2000);
  window.addEventListener("resize", syncPosition);

  return {
    update(newWidgetId: string) {
      widgetId = newWidgetId;
      syncPosition();
    },
    destroy() {
      clearInterval(interval);
      window.removeEventListener("resize", syncPosition);
      unlisten.then((fn) => fn());
    },
  };
}
