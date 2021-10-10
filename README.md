# newsfrwdr

Checks inputs for new entries and forwards them to outputs (based on name/tag). For now, the only inputs it supports are rss feeds.

Inspired by: [rss-forwarder](https://github.com/morphy2k/rss-forwarder)

## Supported outputs

- [x] custom command
- [x] discord webhook
- [x] discord bot
- [x] slack webhook
- [ ] telegram bot

## Usage

### Docker

- `docker run -d -v "/path/to/config.toml:/config/config.toml" ghcr.io/rdk31/newsfrwdr:master`

### Command line

```
Usage: newsfrwdr [OPTIONS]

Optional arguments:
  -h, --help           print help message
  -c, --config CONFIG  alternative path to config.toml
```

## Configuration

### Example configuration:

```toml
[inputs.rust-blog]
url = "https://blog.rust-lang.org/feed.xml"

[inputs.github-blog]
url = "https://github.blog/all.atom"
tags = ["it"]

[inputs.rdk31]
url = "https://rdk31.com/atom.xml"

[[outputs.default]]   # default output
type = "discord_webhook"
url = "https://discord.com/api/webhooks/abcd..."

[[outputs.rust-blog]] # name output
type = "discord_bot"
token = "token"
user_id = 123456789

[[outputs.it]]        # tag output
type = "discord_webhook"
url = "https://discord.com/api/webhooks/ijkl..."

[[outputs.it]]        # forward the same tag to another channel
type = "custom"
command = "notify-send"

[[outputs.github-blog]]
type = "slack"
url = "https://hooks.slack.com/services/..."
```

### Inputs

| Field         |   Type   | Required |   Default   | Description            |
| ------------- | :------: | :------: | :---------: | ---------------------- |
| key           |  string  |   yes    |      -      | input name             |
| `url`         |  string  |   yes    |      -      | url to the feed        |
| `interval`    |  string  |    no    |    "30m"    | feed refresh interval  |
| `retry_limit` |   int    |    no    |     10      | feed fetch retry limit |
| `tags`        | [string] |    no    | ["default"] | array of tags          |

### Outputs

#### `discord_webhook` type

| Field  |  Type  | Required | Default | Description                  |
| ------ | :----: | :------: | :-----: | ---------------------------- |
| key    | string |   yes    |    -    | input name or tag to forward |
| `type` | string |   yes    |    -    | output type                  |
| `url`  | string |   yes    |    -    | discord webhook url          |

#### `discord_bot` type

| Field     |  Type  | Required | Default | Description                  |
| --------- | :----: | :------: | :-----: | ---------------------------- |
| key       | string |   yes    |    -    | input name or tag to forward |
| `token`   | string |   yes    |    -    | discord bot token            |
| `user_id` |  u64   |   yes    |    -    | user id to push entries to   |

#### `slack` type

| Field  |  Type  | Required | Default | Description                  |
| ------ | :----: | :------: | :-----: | ---------------------------- |
| key    | string |   yes    |    -    | input name or tag to forward |
| `type` | string |   yes    |    -    | output type                  |
| `url`  | string |   yes    |    -    | slack webhook url            |

#### `custom` type

Serializes entries to this json structure:

```json
{
  "title": "title",
  "description": "description",
  "author": "null or string",
  "url": "url",
  "timestamp": "ISO 8601 string"
}
```

| Field       |   Type   | Required | Default | Description                                                                                |
| ----------- | :------: | :------: | :-----: | ------------------------------------------------------------------------------------------ |
| key         |  string  |   yes    |    -    | input name or tag to forward                                                               |
| `type`      |  string  |   yes    |    -    | output type                                                                                |
| `command`   |  string  |   yes    |    -    | command to run                                                                             |
| `arguments` | [string] |    no    |   []    | command arguments                                                                          |
| `use_stdin` |   bool   |    no    |  false  | - false - add the message to the arguments array <br /> - true - push the message to stdin |
