import { execFileSync } from 'node:child_process'
import { createHash } from 'node:crypto'
import {
  chmodSync,
  copyFileSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from 'node:fs'
import { tmpdir } from 'node:os'
import { basename, join, resolve } from 'node:path'
import { z } from 'zod'

const repoRoot = resolve(import.meta.dirname, '../../..')
const packageMetadata = z.object({ version: z.string() })

/** Arch prereleases must sort below the corresponding stable version. */
export function archVersion(version) {
  if (!/^\d+\.\d+\.\d+(?:-beta\.\d+)?$/.test(version)) {
    throw new Error(`Unsupported app version: ${version}`)
  }
  return version.replace('-beta.', 'beta')
}

/** A package recipe for the exact native archive built alongside it. */
export function packageRecipe(version, archive, checksum) {
  if (!/^[\w.-]+$/.test(archive) || !/^[a-f0-9]{64}$/.test(checksum)) {
    throw new Error('Invalid archive metadata')
  }
  return `pkgname=reflect-open-linux
pkgver=${archVersion(version)}
pkgrel=1
pkgdesc='Offline-first Markdown notes with links, search, and BYOK AI'
arch=('x86_64')
url='https://github.com/zophonzero/reflect-open-linux'
license=('MIT')
depends=('webkit2gtk-4.1' 'gtk3' 'openssl' 'glibc' 'gcc-libs' 'hicolor-icon-theme' 'xdg-utils' 'gst-plugins-base' 'gst-plugins-good' 'gst-plugins-bad')
optdepends=('gnome-keyring: Secret Service for AI and GitHub credentials' 'gst-libav: additional audio and video codecs')
provides=('reflect-open')
conflicts=('reflect-open')
options=('!strip')
source=('${archive}')
sha256sums=('${checksum}')
package() {
  cp -a "$srcdir/usr" "$pkgdir/"
}
`
}

function build(artifactDir) {
  if (process.platform !== 'linux' || process.arch !== 'x64') {
    throw new Error('This package currently targets Linux x86_64')
  }
  execFileSync('pnpm', ['tauri', 'build', '--no-bundle'], { cwd: repoRoot, stdio: 'inherit' })
  const { version } = packageMetadata.parse(
    JSON.parse(readFileSync(join(repoRoot, 'apps/desktop/package.json'), 'utf8')),
  )
  archVersion(version)
  const stage = mkdtempSync(join(tmpdir(), 'reflect-linux-package-'))
  try {
    const binaries = join(stage, 'usr/lib/reflect-open')
    mkdirSync(binaries, { recursive: true })
    for (const name of ['reflect-open', 'reflect', 'reflect-capture-host']) {
      copyFileSync(join(repoRoot, 'target/release', name), join(binaries, name))
      chmodSync(join(binaries, name), 0o755)
    }
    const binDir = join(stage, 'usr/bin')
    mkdirSync(binDir, { recursive: true })
    for (const name of ['reflect-open', 'reflect']) {
      symlinkSync(`../lib/reflect-open/${name}`, join(binDir, name))
    }
    for (const [source, destination] of [
      [
        'packaging/linux/app.reflect.desktop.desktop',
        'usr/share/applications/app.reflect.desktop.desktop',
      ],
      [
        'apps/desktop/src-tauri/icons/128x128.png',
        'usr/share/icons/hicolor/128x128/apps/app.reflect.desktop.png',
      ],
      ['LICENSE', 'usr/share/licenses/reflect-open-linux/LICENSE'],
    ]) {
      const target = join(stage, destination)
      mkdirSync(resolve(target, '..'), { recursive: true })
      copyFileSync(join(repoRoot, source), target)
    }
    mkdirSync(artifactDir, { recursive: true })
    const archive = `reflect-open-${version}-linux-x86_64.tar.gz`
    const archivePath = join(artifactDir, archive)
    execFileSync('tar', ['-czf', archivePath, '-C', stage, 'usr'], { stdio: 'inherit' })
    const checksum = createHash('sha256').update(readFileSync(archivePath)).digest('hex')
    writeFileSync(join(artifactDir, 'PKGBUILD'), packageRecipe(version, archive, checksum))
    writeFileSync(join(artifactDir, `${archive}.sha256`), `${checksum}  ${archive}\n`)
    console.log(`Linux artifacts: ${artifactDir}`)
  } finally {
    rmSync(stage, { recursive: true, force: true })
  }
}

if (process.argv[1] && resolve(process.argv[1]) === import.meta.filename) {
  const [command = 'build', ...args] = process.argv.slice(2)
  const output = args.find((arg) => arg.startsWith('--artifact-dir='))
  if (
    command !== 'build' ||
    args.some((arg) => !arg.startsWith('--artifact-dir=')) ||
    args.length > 1
  ) {
    throw new Error(`Usage: ${basename(process.argv[1])} build [--artifact-dir=/path]`)
  }
  build(
    resolve(output?.slice('--artifact-dir='.length) || join(repoRoot, 'target/linux-artifacts')),
  )
}
