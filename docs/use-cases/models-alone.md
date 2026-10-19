# Several model services side by side

Two or more chapkit model services in one deployment directory, with no
chap-core. Each model gets its own host port and its own data volume, and the
models do not depend on each other. For one model, see
[A chapkit model service on its own](./model-alone.md).

```sh
chap-devops init models --only none --models chapkit_simple_multistep_model,auto_arima_chapkit
cd models
chap-devops up --wait
chap-devops status
chap-devops models list --enabled  # the PORT column says where each one answers
```

`chap-devops init` gives each model the next free port from 5001 up, and prints it:

```text
created a deployment in /path/to/models
components: none
enabled chapkit_simple_multistep_model v0.1.2 on http://localhost:5001
enabled auto_arima_chapkit v1.0.2 on http://localhost:5002
run `cd /path/to/models && chap-devops up` to start the deployment
```

A first `chap-devops up` pulls the model images. On disk, the image of Simple
Multistep is about 1 GB, and the image of Auto-ARIMA about 6 GB. The pull can take some minutes. The images
are amd64 only, so on an arm64 machine they run under emulation.

`chap-devops up --wait` returns once each model answers its own `/health`. Without
`--wait`, `chap-devops up` returns when the containers start, and for some seconds
`chap-devops status` can say `running, not answering`. After compose's own lines,
`chap-devops up --wait` ends with:

```text
waiting up to 300s for the 2 models to answer
started auto-arima-chapkit, chapkit-simple-multistep-model
ready in 4s
  auto-arima-chapkit              up  http://localhost:5002
  chapkit-simple-multistep-model  up  http://localhost:5001
```

Then `chap-devops status` shows:

```text
MODEL                           STATE  REACH      LAST PING
auto-arima-chapkit              up     port 5002  -
chapkit-simple-multistep-model  up     port 5001  -

all 2 models up, each answering on its own host port
```

It worked when `chap-devops status` shows every model as `up`, each on its own port.
Each model answers on `http://localhost:<port>`, with its API documentation at
`/docs`. The ids for `--models` are in the ID column of `chap-devops models list`.
That command also works outside a deployment directory.

Compose publishes these ports on every address of the machine, and the models
have no login. To keep a model on this machine only, see
[Which address a model port is published on](../ports.md#which-address-a-model-port-is-published-on).

Without a folder, `chap-devops run` does the same one model at a time:
[Any number of model services, no folder](./models-with-run.md).

All shapes: [Use cases](../use-cases.md).
