# Lane Ra1 (Raise) - OI-4 admission-ratio band

Standing: PARTIAL. Band logic ALIVE via selftest; full e2e pipeline run UNVERIFIED.

Owned file: scripts/courts/court_e2e_arcus_cgap_smalt.sh (present; not BLOCKED).

## Change
- Added `band_check` (python reads receipt arcus_admitted/arcus_sent) with band
  [ADMIT_BAND_LO=0.20, ADMIT_BAND_HI=0.60], env-overridable. Rationale: observed
  355,824/1,000,000 = 0.3558; SPSC saturation refusals lawful (regime 3).
  Band is empirical, not spec. HI also guards vacuity (no refusals = regime 3 unexercised).
- Called after the PASS 1 conservation check; out of band exits 1.
- Added `--band-selftest` mode (synthetic receipts, no pipeline needed).

## Commands
- `bash -n` script: exit 0
- `bash script --band-selftest`: exit 0 (observed 0.3558 in band; 100%-admitted mutant flipped)
- Mutation M-BAND-WIDEN: `ADMIT_BAND_HI=1.0 bash script --band-selftest`: exit 1
  ("mutant passed band: VACUOUS") - court flips.

## Not verified
- Full script run: needs cgap_e2e_consumer, arcus producer, smalt (mix); not run here.
  The in-pipeline band_check call is syntax-checked only.
- Script's WORKSPACE_ROOT is `../../..` from scripts/courts (parent of repo), a pre-existing quirk; untouched.

## Fences
Only this script edited; no other gate/court consumes it. No git writes.
