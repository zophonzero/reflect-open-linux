import { afterEach, describe, expect, it, vi } from 'vitest'
import { supportsAgentSkills, supportsAppUpdates } from './desktop-capabilities.ts'

const nativeShell = vi.hoisted(() => vi.fn(() => true))
vi.mock('@/lib/platform.ts', () => ({ isMacosDesktop: false, isNativeShell: nativeShell }))

afterEach(() => {
  vi.unstubAllEnvs()
  nativeShell.mockReturnValue(true)
})

describe('Linux desktop capabilities', () => {
  it('enables agent access but leaves updates to the package manager', () => {
    vi.stubEnv('TAURI_ENV_PLATFORM', 'linux')
    expect(supportsAgentSkills()).toBe(true)
    expect(supportsAppUpdates()).toBe(false)
  })

  it('does not expose native capabilities in browser previews', () => {
    vi.stubEnv('TAURI_ENV_PLATFORM', 'linux')
    nativeShell.mockReturnValue(false)
    expect(supportsAgentSkills()).toBe(false)
    expect(supportsAppUpdates()).toBe(false)
  })

  it('preserves Windows updater support', () => {
    vi.stubEnv('TAURI_ENV_PLATFORM', 'windows')
    expect(supportsAppUpdates()).toBe(true)
    expect(supportsAgentSkills()).toBe(false)
  })

  it.each(['ios', 'android'])('does not enable desktop integrations on %s', (platform) => {
    vi.stubEnv('TAURI_ENV_PLATFORM', platform)
    expect(supportsAgentSkills()).toBe(false)
    expect(supportsAppUpdates()).toBe(false)
  })
})
