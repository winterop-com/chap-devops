# The chapkit CLI

chapkit is the framework of the model services. Its command line, `chapkit`,
creates a model project and migrates an `MLproject` model to a chapkit
service. It is usually installed with uv, which needs Python on the machine.

`varde chapkit` runs the same `chapkit` in a container. You need Docker and
nothing more:

```sh
varde chapkit mlproject migrate my-model
varde chapkit --version
```

Each argument after `chapkit` goes to chapkit unchanged. To read chapkit's own
help, run `varde chapkit` with no arguments. `varde chapkit --help` shows only
the options of varde itself.

## Which image

chapkit is a dependency of chap-core, so the chap-core image has the chapkit
CLI. varde runs it in that image, and there is no other image to pin or to
update. The chap-core image v2.4.0 has chapkit 2.1.0.

The version follows the rules of `varde chap`: in a deployment, the
deployment's chap-core tag; outside a deployment, the newest chap-core
release; `--tag TAG` overrides both. See
[Which version](./chap-cli.md#which-version). The image is amd64 only, so on
an arm64 machine, such as a Mac with Apple silicon, it runs under emulation.

`chapkit mlproject run` is the exception: it runs in a chapkit `-cli` image.
See [Serve an MLproject model](#serve-an-mlproject-model).

## Your files

varde mounts the current directory in the container at the same path, and the
container runs as your user. So the paths in the arguments mean the same in
the container, and the files that chapkit writes are yours. A path outside the
current directory is mounted too. See [Your files](./chap-cli.md#your-files).

## Migrate an MLproject model

`chapkit mlproject migrate DIR` makes a chapkit service from the `MLproject`
file in `DIR`. It moves the files that the service does not need to `_old/`,
and writes `main.py`, `pyproject.toml`, `Dockerfile` and the other files of a
chapkit project. Without `--yes`, it asks first; in a terminal, you answer the
question as with a local chapkit.

```sh
varde chapkit mlproject migrate my-model --yes
```

The output ends with the lines of varde:

```text
chapkit finished; it wrote my-model/.dockerignore, my-model/.gitignore, my-model/CHAPKIT.md, my-model/Dockerfile, my-model/Makefile, my-model/README.md, my-model/_old/.gitignore, my-model/_old/MLproject, and 7 more
wrote my-model/uv.lock with `uv lock` in the same image, so `docker build` works
```

The `Dockerfile` of a chapkit project copies `uv.lock`, and chapkit tells you
to run `uv lock` on your machine. varde does this for you: when the run writes
a `pyproject.toml` and the `Dockerfile` next to it copies a `uv.lock` that is
not there, varde runs `uv lock` in the same image.

Then build the image and test it, as for a
[new model project](./models.md#from-the-project-to-a-backtest):

```sh
cd my-model
docker build --platform linux/amd64 --build-arg GIT_REVISION=$(git rev-parse HEAD) -t my-model:dev .
varde run my-model:dev
varde -C ~/.local/share/varde/run/default models test my_model
```

This was tested with `minimalist_configurable_model` from chap-models, on
chap-core v2.4.0 (chapkit 2.1.0): the migrated image passed the model-level
test with 1 training and 1 prediction.

## Serve an MLproject model

`chapkit mlproject run DIR` serves an `MLproject` model as a chapkit service,
with no image to build:

```sh
varde chapkit mlproject run my-model
varde chapkit --image r-inla mlproject run my-model
varde chapkit --image chapkit_ghr_model:dev mlproject run .
```

The model needs its language and chapkit in one image. The chap-core image has
no R, so varde runs a chapkit `-cli` image instead:

| `--image` | Image |
| --- | --- |
| `py` | `ghcr.io/dhis2-chap/chapkit-py-cli` |
| `r` | `ghcr.io/dhis2-chap/chapkit-r-cli` |
| `r-tidyverse` | `ghcr.io/dhis2-chap/chapkit-r-tidyverse-cli` |
| `r-inla` | `ghcr.io/dhis2-chap/chapkit-r-inla-cli` (amd64 only) |
| any other value | That image, such as one you built yourself |

Without `--image`, the train command in `MLproject` selects the image: a
`python` or `uv` command gets `py`, and an `Rscript` command gets
`r-tidyverse`. A model that needs INLA, or packages that no `-cli` image has,
needs `--image`. An image of your own must have chapkit 2.4.0 or later, such
as an image built `FROM` a `-cli` image. varde pulls the image if this docker
does not have it.

varde mounts the model directory at `/work` and publishes the service on the
first free port from 5001 (in a deployment, from its port range). It prints
the URL when the service answers at `/health`. The logs follow in the
terminal, and Ctrl-C stops the service and removes its container.

The options of `chapkit mlproject run` after `DIR` go to chapkit unchanged,
such as `--config-format flat` or `--log-level debug`. A relative `--database`
path is in the model directory. varde sets `--host` and `--port` itself.

In a deployment that runs chap-core, the service joins the deployment network
and registers with chap-core while it runs, with the registration key from
`.env`. Outside a deployment, or with an external chap-core, it does not
register; use the URL that varde prints.

## What varde refuses

`chapkit test` needs a model service that the container can reach. varde
refuses it and names the command that does the work:

| Command | Use instead |
| --- | --- |
| `chapkit test` | `varde models test ID`, which runs `chapkit test` in the model's own container. |

If `uv lock` fails, varde gives a warning; see
[`could not write DIR/uv.lock`](./troubleshooting.md#could-not-write-diruvlock-and-the-dockerfile-copies-it).

For a new project, `varde models new DIR` is the same as `chapkit init`, with
the templates in varde. See [A new model project](./models.md#a-new-model-project).
`varde chapkit init NAME` also works, and gets a `uv.lock` the same way.

`--json` does not apply: the output is chapkit's own.
