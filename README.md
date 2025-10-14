# Qtangle

An automatic post-processor for qBittorrent downloads.

## Usage

Qtangle is intended to be used as a Docker container alongside a qBittorrent instance:

```yaml
version: "3"
services:
  qtangle:
    image: ghcr.io/danlivings/qtangle:main
    container_name: qtangle
    environment:
      QTANGLE__LOG: "info"
      QTANGLE__QBITTORRENT__API_URL: "http://qbittorrent:9618"
      QTANGLE__QBITTORRENT__PASSWORD: "password" # Change this
      QTANGLE__QBITTORRENT__USERNAME: "username" # Change this, too
    volumes:
      - ./qtangle.toml:/app/qtangle.toml
      - ./test/downloads:/downloads
      - ./test/media:/app/media
    depends_on:
      - qbittorrent

  qbittorrent:
    image: lscr.io/linuxserver/qbittorrent:latest
    container_name: qbittorrent
    # Your config goes here...
```

### Configuration
Qtangle can be configured using a TOML file:

```toml
[torrent]
# How often to poll for torrent data.
poll_interval = "15s"
# Optional. Uncomment to only process torrents with the given tag.
#filter_by_tag = "qtangle"

[copy.target_folders]
# The "*" key is an optional catch-all for untagged torrents. All other keys in this section are treated as user-defined
# tags. The values correspond to a target folder.
#
# For example, using the below configuration:
# - torrents tagged "tv" will be copied to "media/TV"
# - torrents tagged "movie" will be copied to "media/Movies"
# once the download is complete.
#
# If the below key is uncommented, then torrents that aren't tagged will be copied to "media/Untagged".
#"*" = "media/Untagged"
tv = "media/TV"
movie = "media/Movies"

[delete]
# Optional. Remove torrents once they reach this seed ratio.
seed_ratio = 1

[qbittorrent]
# The base URL of your qBittorrent instance.
api_url = "http://qbittorrent:9618"
# The username to authenticate with.
username = "username"
# The password to authenticate with.
password = "password"
```

By default, Qtangle uses `qtangle.toml` in the same folder as the program itself, although this can be configured using
the `-c` or `--config-path` command-line argument.

All the configuration options available in the TOML file can also be configured through environment variables, which can
be used to avoid storing sensitive data. Qtangle's environment variables begin with the prefix `QTANGLE__` and use `__`
(double underscore) to separate namespaces; for example, `QTANGLE__TORRENT__POLL_INTERVAL` or
`QTANGLE__DELETE__SEED_RATIO`.

#### Dry run
Qtangle can be used in dry-run mode to test the configuration before you let it loose copying or deleting files. This
can be enabled using the `-n` or `--dry-run` command-line argument. In this mode, Qtangle prints out what it would do to
`stdout` but doesn't actually perform any operations.

## Building from source

You will need Rust version 1.85.0 or higher. `cargo` can be used to build and run from source:

```shell
$ cargo run -- --dry-run
```
