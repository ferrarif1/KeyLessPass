/* End-to-end generate/revoke experiment using the pinned official MFDPG API. */
const MFDPG = require('./upstream')
const mfkdf = require('./upstream/node_modules/mfkdf')

const factorSets = 3
const versionsPerSet = 12
const regex = /a|b[0-9]/

async function run () {
  const rows = []
  for (let factorSet = 0; factorSet < factorSets; factorSet++) {
    const generator = await new MFDPG([
      await mfkdf.setup.factors.password(`epscd-mfdpg-factor-${factorSet}`)
    ])
    const domain = `enumerable-${factorSet}.example`
    const passwords = []
    for (let version = 0; version < versionsPerSet; version++) {
      passwords.push(await generator.generate(domain, regex))
      if (version + 1 < versionsPerSet) await generator.revoke(domain)
    }
    const distinct = new Set(passwords).size
    rows.push({
      factorSet,
      domain,
      versions: versionsPerSet,
      distinctPasswords: distinct,
      repeatedPasswords: versionsPerSet - distinct,
      passwords
    })
  }
  process.stdout.write(`${JSON.stringify({
    schemaVersion: 1,
    classification: 'EMPIRICAL_END_TO_END_ARTIFACT_RESULT',
    officialRepository: 'https://github.com/multifactor/mfdpg',
    officialCommit: '6c26096dd22ff2b18aa5d8e4c3d5b0caf7b45bb7',
    mechanism: 'MFDPG.generate followed by MFDPG.revoke',
    regex: 'a|b[0-9]',
    acceptedLanguageSize: 11,
    factorSets,
    versionsPerSet,
    aggregateVersions: rows.reduce((sum, row) => sum + row.versions, 0),
    aggregateDistinctWithinSets: rows.reduce((sum, row) => sum + row.distinctPasswords, 0),
    aggregateRepeatedWithinSets: rows.reduce((sum, row) => sum + row.repeatedPasswords, 0),
    rows,
    boundary: 'The 11-word toy domain makes repetition observable and is not representative of real password-space security. The experiment asks only whether the released revoke mechanism structurally prevents output reuse; it does not assess MFDPG security.'
  }, null, 2)}\n`)
}

run().catch(error => {
  console.error(error)
  process.exitCode = 1
})
