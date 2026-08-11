/* Minimal harness over the exact dependencies locked by the official artifact.
 * It isolates MFDPG index.js lines 132--135: RandExp, random-seed, and randInt.
 */
const { createHash } = require('crypto')
const RandExp = require('./upstream/node_modules/randexp')
const rand = require('./upstream/node_modules/random-seed')

const totalSamples = 100000
const batchCount = 10
const samplesPerBatch = totalSamples / batchCount
const rootSeed = 'epscd-mfdpg-official-20260809'
const outputs = ['a', ...Array.from({ length: 10 }, (_, index) => `b${index}`)]

function emptyCounts () {
  return Object.fromEntries(outputs.map(output => [output, 0]))
}

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
    maxMinFrequencyRatio: maximum / minimum,
    empiricalMinEntropyBits: -Math.log2(maximum / samples)
  }
}

const aggregateCounts = emptyCounts()
const batches = []
for (let batch = 0; batch < batchCount; batch++) {
  const counts = emptyCounts()
  for (let sample = 0; sample < samplesPerBatch; sample++) {
    const seed = createHash('sha256')
      .update(`${rootSeed}:batch=${batch}:sample=${sample}`)
      .digest('hex')
    const generator = new RandExp(/a|b[0-9]/)
    generator.randInt = rand.create(seed).intBetween
    const output = generator.gen()
    counts[output]++
    aggregateCounts[output]++
  }
  batches.push({ batch, samples: samplesPerBatch, counts, ...summarize(counts, samplesPerBatch) })
}

process.stdout.write(`${JSON.stringify({
  schemaVersion: 1,
  classification: 'EMPIRICAL_ARTIFACT_RESULT',
  publicationStatus: 'arXiv preprint; peer-reviewed venue not verified',
  officialRepository: 'https://github.com/multifactor/mfdpg',
  officialCommit: '6c26096dd22ff2b18aa5d8e4c3d5b0caf7b45bb7',
  exercisedArtifactPath: 'index.js:132-135 via exact locked dependencies',
  regex: 'a|b[0-9]',
  acceptedLanguage: outputs,
  languageSize: outputs.length,
  totalSamples,
  batchCount,
  samplesPerBatch,
  rootSeed,
  aggregate: { counts: aggregateCounts, ...summarize(aggregateCounts, totalSamples) },
  batches,
  boundary: 'The official output-selection dependencies and randInt override are executed unchanged. Labeled SHA-256 seeds replace the artifact Argon2id preimage stage so that output selection can be sampled 100,000 times. This is not an end-to-end MFDPG latency or security experiment.'
}, null, 2)}\n`)
