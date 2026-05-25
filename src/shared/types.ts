export interface ProgressPayload {
  step: string
  percent: number
}

export interface LogEventPayload {
  line: string
}

export interface ExitEventPayload {
  code: number
}

export interface AppSettings {
  defaultRunner: string
}

export interface ServerConfig {
  id: string
  name: string
  executablePath: string
  patcherPath?: string
  winePrefix?: string
  runner?: string // path to wine/proton binary; undefined = use global setting
}

export interface DependencyStatus {
  wine: boolean
  winetricks: boolean
  dxvk: boolean
  prefixConfigured: boolean
  audioOk: boolean
  audioDriver: string
  audioWarning: string | null
}

export type AudioStatus = Pick<
  DependencyStatus,
  'audioOk' | 'audioDriver' | 'audioWarning'
>

export interface RunnerInfo {
  id: string
  name: string
  path: string
}

export interface ToolInfo {
  found: boolean
  path: string | null
  label: string | null
}

export interface DgVoodooStatus {
  cpl: ToolInfo
  d3dimmDll: ToolInfo
  ddrawDll: ToolInfo
  conf: ToolInfo
  configured: boolean
  needsInstall: boolean
  canAutoInstall: boolean
  canUninstall: boolean
  issues: string[]
}

export interface ServerToolsStatus {
  gameDir: string
  openSetup: ToolInfo
  patcher: ToolInfo
  dgvoodoo: DgVoodooStatus
}

export interface InstallDgVoodooResult {
  installed: string[]
  status: ServerToolsStatus
}

export interface UninstallDgVoodooResult {
  removed: string[]
  status: ServerToolsStatus
}

export const TOOL_KINDS = ['opensetup', 'patcher', 'dgvoodoo'] as const

export type ToolKind = (typeof TOOL_KINDS)[number]

export function isToolKind(value: string | null): value is ToolKind {
  return TOOL_KINDS.includes(value as ToolKind)
}
