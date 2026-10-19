# A model image you built yourself

For testing a model the way it will really run: build its image on this
machine and run it in a chap-devops deployment, registered with chap-core, without
publishing it anywhere first.

For a new model, `chap-devops models new` creates a project that builds in this
way, and its `README.md` has these commands with the names filled in. See
[A new model project](../models.md#a-new-model-project).

In the model's checkout:

```sh
docker build --platform linux/amd64 --build-arg GIT_REVISION=$(git rev-parse HEAD) -t my-model:dev .
```

The `GIT_REVISION` build argument puts the commit in the image. The model
reports it when it registers, and chap-core 2.4 refuses to store the model
template of a service that reports no revision. Without it, the model stays
`registered, not configured` and nothing can run it.

The first build pulls the model's base image, such as
`ghcr.io/dhis2-chap/chapkit-py` or `ghcr.io/dhis2-chap/chapkit-r-inla`. The
R-INLA base is about 1.4 GB, and its pull can take ten minutes or more. A
rebuild uses the cached base and takes some seconds.

The model registers with chap-core under the `id` in its `MLServiceInfo`
(in `main.py` for a chapkit model). Find that id before you add the image:

```sh
grep -A1 'MLServiceInfo(' main.py
```

Then make a deployment with chap-core, or use one that you have:

```sh
chap-devops init lab --models none
cd lab
```

Add the image with that id as `--service-id`, and start it:

```sh
chap-devops models add my-model:dev --service-id chapkit-minimalist-example-py
chap-devops up --wait
chap-devops status
chap-devops models test my_model
```

Replace `chapkit-minimalist-example-py` with the id of your model. The model
id, `my_model`, comes from the image name. `chap-devops models add` prints:

```text
added my_model (chapkit-minimalist-example-py)
enabled my_model dev at http://localhost:8700/v2/services/chapkit-minimalist-example-py/run/
the service must register with chap-core as `chapkit-minimalist-example-py`; if its own MLServiceInfo.id differs, `chap-devops status` shows it as unmanaged - run `chap-devops models remove my_model`, then add it again with `--service-id <that id>`
run `chap-devops up` to apply
```

`chap-devops up --wait` returns when the model has registered, and it then
creates the configured model that chap-core needs to run it. A plain
`chap-devops up` returns when the containers start; `chap-devops status` then says
`running, not registered` for some seconds, and `registered, not
configured` until `chap-devops models configs sync` runs. After `chap-devops up --wait`,
`chap-devops status` shows:

```text
chap-core   up   http://localhost:8700   2.4.0   auth: off

MODEL                          STATE       REACH          LAST PING
chapkit-minimalist-example-py  registered  via chap-core  3s ago

1 model registered
```

`chap-devops models test my_model` runs `chapkit test` in the model's container:

```text
testing 1 model (model level; add --backtest to go through chap-core)
chapkit-minimalist-example-py    pass    9s   1 training, 1 prediction

1 of 1 model passes
```

Without `--service-id`, the service id is the image name, here `my-model`.
If the model registers under a different id, `chap-devops status` shows its row as
`unmanaged` and names the commands that correct it:

```text
MODEL                          STATE                    REACH                     LAST PING
my-model                       running, not registered  via chap-core             -
chapkit-minimalist-example-py  unmanaged                http://a75691bfa55d:8000  10s ago

1 of 1 model is not registered.
my-model: its container registered as `chapkit-minimalist-example-py`, the unmanaged row above; run `chap-devops models remove my_model`, then `chap-devops models add my-model:dev --service-id chapkit-minimalist-example-py`
```

A reference with no registry host (`my-model:dev`, `me/my-model:dev`) is read
as an image in the local image store:

- **It has to be there already.** `models add` reads the user and data
  directory from the local image. If the image is missing, it refuses and
  gives the build command:
  `error: nope-model:dev is not in the local image store for linux/amd64; build it with ...`.
- **It is never pulled.** The overlay says `pull_policy: never`, so `chap-devops up`
  starts what is in the store, and `chap-devops update` shows it as
  `pinned, skipped`.
- **`--platform linux/amd64`** matches the platform every model overlay pins.
  On an Apple Silicon Mac the build runs under emulation and takes longer.
- **Rebuilding** with the same tag and running `chap-devops restart` starts the new
  build. `chap-devops restart` prints `recreated chapkit-minimalist-example-py`.

## A local build of a marketplace model

If the model is one the marketplace already lists (you are working on it),
give the local build the marketplace model's service id, so it registers
under the name chap-core expects. Build the image in the model's checkout,
for example `chapkit_simple_multistep_model`:

```sh
docker build --platform linux/amd64 --build-arg GIT_REVISION=$(git rev-parse HEAD) -t multistep:dev .
```

Then, in the deployment:

```sh
chap-devops models disable chapkit_simple_multistep_model          # if it is enabled
chap-devops models add multistep:dev --service-id chapkit-simple-multistep-model
chap-devops up
```

The two cannot run at the same time. If the marketplace model is enabled,
`chap-devops models add` refuses:

```text
error: the marketplace model chapkit_simple_multistep_model is enabled as the compose service `chapkit-simple-multistep-model`; disable it with `chap-devops models disable chapkit_simple_multistep_model` to run this local build in its place
```

In the same way, `chap-devops models enable chapkit_simple_multistep_model` refuses while the
local build holds the service, and names `chap-devops models disable multistep`. To go
back to the published image:

```sh
chap-devops models remove multistep
chap-devops models enable chapkit_simple_multistep_model
chap-devops up
```

`chap-devops models disable` and `chap-devops models remove` keep the model's data volume
and print the `docker volume rm` command that removes it. `chap-devops down --volumes`
also removes that volume, because it has the compose project label of the
deployment. It warns about each volume with the deployment prefix that it did
not remove.

More: [Models outside the marketplace](../models.md#models-outside-the-marketplace).

All shapes: [Use cases](../use-cases.md).
