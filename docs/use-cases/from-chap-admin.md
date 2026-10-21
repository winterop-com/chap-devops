# Taking over a chap-admin deployment

A Chap server that runs chap-core's own compose files, with models installed
by `chap-admin`, can move to chap-devops. `chap-devops init --adopt` writes a
new deployment directory that uses the same database. The old directory is
read and never written, so you can go back to it.

## What comes over

| From the old directory | In the new deployment |
| --- | --- |
| The compose project name (the directory name, or `COMPOSE_PROJECT_NAME` in `.env`) | The same name, so chap-core uses the same volumes: the database in `<project>_chap-db`, and the logs and runs. |
| `POSTGRES_USER`, `POSTGRES_PASSWORD`, `POSTGRES_DB` in `.env` | The same values in the new `.env`, so the password matches the database. |
| `CHAP_API_TOKEN` and `SERVICEKIT_REGISTRATION_KEY` | The same token and key. A token without a key gets a new key. |
| `CHAP_IMAGE_TAG` | The chap-core tag, unless `--chap-tag` names another. |
| Port 8000, where chap-core's compose files publish the API | Port 8000, unless `--api-port` names another. |
| Each model in `compose.marketplace.yml` | A model of the new deployment, with its data copied (see below). |

Each model comes over the way it ran:

- A marketplace model at the stable version follows the stable channel, as
  `chap-admin update-all` moved it.
- A marketplace model at an older version stays at that version. Run
  `chap-devops models enable ID --channel stable` to follow the marketplace.
- A model the marketplace no longer lists, such as CHAP-EWARS, is added with
  `chap-devops models add` at the image it ran.
- A custom image (`chap-admin install --image`) is added with
  `chap-devops models add`, under the template name it registered with.

The data volume of each model is copied to the name chap-devops uses,
`<project>_ck_<id>_data`. The old volume stays. Chap itself keeps its model
templates and configured models in the database, so the models are configured
as they were.

Other settings of the old `.env` do not come over. The command names each one,
so that you can copy the line into the new `.env`.

## Steps

1. Stop the old deployment. `down` removes the containers and keeps the
   volumes:

   ```sh
   cd /srv/chap
   docker compose -f compose.yml -f compose.marketplace.yml down
   ```

2. Write the new deployment:

   ```sh
   chap-devops init /srv/chap-devops --adopt /srv/chap
   ```

   ```text
   took over the deployment in /srv/chap (compose project chap)
   copied the data of chapkit_simple_multistep_model from chap_marketplace-chapkit-simple-multistep-model-data to chap_ck_chapkit_simple_multistep_model_data
   ```

3. Start it, and check it:

   ```sh
   cd /srv/chap-devops
   chap-devops up
   chap-devops status
   ```

It worked when `chap-devops status` shows chap-core `up` and each model
registered. The Modeling App shows the same models and the same backtests and
predictions as before.

To go back, run `chap-devops down` in the new directory, then start the old
deployment again with its own compose command. Both use the same database.

## What it refuses

- A deployment that still has containers: the two would use one database.
  Run the `docker compose -p <project> down` command in the message.
- A project name with no `<project>_chap-db` volume: chap-core would start
  with an empty database. If the deployment ran under another name, set
  `COMPOSE_PROJECT_NAME` in the old `.env` to that name.
- `--models`, `--interactive`, `--source`, `--chap-core-url`, `--api-token`,
  `--fresh-env` and `--no-env`: the takeover decides those itself.
- A target directory that is a deployment already.

If docker cannot be asked, `init --adopt` writes the deployment, copies no
model data, and says so. Stop the old deployment before `chap-devops up`.
