import type { Vector3 } from 'three';

/**
 * Convert geographic (lat, lon) in degrees to a Vector3 on a sphere of given radius.
 * Standard spherical projection: lat=0,lon=0 → (radius, 0, 0).
 * Source: standard astronomical convention; cross-checked against three.js examples.
 */
export function latLonToVec3(lat: number, lon: number, radius: number, out: Vector3): Vector3 {
  const phi = (90 - lat) * (Math.PI / 180);
  const theta = (lon + 180) * (Math.PI / 180);
  const x = -radius * Math.sin(phi) * Math.cos(theta);
  const y = radius * Math.cos(phi);
  const z = radius * Math.sin(phi) * Math.sin(theta);
  out.set(x, y, z);
  return out;
}

/** Detect WebGL 1/2 availability without instantiating a real renderer. */
export function hasWebGL(): boolean {
  if (typeof window === 'undefined' || typeof document === 'undefined') return false;
  try {
    const canvas = document.createElement('canvas');
    return !!(
      canvas.getContext('webgl2') ||
      canvas.getContext('webgl') ||
      (canvas.getContext('experimental-webgl') as WebGLRenderingContext | null)
    );
  } catch {
    return false;
  }
}

/** Read prefers-reduced-motion (returns false during SSR / no matchMedia). */
export function prefersReducedMotion(): boolean {
  if (typeof window === 'undefined' || !window.matchMedia) return false;
  return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
}
