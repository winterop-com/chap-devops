# The chapkit CLI

chapkit is the framework of the model services. Its command line, `chapkit`,
creates a model project, changes an `MLproject` model to a chapkit service and
back, and serves an `MLproject` model with no image to build. It is usually installed with uv, which needs Python on the machine.

`varde chapkit` runs the same `chapkit` in a container. You need Docker and
nothing more:

```sh
varde chapkit mlproject migrate my-model
varde chapkit mlproject run my-model
```

Each argument after `chapkit` goes to chapkit unchanged. To read chapkit's own
help, run `varde chapkit` with no arguments. `varde chapkit --help` shows only
the options of varde itself.

## Which image

chapkit is a dependency of chap-core, so the chap-core image has the chapkit
CLI. varde runs it in that image, and there is no other image to pin or to
update. The chap-core image v2.4.0 has chapkit 2.1.0. From v2.5.0 it has
chapkit 2.4.0, which adds `chapkit mlproject export` and writes the full
MLproject contract with `migrate`. In a deployment on an older tag, add
`--tag v2.5.0` for these.

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

chapkit selects the base image of the service from the scripts at the root of
`DIR`: R scripts give an R image, Python scripts give `chapkit-py`. If the
scripts are in a subdirectory, as in the `mlproject-*` templates of
`varde models new`, chapkit stops with `No Python (.py) or R (.r/.R) scripts at
the project root; cannot pick a base image.` Then give the image:

```sh
varde chapkit mlproject migrate my-model --yes --base-image chapkit-py
```

The values are `chapkit-py`, `chapkit-r`, `chapkit-r-tidyverse` and
`chapkit-r-inla`. The `README.md` of a project from an `mlproject-*` template
gives the command with the correct value.

On chap-core `master` before v2.5.0 (chapkit 2.4.0), these models were
migrated, built, and passed the model-level test with 1 training and 1
prediction: `minimalist_configurable_model` and `mean` from chap-models, and
the `mlproject-py` and `mlproject-r` templates with `--base-image`. On
v2.5.0, the `mlproject-py` template passed again.

## Export a service as an MLproject

`chapkit mlproject export` reads the `main.py` of a chapkit service, and
writes the `MLproject` file that describes the same model. It needs chapkit
2.4.0, so it needs chap-core v2.5.0 or later:

```sh
cd my-service
varde chapkit mlproject export main.py -o MLproject
```

chapkit imports `main.py` to read the service. This works in the chap-core
image when `main.py` imports only chapkit and the Python standard library,
as a service that runs scripts does. If `main.py` imports the packages of
the model, the import can fail in the chap-core image.

Only a service that runs scripts can be exported, such as one from the
`shell-py` or `shell-r` template. For a model in `main.py` (the `fn-py`
template), chapkit stops with `Only ShellModelRunner services can be
exported`.

The exported `MLproject` gives the train and predict commands a fixed
`config.yml`, which chapkit writes. So it runs where chapkit runs it: with
`varde chapkit mlproject run`, and with `varde chap --run-config.as-chapkit`.
Without `--run-config.as-chapkit`, chap does not write `config.yml`, and the
train command stops with `No such file or directory: 'config.yml'`.

chapkit 2.4.0 also writes a warning that the `author_note` of the
`MLproject` and of the service are not the same, when `main.py` has a
`description` and no `author_note`. The `MLproject` is correct, and you can
ignore the warning.

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
| `py-data` | `ghcr.io/dhis2-chap/chapkit-py-data-cli`, with numpy, pandas, scipy, scikit-learn, statsmodels, joblib, pyyaml and xarray |
| `r` | `ghcr.io/dhis2-chap/chapkit-r-cli` |
| `r-tidyverse` | `ghcr.io/dhis2-chap/chapkit-r-tidyverse-cli` |
| `r-inla` | `ghcr.io/dhis2-chap/chapkit-r-inla-cli` (amd64 only) |
| any other value | That image, such as one you built yourself |

`varde chapkit --image`, with no value, lists the short names, what each image
has, and whether this docker has it already.

Without `--image`, the train command in `MLproject` selects the image: a
`python` or `uv` command gets `py-data`, and an `Rscript` command gets
`r-tidyverse`. A model that needs INLA, or packages that no `-cli` image has,
needs `--image`. An image of your own must have chapkit 2.4.0 or later, such
as an image built `FROM` a `-cli` image. varde pulls the image if this docker
does not have it.

varde mounts the model directory at `/work` and publishes the service on the
first free port from 5001 (in a deployment, from its port range). It prints
the URL when the service answers at `/health`. The logs follow in the
terminal, and Ctrl-C stops the service and removes its container. A `kill`
of the varde process does the same, and so does `docker stop` of the
container.

The image has chapkit and the packages in the table above. varde does not
install the dependencies that the `MLproject` names in `uv_env` or
`renv_env`. If the model needs a package that the image does not have, the
train job stops with an error such as `No module named 'cyclopts'`. Then
give `--image` with an image that has the package, or let chap install
them with `varde chap --run-config.as-chapkit`
([The chap CLI](./chap-cli.md#an-mlproject-as-a-chapkit-service)).

If the model directory is a git checkout, varde gives chapkit its commit as
`GIT_REVISION`, as the `docker build` of an image does. chap-core stores a
model only with a revision. If the directory has changes that are not
committed, varde warns: the service reports the last commit, so commit the
changes and raise `version` in `MLproject` first.

The options of `chapkit mlproject run` after `DIR` go to chapkit unchanged,
such as `--config-format flat` or `--log-level debug`. A relative `--database`
path is in the model directory. varde sets `--host` and `--port` itself.

In a deployment that runs chap-core, the service joins the deployment network
and registers with chap-core while it runs, with the registration key from
`.env`. Then the commands of the deployment use it by its id, the `name` in
`MLproject`:

```sh
varde models configs sync my-model
varde models test my-model --backtest
```

`varde status` does not show the service, because varde does not manage it.

Outside a deployment, or with an external chap-core, it does not register;
use the URL that varde prints. `varde chap` runs in a container, where
`localhost` is the container, so give it the same port at
`host.docker.internal`:

```sh
varde chap eval --model-name http://host.docker.internal:5001 \
  --dataset-csv laos_subset.csv --output-file my-model.nc
```

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
