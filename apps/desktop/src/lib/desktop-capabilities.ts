import { isMacosDesktop, isNativeShell } from '@/lib/platform.ts'

/** Desktop platforms whose installed bundles expose the agent CLI. */
export function supportsAgentSkills(): boolean {
  return isMacosDesktop || (isNativeShell() && import.meta.env.TAURI_ENV_PLATFORM === 'linux')
}

/** Linux packages receive updates through their package manager. */
export function supportsAppUpdates(): boolean {
  const platform = import.meta.env.TAURI_ENV_PLATFORM
  return isNativeShell() && platform !== 'linux' && platform !== 'ios' && platform !== 'android'
}
