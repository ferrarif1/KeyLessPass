# MP-SPDZ threshold experiment

`aster_exact_domain.mpc` evaluates the exact-domain rank-release path with the
official MP-SPDZ `mal-shamir-bmr-party.x` honest-majority malicious backend.
The construction deliberately distinguishes two claims:

1. MP-SPDZ supplies the threshold execution and active-security backend.
2. ASTER supplies the circuit composition: XOR-combined private key inputs,
   a domain-separated ten-round AES Feistel permutation, fixed-cap secret
   cycle walking, and release of only the success bit and final rank.

This is not described as a reviewed threshold implementation of FF1, and the
experiment does not claim availability from arbitrary subsets of online
parties.  With three parties the Shamir corruption threshold is one; with five
parties it is two.

The fixed input vector is generated and independently evaluated by
`aster_exact_domain_reference.py`.  `run_mpspdz_experiment.py` builds/uses the
pinned official MP-SPDZ commit, runs three- and five-party loopback trials, and
writes machine-readable timing and agreement results.
