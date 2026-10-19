# Growing a deployment

Nothing about the first `init` is final. Components go on and off with
`chap-devops components enable NAME` / `disable NAME`, models with `chap-devops models
enable ID` / `disable ID`, and `chap-devops up` applies the change.

The move people make most is from a standalone piece to Chap: a model service
or an OCS that was running alone gets chap-core added beside it. The examples
here start from a model on its own (see [A chapkit model service on its
own](./model-alone.md)):

```sh
chap-devops init multistep --only none --models chapkit_simple_multistep_model
cd multistep
chap-devops up
```

Add chap-core:

```sh
chap-devops components enable chap-core
chap-devops up
chap-devops status                 # the models now register with chap-core
```

`chap-devops components enable chap-core` prints `enabled chap-core on
http://localhost:8700`, a line that says the models keep their host ports, and
`` run `chap-devops up` to apply ``. Model services read the
orchestrator URL on that `chap-devops up` and register.
Registration takes up to a minute after the start, so the first `chap-devops status`
can show `running, not registered` and the line
`started under two minutes ago and registers once it is ready`.
If it does, run `chap-devops status` again in a minute:

```text
chap-core   up   http://localhost:8700   2.4.0   latest: running f80b66f755ef, revision 7c95b2a7df62   auth: off

MODEL                           STATE                       REACH      LAST PING
chapkit-simple-multistep-model  registered, not configured  port 5001  11s ago

1 model registered
chapkit-simple-multistep-model: chap-core has no configured model for it, so nothing can run it; run `chap-devops models configs sync`
```

`registered, not configured` means chap-core has the model but no configured
model of it yet. `chap-devops models configs sync`, or `chap-devops up --wait`, creates
them. See [Configured models](../models.md#configured-models).

The host ports they were given while standalone stay published until you
`chap-devops models unexpose ID`. The chap-core compose file comes from the copy built
into chap-devops, and the image tag is `latest`, a moving tag: the `latest: running
...` cell in `chap-devops status` names the build that runs. `chap-devops update
--pin-chap-core` moves it to the newest release, which is what `chap-devops init`
with chap-core records.

The other direction works too: `chap-devops components disable chap-core` keeps the
models, publishes a host port for each one that had none, and says where each
one now answers:

```text
$ chap-devops components disable chap-core
disabled chap-core
stopped and removed 6 containers (chap, chapkit-simple-multistep-model, chapkit-simple-multistep-model-init, postgres, redis, worker); host ports 5001, 8700 are free again
run `chap-devops up` to apply
```

A model that had no host port (after `chap-devops models unexpose`) gets one more
line, for example
`chapkit_simple_multistep_model now registers nowhere and is published on http://localhost:5001`.
The disable stops the model containers too, so run `chap-devops up` to start the
models again without chap-core.

chap-core's own volumes are kept either way. While chap-core is enabled,
`chap-devops down --volumes` removes them. After the disable it does not, because
the deployment no longer has the compose file that names them. To remove them,
run `chap-devops components enable chap-core` first, then `chap-devops down --volumes`.

It worked when `chap-devops status` shows the new pieces: chap-core `up` and the
models `registered` after adding chap-core, or each model `up` on its own port
after taking it away.

The same goes for any other piece: `chap-devops components enable ocs`,
`chap-devops components enable dhis2` or `chap-devops models enable ID`, then `chap-devops up`. A
DHIS2 added this way runs the default version unless you name one:
`chap-devops components enable dhis2 --tag 2.43` (or `--image dhis2/core-dev --tag
master` for the unreleased one), or `v` on its row in `chap-devops ui` (see [Changing the DHIS2 version](../dhis2.md#changing-the-dhis2-version)).
Only the default version has a demo database. Another version starts empty, and
the enable says so in a `warning:` line. If the deployment has chap-core, run
`chap-devops dhis2 connect` once DHIS2 answers (see
[Connecting the Modeling App to Chap](../dhis2.md#connecting-the-modeling-app-to-chap)).

All shapes: [Use cases](../use-cases.md).
