# Bugs found while testing chap-devops

Bugs in the systems that chap-devops deploys, found by running them for real.
Bugs in chap-devops itself are fixed in the same session and are not listed.
Each entry names where it was seen, what happens, and what would fix it. A
bug that is fixed leaves the file.

## Modeling App

### Import is blocked by default: "Outbreak indicator is required"

- Seen: Modeling App 7.3.0, DHIS2 2.42.6 with the Laos demo database,
  chap-core v2.5.0, 2026-10-19.
- On **Import** of a prediction, **Use alert outputs** is on by default. The
  form then needs an **Outbreak indicator**, and **Clear and import** does
  nothing until one is picked; the message is under the box, below the fold.
- The Laos demo database has a fitting element, `CHAP Dengue Cases (Any) -
  Weekly Outbreak Alert`, but the box does not suggest it, while each quantile
  box suggests its own `CHAP ... Quantile ...` element.
- Fix: suggest the outbreak alert element as the quantile elements are
  suggested, or default the switch to off when the setup has no alert policy
  (the setup in this test had none).

### Times are off by the UTC offset

- Seen: Modeling App 7.3.0 with chap-core v2.5.0, at 22:10 CEST (UTC+2).
- A prediction run started at 22:10 shows **Created 19.10.2026, 20:10** and
  **Last run about 2 hours ago**.
- chap-core returns `created` as `2026-10-19T20:10:43.203219`: UTC, with no
  time zone. The app reads it as local time.
- Fix: chap-core sends the offset (`...Z`), or the app reads a timestamp with
  no zone as UTC.

### The optional endemic threshold has no element in the demo database

- Seen: same as above.
- The import form has **Endemic threshold (optional)**; the Laos demo database
  has no element for it. Not a blocker, but the demo cannot show the feature.

## DHIS2 demo database (Laos, 2.42)

### The CHAP output elements are weekly, the forecasts monthly

- The `CHAP Dengue Cases (Any) - Weekly Quantile ...` and `... Outbreak Alert`
  elements belong to a weekly data set. A monthly forecast imports into them
  (DHIS2 2.42.6 accepts the values), but a monthly forecast has no monthly
  elements to go to.
- Fix: add monthly CHAP output elements to the demo database.

## chap-core

### A dataset that the baselines ran on cannot be deleted (CLIM-1447)

- Seen: chap-core `master` before v2.5.0, and v2.5.0.
- `DELETE /v1/crud/datasets/{id}` answers HTTP 500 (`ForeignKeyViolation ...
  "backtest_dataset_id_fkey"`) after a backtest, because the built-in
  baselines wrote backtests on the dataset. Every backtest listing hides
  them, so a client cannot find them to delete them first. The only way is
  through the `result` of the `run_baseline` jobs, which chap-devops uses.

### The refusal of a template without a revision contradicts itself

- `stored from revision None, but its source now reports revision None`.

### `GET /v1/jobs/{id}` gives the status only

- It answers `"SUCCESS"`, while `GET /v1/jobs` gives the whole job, with its
  start time and result.

## Open Climate Service (`main`)

### A failed read from the CHIRPS3 server is a 500, with no retry

- `POST /ingestions` answered 500 (`RasterioIOError: Read failed`,
  `TIFFFillTile ... got 0 bytes`) once; the same request passed 4 seconds
  later.

## chap-models

- IMPA-TECH-SARIMA fails in chap itself (`NoPredictionsError`).
- rwanda_random_forest fails at train when served with `mlproject run`.
- chap_pymc needs the packages of its `uv_env`, which a `-cli` image does not
  have.
