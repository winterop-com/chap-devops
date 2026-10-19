# Several deployments on one machine

Each `chap-devops init` directory is a deployment of its own, with its own
containers, data and ports. Any number of them can live on one machine. They
share only the host ports, and by default every deployment asks for the same
ones: chap-core on 8700, DHIS2 on 8780 and OCS on 8790 (models from 5001 up).
There are two ways to work with that.

## One deployment at a time

Keep the default ports and run one deployment at a time. For example, compare a
DHIS2 2.42 with a 2.43:

```sh
chap-devops init d242 --only dhis2
chap-devops init d243 --only dhis2 --dhis2-tag 2.43   # warns: 8780 is also used by d242
cd d242 && chap-devops up
cd ../d243 && chap-devops up
```

The second `init` gives two warnings:

```text
warning: port 8780 is also used by d242 (/home/me/d242), which is not running; run only one of them at a time, or run `chap-devops components enable dhis2 --port 8781`
warning: chap-devops knows no DHIS2 demo dump for 2.43, so a new `dhis2_db` starts empty; name one with `chap-devops components enable dhis2 --seed URL` (a URL or a path)
```

`init` warns when another deployment that it can see uses the same port. It
sees a deployment in a directory beside the new one, and a deployment that
Docker started before. If the port is in use now, the warning is the plain
`port 8780 is already in use on this machine (needed by dhis2)` line and
names no deployment. The warning names a free port. `init` refuses nothing:
two deployments on one port can take turns. The second warning is about the
data: chap-devops has a demo database for 2.42 only, so d243 starts with an empty
DHIS2.

`chap-devops up` returns when the containers are started. DHIS2 needs more time to
answer: `chap-devops status` shows `starting` until it does, then `dhis2 is up`.

`chap-devops up` checks every port before it starts anything. When the port belongs
to another chap-devops deployment that is running, `chap-devops up` names it. At a
terminal, it prints the lines of the refusal below, then asks:

```text
d242 is using these ports. Stop it and start this deployment instead? Its data is kept. [y/N]
```

If you answer `y`, chap-devops runs `chap-devops down` on the other deployment, then starts
this one:

```text
stopping d242 (/home/me/d242) to free its ports; `chap-devops -C /home/me/d242 up` starts it again
...
started dhis2, dhis2-db
```

chap-devops deletes nothing. `chap-devops up` in the other directory starts it again, with
its database. In a script, there is no one to answer. `chap-devops up` then refuses
and names each way out:

```text
error: 1 host port this deployment needs is already in use; nothing was started
  port 8780 (needed by dhis2) is in use, and d242 (/home/me/d242) publishes it too; stop it with `chap-devops -C /home/me/d242 down`, or move this one: run `chap-devops components enable dhis2 --port 8781`
  or run `chap-devops up --no-preflight` to hand the conflict to Docker
  or run `chap-devops up --replace` to stop d242 first
```

`chap-devops up --replace` gives the answer `y` before the question:

```sh
chap-devops up --replace
```

Only a running deployment is named, and only a running one is stopped by
`--replace`. A stopped deployment that publishes the same port holds nothing,
so `chap-devops up` starts this one. If the port is in use for another reason, the
line is the plain `port 8780 is already in use on this machine (needed by
dhis2)` line, and there is no `--replace` line.

When a program that is not a chap-devops deployment holds the port, chap-devops only
reports it and does not stop that program. `chap-devops up` names the port and the
setting that moves this deployment to a different port.

## All deployments at the same time

Give each deployment its own ports. At `init`:

```sh
chap-devops init a --with ocs,dhis2
chap-devops init b --with ocs,dhis2 --api-port 8701 --ocs-port 8791 --dhis2-port 8781 --port-base 5101
```

Both deployments also get chap-core, with no model. If d242 and d243 from the section above are in the
same directory, `init a` also warns about port 8780. Stop them before you
start `a`. `init b` prints the ports it uses:

```text
chap-core: v2.4.0, API on http://localhost:8701
OCS: http://localhost:8791
DHIS2: http://localhost:8781
```

`--port-base` moves the range that model services get host ports from. It has
an effect only on models with host ports: models in a deployment without
chap-core, and models that `chap-devops models expose` publishes. In `b` above, the
model is reached through chap-core, so it has no host port.

On a deployment that already exists:

| What | How |
| --- | --- |
| chap-core's API | `CHAP_API_PORT=8701` in `.env`, then `chap-devops up` |
| OCS | `chap-devops components enable ocs --port 8791`, then `chap-devops up` |
| DHIS2 | `chap-devops components enable dhis2 --port 8781`, then `chap-devops up` |
| a model | `chap-devops models expose ID --port 5101`, or `chap-devops models unexpose ID` |

If the deployment has the port already, the command changes nothing, says so,
and gives no `chap-devops up` line:

```text
ocs was already enabled on http://localhost:8791; nothing changed
chapkit-simple-multistep-model is already exposed on http://localhost:5101; nothing changed
```

The two lines come from two commands. With `-v`, the `ocs` line has the hint
``if it is not running, `chap-devops up` starts it``. A model keeps its own port: the
port that its running container publishes does not count as a port in use.

You do not have to find a free port. When a port is in use, the warnings of
`chap-devops init` and the refusal of `chap-devops up` name the next free port in the line
to copy.

Inside each deployment, the services keep their container ports
(`http://chap:8000`, `http://ocs:9000`). Thus a change to a host port does not
change how the services reach each other. Memory is the other limit: each
DHIS2 can use 4 to 5 GB.

## Which deployment a command uses

Each command works on the deployment in the current directory, or on the one
that `-C DIR` names:

```sh
chap-devops -C d242 status
chap-devops -C d243 down
```

`chap-devops status` in each directory shows what that deployment runs and where.

More: [Ports](../ports.md), [The preflight](../ports.md#the-preflight),
[A host port is already in use](../troubleshooting.md#a-host-port-is-already-in-use).

All shapes: [Use cases](../use-cases.md).
