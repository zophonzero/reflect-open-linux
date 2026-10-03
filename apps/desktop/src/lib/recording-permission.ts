import { call } from '@reflect/core'
import { z } from 'zod'
import { isNativeShell } from '@/lib/platform.ts'

/** Linux needs app consent before WebKitGTK can grant microphone access. */
export async function requestRecordingPermission(): Promise<boolean> {
  if (!isNativeShell() || import.meta.env.TAURI_ENV_PLATFORM !== 'linux') return true
  return await call('microphone_request_access', {}, z.boolean())
}
