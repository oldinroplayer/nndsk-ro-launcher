import { describe, expect, it } from 'vitest'
import type { ServerConfig } from '../../shared/types'
import {
  findSelectedServer,
  firstServerId,
  nextSelectedId,
} from './servers.logic'

const servers: ServerConfig[] = [
  { id: 'a', name: 'Alpha', executablePath: '/a/client.exe' },
  { id: 'b', name: 'Beta', executablePath: '/b/client.exe' },
  { id: 'c', name: 'Gamma', executablePath: '/c/client.exe' },
]

describe('nextSelectedId', () => {
  it('mantiene la selección si se elimina otro servidor', () => {
    expect(nextSelectedId('b', 'a', servers.slice(1))).toBe('b')
  })

  it('pasa al primero si se elimina el seleccionado', () => {
    expect(nextSelectedId('b', 'b', [servers[0], servers[2]])).toBe('a')
  })

  it('devuelve null si la lista queda vacía', () => {
    expect(nextSelectedId('a', 'a', [])).toBeNull()
  })
})

describe('findSelectedServer', () => {
  it('encuentra el servidor por id', () => {
    expect(findSelectedServer(servers, 'b')?.name).toBe('Beta')
  })

  it('devuelve null si el id no existe', () => {
    expect(findSelectedServer(servers, 'missing')).toBeNull()
  })
})

describe('firstServerId', () => {
  it('devuelve el id del primero', () => {
    expect(firstServerId(servers)).toBe('a')
  })

  it('devuelve null en lista vacía', () => {
    expect(firstServerId([])).toBeNull()
  })
})
