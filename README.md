# DynDNS Client for cloudflare

Gets your current public ip address (IPv4/IPv6) and updates the specified cloudflare dns records.

## Configuration:

Configuration of the client is done via a `toml` config file.
By default, the client will load its config from `config.toml` in the working directory, however this can be changed by providing the `-c <PATH TO CONFIG>` command line argument.

### Example `config.toml`:
```toml
ipv4_service = "https://api.ipify.org" # Api that returns the current ipv4 address
ipv6_service = "https://api64.ipify.org" # Api that returns the current ipv6 address

[[zones]] 
identifier = "<YOUR ZONE ID HERE>"

[zones.auth]
BearerAuth = "<YOUR ZONE AUTHENTICATION HERE>"

# There can be multiple search rules. All records matching any of the rules will be changed
[[zones.search]] # Updates any A-type record with name "test.mydomain.net"
type = "A"
name = "test.mydomain.net"

[[zones.search]] # Updates any AAAA-type record
type = "AAAA"
```

As any record matching either of the two search rules in example above are updated, this would mean:
1. The A-Record "test.mydomain.net" will be updated (if it exists)
2. All AAAA-Records of the zone will be updated

### Configuration Reference:
- Configuration file structure:
    | Name           | Type             |
    | -------------- | ---------------- |
    | `ipv4_service` | *optional* url   |
    | `ipv6_service` | *optional* url   |
    | `zones`        | *list of* `Zone` |

    *Notes*: Atleast one of ipv4_service and ipv6_service must be set and zones must have atleast one entry
- **Zone**:
    | Name         | Type                 |
    | ------------ | -------------------- |
    | `identifier` | string               |
    | `auth`       | **`Authentication`** |
    | `search`     | *list of* `Rule`     |

- **Authentication** (either of):
    1. Using Bearer Authentication:
        | Name         | Type             |
        | ------------ | ---------------- |
        | `BearerAuth` | string           |
    2. Using ApiKey Authentication:
        ***todo***
- **Rule**:
    | Name        | Type                     |
    | ----------- | ------------------------ |
    | `comment`   | *optional* `StringMatch` |
    | `content`   | *optional* string        |
    | `match`     | *optional* **`Match`**   |
    | `name`      | *optional* string        |
    | `proxied`   | *optional* bool          |
    | `search`    | *optional* string        |
    | `tag`       | *optional* `StringMatch` |
    | `tag_match` | *optional* **`Match`**   |
    | `type`      | *optional* string        |

- **StringMatch**:
    | Name         | Type              |
    | ------------ | ----------------- |
    | `exact`      | *optional* string |
    | `absent`     | *optional* bool   |
    | `contains`   | *optional* string |
    | `endswith`   | *optional* string |
    | `present`    | *optional* bool   |
    | `startswith` | *optional* string |
- **Match** (either of):
    1. `"any"`
    2. `"all"`

## Experimental automatic update feature:
Using the update feature allows for the use of the `-u` command line argument which checks the newest release on GitHub.
If the version of the executable differs from the latest git release, it swaps the binaries and restarts, allowing for a very crude automatic update function.