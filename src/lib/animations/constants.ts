/**
 * Duraciones estándar para mantener la consistencia visual.
 * Basado en principios de diseño de interfaces modernas (Fluent/iOS).
 */
export const DURATIONS = {
    FASTEST: 100, // Micro-interacciones instantáneas
    FAST: 200,    // Hovers, cambios de estado simples
    NORMAL: 300,  // Apertura de menús pequeños, transiciones de entrada
    SLOW: 500,    // Transiciones de página, layouts grandes
    SLOWEST: 800  // Animaciones ambientales o dramáticas
} as const;

/**
 * Delays estándar para orquestar secuencias de animaciones.
 */
export const DELAYS = {
    STAGGER: 50,  // Para listas (item 1: 0, item 2: 50, item 3: 100...)
    SHORT: 100,
    MEDIUM: 200,
    LONG: 400
} as const;
