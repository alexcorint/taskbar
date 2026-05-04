import { invoke } from "@tauri-apps/api/core";
import { emit, listen } from "@tauri-apps/api/event";

export function anchor(node: HTMLElement, widgetId: string) {
    let interval: ReturnType<typeof setInterval>;

    async function syncPosition() {
        try {
            const rect = node.getBoundingClientRect();

            // Obtenemos monitores desde Rust para mayor precisión
            const monitors = await invoke<Array<any>>("get_monitor_rects");
            const monitor = monitors[0]; // Usamos el primario por ahora
            if (!monitor) return;

            const scale = monitor.scale_factor;

            // Coordenadas LÓGICAS (independientes del escalado)
            // monitor.x/y vienen de Rust en píxeles físicos
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
            console.error(`[Anchor Action: ${widgetId}] Error syncing position:`, e);
        }
    }

    // Handshake inicial
    setTimeout(syncPosition, 500);

    // Escuchar peticiones explícitas de sincronización
    const unlisten = listen("request-sync", () => {
        syncPosition();
    });

    // Actualización periódica y ante cambios de ventana
    interval = setInterval(syncPosition, 1000);
    window.addEventListener("resize", syncPosition);

    return {
        update(newWidgetId: string) {
            widgetId = newWidgetId;
            syncPosition();
        },
        destroy() {
            clearInterval(interval);
            window.removeEventListener("resize", syncPosition);
            unlisten.then(fn => fn());
        }
    };
}
