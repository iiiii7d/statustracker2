# StatusTracker 2

![GitHub release (latest SemVer including pre-releases)](https://img.shields.io/github/v/release/iiiii7d/statustracker2?include_prereleases)
[![pages-build-deployment](https://github.com/iiiii7d/statustracker2/actions/workflows/pages/pages-build-deployment/badge.svg)](https://github.com/iiiii7d/statustracker2/actions/workflows/pages/pages-build-deployment)

Track players on a Minecraft server through their Dynmap

Made by 7d for the Minecart Rapid Transit server

## Usage

1. Create a MongoDB cluster and get a URI
2. Install the server with cargo: `cargo install --git https://github.com/iiiii7d/statustracker2.git --tag v{version no.}` (for latest commit, omit --tag flag)
3. Create a toml file for the configuration with the following contents:

   ```toml
   # Find a JSON file hosted by dynmap that starts with "currentcount" as a key
   dynmap_link = "https://your.server/path.to.json"
   # A name of an environment variable that contains the URI of your MongoDB cluster, or the URI itself
   mongodb_uri = "MONGO"
   # Statustracker 2 uses only one database in your cluster
   database_name = "server"
   # Optional, `true` if the server is hosted over HTTP (not S). This will affect the redirect
   hosted_over_http = false

   # Optional, a mapping of category name to player UUIDs, these would show up as separate lines in the graph on the client.
   # Don't use `all` as a category name.
   [categories]
   staff = ["(uuid1)", "(uuid2)"]
   ```

4. Run `statustracker-server <config_file_name>`
5. The server uses Rocket, additional configuration for the server framework itself goes in [Rocket.toml](https://rocket.rs/v0.4/guide/configuration/#rockettoml) (if in production, you may need to set `address = "0.0.0.0"`)
6. Enter the URL of the site that the server is hosted on, and it should redirect to the client for StatusTracker 2

## Usage (Docker)

1. Create a MongoDB cluster and get a URI
2. Pull the image with `docker pull ghcr.io/iiiii7d/statustracker2:v<version>`
3. Create a toml file for the configuration with the contents above. Save it as `statustracker.toml` in a new folder.
4. Create `Rocket.toml` like the above, if applicable. Save it in the same folder.
5. Run the image with `docker run -dp <host_port>:8000 -v <host_config_path>:/cfg ghcr.io/iiiii7d/statustracker2:v<version>`
   - `<host_port>` is the host's port
   - `<host_config_path>` is the path of the folder containing `statustracker.toml` (and `Rocket.toml`)
   - If there are environment variables, put them in a `.env`, then add `--env ./.env` before `ghcr.io`
6. Enter the URL of the site that the server is hosted on, and it should redirect to the client for StatusTracker 2
