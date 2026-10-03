import assert from 'node:assert/strict'
import { test } from 'vitest'
import { archVersion, packageRecipe } from './release-linux.mjs'

test('maps the canonical version to an Arch-compatible prerelease', () => {
  assert.equal(archVersion('0.15.0-beta.1'), '0.15.0beta1')
  assert.equal(archVersion('0.15.0'), '0.15.0')
  assert.throws(() => archVersion('invalid'))
})

test('pins the packaged artifact and rejects shell syntax in metadata', () => {
  const recipe = packageRecipe('0.15.0-beta.1', 'reflect-linux.tar.gz', 'a'.repeat(64))
  assert.ok(recipe.includes("source=('reflect-linux.tar.gz')"))
  assert.ok(recipe.includes(`sha256sums=('${'a'.repeat(64)}')`))
  assert.throws(() => packageRecipe('0.15.0', "bad'archive", 'a'.repeat(64)))
  assert.throws(() => packageRecipe('0.15.0', 'app.tar.gz', 'SKIP'))
})
