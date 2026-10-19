# Chap with forecasting models

The default, and what most deployments are: chap-core plus the models it runs.

```sh
chap-devops init mychap --models chapkit_simple_multistep_model
cd mychap
chap-devops up
chap-devops status                 # chap-core up, every model registered
chap-devops models test --all      # make each model train and predict once
```

It worked when `chap-devops status` shows chap-core `up` and every model
registered:

```text
chap-core   up   http://localhost:8700   2.4.0   auth: off

MODEL                           STATE                       REACH          LAST PING
chapkit-simple-multistep-model  registered, not configured  via chap-core  1s ago

1 model registered
chapkit-simple-multistep-model: chap-core has no configured model for it, so nothing can run it; run `chap-devops models configs sync`
```

and `chap-devops models test --all` says every model passes:

```text
testing 1 model (model level; add --backtest to go through chap-core)
chapkit-simple-multistep-model    pass    5s   1 training, 1 prediction

1 of 1 model passes
```

`registered, not configured` is the normal state after a plain `chap-devops up`.
chap-core v2.4.0 makes no configured model when a model registers, and the
model level of `chap-devops models test` does not need one. A backtest and the
Modeling App need one. The [backtest](#test-through-chap-core) below creates
them, and so do `chap-devops models configs sync` and `chap-devops up --wait`. After that,
`chap-devops status` shows the model as `registered`. See
[Configured models](../models.md#configured-models).

You get chap-core's API on port 8700 and one service per model. The models
register with chap-core over the compose network and publish no host port.
You reach them through chap-core at
`http://localhost:8700/v2/services/<service_id>/run/`.

To change the models later, use `chap-devops ui`, or `chap-devops models enable ID` and
`chap-devops models disable ID`. `disable` stops the model at once. After `enable`,
run `chap-devops up` to start the model.

## Every model in the marketplace

`all` is every model the marketplace lists (templates aside), and a list of
ids picks some. `chap-devops models list` prints them. Without `--models`, `init`
enables no model.

This is a new deployment. It needs port 8700 too. If the deployment from the
first section is still up, go to `mychap` and run `chap-devops down --volumes --yes`
first, then `cd ..`. `chap-devops init` then warns that `mychap` also uses port
8700. While `mychap` is down, you can ignore that warning.

```sh
chap-devops init allmodels --models all
cd allmodels
chap-devops up
chap-devops status                 # again, until every model is registered
chap-devops models test --all      # make each model train and predict once
```

The model images are large: four images of 1 to 7 GB each, all linux/amd64
only. `chap-devops init` already pulls one of them to read its user, and says
``asking ghcr.io/chap-models/chapkit_ghr_model:sha-dfb2e3f what uid `app` is (this pulls the image)``.
The first `chap-devops up` pulls the others, which can take many minutes.

`chap-devops up` returns before the models register. Right after it, `chap-devops status`
can show some or all models as not registered yet, and exits with 1:

```text
MODEL                             STATE                    REACH          LAST PING
auto-arima-chapkit                running, not registered  via chap-core  -
...
chapkit-simple-multistep-model    running, not registered  via chap-core  -

4 of 4 models are not registered.
auto-arima-chapkit: started under two minutes ago and registers once it is ready; run `chap-devops status` again in a minute
...
```

If you see this, wait a minute and run `chap-devops status` again. On an M2 Max,
all four models registered in less than a minute. It worked when `chap-devops status`
ends with `all 4 models registered`, followed by one
`chap-core has no configured model for it` line for each model, as in the
first section. `chap-devops models test --all` then says:

```text
testing 4 models (model level; add --backtest to go through chap-core)
auto-arima-chapkit                  pass   10s   1 training, 1 prediction
chapkit-ghr-model                   pass   24s   1 training, 1 prediction
chapkit-rwanda-malaria-bym-model    pass   20s   1 training, 1 prediction
chapkit-simple-multistep-model      pass    6s   1 training, 1 prediction

4 of 4 models pass
```

On an Apple Silicon Mac the models run under emulation; the four tests above
took about 60 seconds on an M2 Max.

## Test through chap-core

`chap-devops models test --all --backtest` tests each model through chap-core, the
way the Modeling App does. It makes a dataset from the sample data of the
model, runs a backtest on it and reads the scores. A backtest needs a
configured model, so the command first creates the configured models from the
marketplace entry of each model, as `chap-admin install` does. You make no
API call yourself.

```sh
chap-devops models test --all --backtest
```

It worked when every model passes with its scores, and the closing lines name
the configured models it created:

```text
testing 4 models (through chap-core: a dataset, a backtest and its scores)
auto-arima-chapkit                  pass      18s   crps 3.4  mae 4.9  rmse 5.8
  configured model: monthly (id 14)
chapkit-ghr-model                   pass   1m 33s   crps 14.7  mae 21.8  rmse 23.8
  configured model: monthly_climate (id 15)
chapkit-rwanda-malaria-bym-model    pass      38s   crps 16.3  mae 21.5  rmse 26.5
  configured model: monthly (id 16)
chapkit-simple-multistep-model      pass      13s   crps 3.9  mae 5.3  rmse 6.8
  configured model: monthly_climate (id 17)

created configured models in chap-core for auto_arima_chapkit (monthly), chapkit_ghr_model (monthly_climate), chapkit_rwanda_malaria_bym_model (monthly), chapkit_simple_multistep_model (monthly_climate, monthly_selfhistory)
4 of 4 models pass
```

The four backtests took about three minutes on an M2 Max. The scores are not
the same from one run to the next. A second run creates no configured models,
so it has no `created configured models` line. After the first run,
`chap-devops status` shows each model as `registered`.

Next: [Models and the marketplace](../models.md), [Authentication](../auth.md)
before you put it on a network, [Backup and restore](../backup.md).

All shapes: [Use cases](../use-cases.md).
