
## To use Tau-Tower on a remote server - accompanying Asciinema

----

### Setting up Tau with Caddy

We use `Caddy` to reverse proxy the broadcast stream. The radio broadcast will
be located at `https://example.com/tau.ogg`.

Make sure that your VPS or remote server has the correct ports open to receive
and broadcast the radio stream. __You should wisely restrict which IP can send
to the receive port__, most likely from the VPS settings. 

``` Caddyfile
example.com {
    reverse_proxy :6001
}

asciinema.example.com {
    reverse_proxy :4000
}

:8001 {
    reverse_proxy :6000
}
```

Below is an updated config file for `tau-tower` which uses the ports set up
in the `Caddyfile` above. 

``` tower.toml
# username and password are NOT secure.
username = "username" 
password = "emanresu" 

# local listen port
listen_port = 6000

# local broadcast port
broadcast-port = 6001      

broadcast-endpoint = "tau.ogg"       

# When Asciinema server is used, we need to allow it to fetch the stream
cors_allow_list = ["https://asciinema.example.com"]
```

In the `tau-radio` client config, we keep our previous config, assuming the
remote server will receive on the port we have set. 
``` config.toml
username = "username"
password = "emanresu"

# IP to the remote server
ip = 111.222.33.44

# The port which Caddy has exposed for us
transmit-port = 8001

# Default audio interface on macOS
audio_interface = "BlackHole 2ch"

broadcast-endpoint = "tau.ogg"
```

----

### Asciinema setup

Below is a copy from the [`Getting started`](https://docs.asciinema.org/getting-started/#self-hosting-the-server)-section from the Asciinema Documentation. 

----

While asciinema.org is the default asciinema server used by the CLI for uploading recordings, you can self-host your own instance if you want full ownership and control over the recordings.

asciinema server is packaged as OCI container image and is available at ghcr.io/asciinema/asciinema-server.

Here's a minimal docker-compose example:

```yml
services:
  asciinema:
    image: ghcr.io/asciinema/asciinema-server:latest
    ports:
      - '4000:4000'
    volumes:
      - asciinema_data:/var/lib/asciinema
    depends_on:
      postgres:
        condition: service_healthy

  postgres:
    image: docker.io/library/postgres:14
    volumes:
      - postgres_data:/var/lib/postgresql/data
    environment:
      - POSTGRES_HOST_AUTH_METHOD=trust
    healthcheck:
      test: ['CMD-SHELL', 'pg_isready -U postgres']
      interval: 2s
      timeout: 5s
      retries: 10

volumes:
  asciinema_data:
  postgres_data:
```

----

Start it with:

```bash
docker compose up
```

Then point asciinema CLI to it by setting ASCIINEMA_API_URL environment variable:
(Set env variable properly if the intended use is a public-facing asciinema server. 
Follow the full guide on https://docs.asciinema.org).

```
# export ASCIINEMA_API_URL=https://asciinema.example.com

# localhost example
export ASCIINEMA_API_URL=http://localhost:4000

asciinema rec demo.cast
asciinema upload demo.cast
```

Note that the above configuration should be used only for testing the server locally. See full [`server self-hosting`](https://docs.asciinema.org/manual/server/self-hosting/) guide to learn how to set it up properly in a full-featured and secure way.

