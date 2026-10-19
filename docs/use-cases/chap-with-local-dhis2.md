# Chap with a local DHIS2 and the Modeling App

A complete local setup: chap-core, its models, and a DHIS2 seeded from the demo
database, connected so the Modeling App in DHIS2 talks to this Chap. For
development, demos and training.

```sh
chap-devops init mychap --models chapkit_simple_multistep_model --with dhis2
cd mychap
chap-devops up                     # the first start of DHIS2 can take minutes
chap-devops status                 # wait until the dhis2 line says up, not starting
chap-devops dhis2 connect          # the route to chap-core, the apps, analytics
chap-devops open dhis2
```

The first `chap-devops up` on a machine pulls the images. The Chap images are about
6 GB to download (see [Quickstart](../quickstart.md)), and DHIS2 adds 1 to
1.5 GB. When the images are on the machine, `up` returns in seconds. DHIS2 then
restores the Laos demo database and migrates its schema. On an arm64 Mac, the
`dhis2` line of `chap-devops status` said `up` less than a minute after `up`
returned. Under emulation, this can take 8 to 15 minutes. `chap-devops logs dhis2`
shows the progress.

`chap-devops up` does not connect DHIS2 to Chap; `chap-devops dhis2 connect` does. Until
it has run once, `up` ends with this line, and `status` shows a shorter form:

```text
chap-devops has not connected this DHIS2 to Chap; run `chap-devops dhis2 connect` once DHIS2 answers
```

If DHIS2 is still starting, `chap-devops dhis2 connect` waits for it, for at most 20
minutes. It logs in as `admin` / `district`, makes the `chap` route, installs
the two apps from the App Hub, and generates the analytics tables. That took
about 30 seconds on the demo database:

```text
DHIS2 2.42.6 at http://localhost:8780, as `admin`
repointed the `chap` route at http://chap:8000/**; chap-core answered through it
installed Modeling App 7.2.0
installed DHIS2 Climate App 1.16.2
analytics finished in 25 seconds (job HXWd71zzX8L)
the Modeling App can reach Chap; open DHIS2 with `chap-devops open dhis2`
```

The demo database has a `chap` route already, so `connect` repoints it at this
chap-core. On an empty DHIS2, that line starts with `created` instead.

It worked when:

- `chap-devops dhis2 connect` ends with `the Modeling App can reach Chap`.
- `chap-devops open dhis2` opens DHIS2 on `http://localhost:8780`. Log in with
  `admin` / `district`.
- Typing `Modeling` in the app menu (the grid icon at the top right) finds
  **Modeling**, and its **Models** page lists the models.

Next, in the Modeling App: [Your first forecast in the Modeling
App](../modeling-app.md) evaluates a model on the demo data and makes a
forecast with it.

To remove the deployment and its data, run `chap-devops down --volumes --yes` in
`mychap`.

## Another DHIS2 version

```sh
chap-devops init chap243 --models chapkit_simple_multistep_model --with dhis2 --dhis2-tag 2.43
chap-devops init chap241 --models chapkit_simple_multistep_model --with dhis2 --dhis2-tag 2.41
chap-devops init chapdev --models chapkit_simple_multistep_model --with dhis2 --dhis2-image dhis2/core-dev --dhis2-tag master
```

Each line makes a new deployment. If the directory already exists, `init`
stops with `already contains a chap-devops deployment; choose another directory, or use --force to overwrite it`. Use a
different name.

The deployments use the same ports (8700 and 8780), so only one of them can be
up at a time. If another deployment is up, `init` tells you, for example:

```text
warning: port 8780 is already in use on this machine (needed by dhis2); free it, or run `chap-devops components enable dhis2 --port 8781`
```

Before you start the new deployment, run `chap-devops down` in the old one.

The Laos climate demo database exists for 2.42 only, so these start with an
empty DHIS2. `init` tells you so:

```text
warning: chap-devops knows no DHIS2 demo dump for 2.43, so `dhis2_db` starts empty; name one with `seed:` in `.chap-devops/components.yaml` (a URL or a path) and run `chap-devops sync`
```

An empty DHIS2 still has the `admin` / `district` login, and `dhis2 connect`
works the same. An empty DHIS2 has no data for the Modeling App to evaluate a
model on.

Do not seed them with DHIS2's Sierra Leone demo when you want Chap connected.
Its `admin` is not a superuser, and `dhis2 connect` stops at the route:

```text
error: DHIS2 refused to write the `chap` route as `admin` (HTTP 403, "You don't have the proper permissions to create this object."): that needs the F_ROUTE_PUBLIC_ADD authority; add it to one of that user's roles in DHIS2 (Users, User role), or name a superuser with `--user NAME`
```

See [Picking the version](./dhis2-alone.md#picking-the-version) and
[Picking the data](./dhis2-alone.md#picking-the-data) in A DHIS2 on its own.

Next: [DHIS2](../dhis2.md), especially
[Connecting the Modeling App to Chap](../dhis2.md#connecting-the-modeling-app-to-chap)
and [Memory](../dhis2.md#memory).

All shapes: [Use cases](../use-cases.md).
