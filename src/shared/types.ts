export interface ProgressPayload {
  step: string
  percent: number
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

export interface RunnerInfo {
  id: string
  name: string
  path: string
}

export interface AudioStatus {
  audioOk: boolean
  audioDriver: string
  audioWarning: string | null
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

export type ToolKind = 'opensetup' | 'patcher' | 'dgvoodoo'
