## Summary

Describe the change and its user or contract impact.

## Validation

- [ ] `make ci` passes locally, or the reason it cannot run is documented.
- [ ] Relevant tests were added or updated.
- [ ] Dependency changes include the required lockfile updates.

> The contract CI workflow is a required status check on `main`. Pull requests cannot be merged until it passes.

## Documentation review

- [ ] If public entrypoints, events, errors, or deployment behavior changed, the contract interface reference and deployment checklist were updated accordingly.

## Contract safety

- [ ] I considered compatibility and migration implications for deployed contracts.