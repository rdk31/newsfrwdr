# newsfrwdr

Checks inputs for new entries and forwards them to outputs (based on name/tag). For now, the only inputs it supports are rss feeds.

Inspired by: [rss-forwarder](https://github.com/morphy2k/rss-forwarder)

## Supported outputs

- [x] discord webhook
- [ ] discord bot
- [ ] slack webhook
- [ ] telegram bot
- [ ] shell command

## Configuration

Example configuration:

```toml
[inputs.rust-blog]
url = "https://blog.rust-lang.org/feed.xml"

[inputs.github-blog]
url = "https://github.blog/all.atom"
tags = ["it"]

[inputs.rdk31]
url = "https://rdk31.com/atom.xml"

[[outputs.default]]   # default output
type = "discord"
url = "https://discord.com/api/webhooks/abcd..."

[[outputs.rust-blog]] # name output
type = "discord"
url = "https://discord.com/api/webhooks/efgh..."

[[outputs.it]]        # tag output
type = "discord"
url = "https://discord.com/api/webhooks/ijkl..."

[[outputs.it]]        # forward the same tag to another channel
type = "discord"
url = "https://discord.com/api/webhooks/mnop..."
```

Full configuration options:

```toml
[inputs.name]
url = "url"
interval = "30m"   # optional
retry_limit = 10   # optional
tags = ["default"] # optional

[[outputs.name/tag]]
type = "discord"   # discord
# type specific arguments, for discord:
url = "url"        # webhook url
```
