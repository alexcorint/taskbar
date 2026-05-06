import Icon from '@iconify/svelte';

/**
 * Diccionario de excepciones (nombres de proceso que no coinciden con su marca en simple-icons).
 * También incluye iconos de sistema que preferimos usar de Fluent.
 */
export const exceptions: Record<string, string> = {
    // --- SISTEMA ---
    'explorer': 'fluent:folder-24-filled',
    'settings': 'fluent:settings-24-filled',
    'systemsettings': 'fluent:settings-24-filled',
    'taskmgr': 'fluent:status-24-filled',
    'cmd': 'fluent:channel-share-12-filled',
    'powershell': 'simple-icons:powershell',
    'windowsterminal': 'fluent:terminal-24-filled',
    'calculator': 'fluent:calculator-24-filled',
    'calc': 'fluent:calculator-24-filled',
    'notepad': 'fluent:note-24-filled',
    'clock': 'fluent:clock-24-filled',
    'photos': 'fluent:image-24-filled',
    'snippingtool': 'fluent:cut-24-filled',

    // --- REDES / HERRAMIENTAS ---
    'msedge': 'simple-icons:microsoftedge',
    'microsoft edge': 'simple-icons:microsoftedge',
    'chrome': 'simple-icons:googlechrome',
    'code': 'simple-icons:visualstudiocode',
    'spotify': 'simple-icons:spotify',
    'spotifylauncher': 'simple-icons:spotify',
    'discord': 'simple-icons:discord',
    'slack': 'simple-icons:slack',
    'teams': 'simple-icons:microsoftteams',
    'whatsapp': 'simple-icons:whatsapp',
    'whatsapp.root': 'simple-icons:whatsapp',
    'telegram': 'simple-icons:telegram',
    'steamwebhelper': 'simple-icons:steam',
    'obs': 'simple-icons:obsstudio',
    'file explorer': 'fluent:folder-24-filled',
    'explorador de archivos': 'fluent:folder-24-filled',
};

/**
 * Lógica de "Iconificación" de aplicaciones.
 * Decide si usar un icono de librería (Fluent/SimpleIcons) o el original de Windows.
 * 
 * @param exePath Ruta completa o nombre del ejecutable (ej: "chrome.exe")
 * @returns El identificador de Iconify si se encuentra en excepciones o se asume marca.
 */
export function iconifyer(exePath: string): string {
    if (!exePath) return 'fluent:app-generic-24-filled';

    // 1. Extraer el nombre del archivo manejando ambos tipos de barra
    const fileName = exePath.split(/[\\/]/).pop() || exePath;

    // 2. Limpiar extensiones comunes y normalizar
    const baseName = fileName
        .toLowerCase()
        .replace(/\.(exe|lnk|bat|cmd|com)$/, '')
        .trim();

    // 3. ¿Es un "rebelde"? (Lista de excepciones)
    if (exceptions[baseName]) {
        return exceptions[baseName];
    }

    // 4. El "Camino Feliz": asumimos que el nombre coincide con la marca
    return `simple-icons:${baseName}`;
}

export default Icon;