# A DHIS2 from chap-devops, with a chap-core elsewhere

For chap-core development with the Modeling App in front of it: chap-devops runs a
DHIS2 (and any models), and chap-core runs somewhere else, usually from its own
checkout on this machine.

1. Start chap-core and make sure that it answers at `http://localhost:8000`.
   - From its checkout: make it listen on `0.0.0.0:8000`, not only on the
     loopback.
   - From a second chap-devops deployment: run the commands below.

   ```sh
   chap-devops init core --only chap-core --models none --api-port 8000
   cd core
   chap-devops up
   chap-devops status                 # wait for the chap-core line to say up
   cd ..
   ```

2. Make the DHIS2 deployment and connect it to that chap-core:

   ```sh
   chap-devops init lab --chap-core-url http://localhost:8000 --with dhis2
   cd lab
   chap-devops up
   chap-devops status                 # wait for the dhis2 line to say up
   chap-devops dhis2 connect
   chap-devops open dhis2
   ```

If chap-core is a container (the second deployment above, or `make restart` in
the chap-core checkout), `init` gives this warning:

```text
warning: chap-core at http://localhost:8000 is the container `core-f380cb-chap-1`, so it calls the models back at host.docker.internal; if it is a process on this machine instead, run `chap-devops components enable chap-core --url http://localhost:8000 --models-host localhost`
```

The warning is about models only. chap-devops found the container, so the models
register with the correct address. A deployment with no models can ignore it.

`chap-devops up` returns after some seconds, before DHIS2 answers. The first `chap-devops
up` pulls the DHIS2 images, about 2 GB, and downloads the seed dump. Then
DHIS2 needs some seconds to some minutes before it answers. Until then,
`chap-devops status` shows `dhis2   starting`.

With `--chap-core-url`, `init` enables no model unless you name one. Add
`--models ID,ID` to start models that register with that chap-core. If a model
cannot register with a chap-core elsewhere, `init` gives a warning that names
it.

`--chap-core-url` records the chap-core elsewhere (see
[chap-core from its checkout, with the models](./chap-core-on-host.md)), and
for the DHIS2 beside it:

- **The route goes to that chap-core as the DHIS2 container reaches it.** A
  `localhost` URL becomes `host.docker.internal`, and the DHIS2 service maps
  that name to the host gateway.
- **`dhis2/dhis.conf` allows it.** A scaffolded file allows every http and
  https target in `route.remote_servers_allowed`, so the chap-core elsewhere is
  covered wherever it runs.
- **chap-core has to listen on more than the loopback** (`0.0.0.0`), since the
  route arrives from a container. A chap-core in a container publishes its
  port on all addresses already.

On the seed database, `chap-devops dhis2 connect` printed this:

```text
DHIS2 2.42.6 at http://localhost:8780, as `admin`
repointed the `chap` route at http://host.docker.internal:8000/**; chap-core answered through it
installed Modeling App 7.2.0
installed DHIS2 Climate App 1.16.2
analytics finished in 20 seconds (job AyL1NbEW3ot)
the Modeling App can reach Chap; open DHIS2 with `chap-devops open dhis2`
```

The seed database has a `chap` route already, so the second line says
`repointed`. On a DHIS2 with no `chap` route, it says `created`.

If chap-core does not answer, `connect` still sets the route, but it gives a
warning and ends with another line:

```text
warning: the `chap` route is in place but nothing answered through it: HTTP 500 Internal Server Error: finishConnect(..) failed with error(-111): Connection refused: host.docker.internal/192.168.65.254:8000; run `chap-devops status` to see whether chap-core is up
...
run `chap-devops dhis2 show` to see what is still missing
```

If you see this warning, start chap-core and run `chap-devops dhis2 connect` again.
See [The route is there but nothing answers through it](../troubleshooting.md#the-route-is-there-but-nothing-answers-through-it).

`chap-devops open dhis2` opens DHIS2 in the browser; with `--no-browser`, it prints
the address. Log in as `admin` with the password `district`. The Modeling App
is `Modeling` in the apps menu.

It worked when `chap-devops dhis2 connect` says `chap-core answered through it` and
ends with `the Modeling App can reach Chap`. In the Modeling App, the Settings
page then shows the chap-core version and `Connected`.

More: [DHIS2](../dhis2.md), [A chap-core elsewhere](../components.md#a-chap-core-elsewhere).

All shapes: [Use cases](../use-cases.md).
