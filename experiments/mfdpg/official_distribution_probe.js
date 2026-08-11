/* Reuses the exact RandExp and random-seed dependency versions installed from
 * the pinned official MFDPG artifact. Run with NODE_PATH=<mfdpg>/node_modules.
 */
const { createHash } = require('crypto')
const RandExp = require('randexp')
const rand = require('random-seed')

const totalSamples = Number(process.argv[2] || 100000)
const batchCount = Number(process.argv[3] || 10)
const rootSeed = process.argv[4] || 'epscd-mfdpg-20260809'
if (!Number.isSafeInteger(totalSamples) || totalSamples <= 0 ||
    !Number.isSafeInteger(batchCount) || batchCount <= 0 ||
    totalSamples % batchCount !== 0) {
  throw new Error('sample count must be a positive multiple of batch count')
}

const outputs = ['a', ...Array.from({ length: 10 }, (_, i) => `b${i}`)]
const emptyCounts = () => Object.fromEntries(outputs.map(value => [value, 0]))

function summarize (counts, samples) {
  const frequencies = Object.values(counts)
  const expected = samples / outputs.length
  const minimum = Math.min(...frequencies)
  const maximum = Math.max(...frequencies)
  return {
    totalVariationDistanceToUniform: frequencies.reduce(
      (sum, count) => sum + Math.abs(count / samples - 1 / outputs.length), 0
    ) / 2,
    chiSquare: frequencies.reduce(
      (sum, count) => sum + ((count - expected) ** 2) / expected, 0
    ),
    chiSquareDegreesOfFreedom: outputs.length - 1,
    expectedCountPerOutput: expected,
    minimumFrequency: minimum,
    maximumFrequency: maximum,
    maxMinFrequencyRatio: maximum / minimum
  }
}

const aggregateCounts = emptyCounts()
const samplesPerBatch = totalSamples / batchCount
const batches = []
for (let batch = 0; batch < batchCount; batch++) {
  const counts = emptyCounts()
  for (let sample = 0; sample < samplesPerBatch; sample++) {
    const seed = createHash('sha256')
      .update(`${rootSeed}:batch=${batch}:sample=${sample}`)
      .digest('hex')
    const generator = new RandExp(/a|b[0-9]/)
    const rng = rand.create(seed)
    generator.randInt = rng.intBetween
    const output = generator.gen()
    counts[output]++
    aggregateCounts[output]++
  }
  batches.push({
    batch,
    seedDomain: `${rootSeed}:batch=${batch}`,
    samples: samplesPerBatch,
    ...summarize(counts, samplesPerBatch)
  })
}

process.stdout.write(`${JSON.stringify({
  schemaVersion: 1,
  publicationStatus: 'preprint artifact; no verified peer-reviewed venue as of 2026-08-09',
  baseline: 'official MFDPG output-generator dependencies',
  officialRepository: 'https://github.com/multifactor/mfdpg',
  officialCommit: '6c26096dd22ff2b18aa5d8e4c3d5b0caf7b45bb7',
  lockfileVersion: 3,
  randexpVersion: '0.5.3',
  randomSeedVersion: '0.3.0',
  regex: 'a|b[0-9]',
  acceptedLanguage: outputs,
  languageSize: outputs.length,
  totalSamples,
  batchCount,
  samplesPerBatch,
  rootSeed,
  aggregate: {
    counts: aggregateCounts,
    ...summarize(aggregateCounts, totalSamples)
  },
  batches,
  boundary: 'Artifact-specific output-selection observation. The probe executes the exact locked RandExp and random-seed dependencies and the randInt replacement used by MFDPG.generate, but replaces Argon2id preimages with labeled SHA-256 seeds; it is neither an end-to-end MFDPG benchmark nor a cryptographic-vulnerability claim.'
}, null, 2)}\n`)
