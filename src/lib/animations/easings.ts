import { 
    cubicOut, 
    expoOut, 
    quintOut, 
    backOut, 
    elasticOut 
} from "svelte/easing";

/**
 * Curvas de aceleración (Easings) personalizadas y estándar.
 */
export const EASINGS = {
    // Svelte Defaults (re-exported for consistency)
    CUBIC_OUT: cubicOut,
    EXPO_OUT: expoOut,
    QUINT_OUT: quintOut,
    BACK_OUT: backOut,
    ELASTIC_OUT: elasticOut,

    // Custom Easings
    SOFT_OUT: "cubic-bezier(0.4, 0, 0.2, 1)", // El estándar de Material Design
    DRAMATIC_OUT: "cubic-bezier(0.19, 1, 0.22, 1)", // Muy suave al final
    REBOUND: "cubic-bezier(0.34, 1.56, 0.64, 1)", // Efecto rebote suave sin oscilación exagerada
} as const;

/**
 * Función para generar un string de cubic-bezier compatible con CSS 
 * a partir de una función de easing si es necesario (uso avanzado).
 */
export function toCssEasing(easingName: keyof typeof EASINGS): string {
    const val = EASINGS[easingName];
    return typeof val === "string" ? val : "ease-out"; // Simplificación para CSS
}
