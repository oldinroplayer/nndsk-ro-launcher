/** Último segmento de una ruta (compatible con / y \\). */
export function basename(path: string): string {
  return path.split(/[/\\]/).pop() ?? path
}

/** Nombre del cliente a partir del .exe (sin extensión). */
export function nameFromExePath(path: string): string {
  return basename(path).replace(/\.exe$/i, '')
}
