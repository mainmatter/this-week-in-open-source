## This-Week-in-Open-Source
It's a tool written on top of [XAMPPRocky/octocrab](https://github.com/XAMPPRocky/octocrab/tree/master/src).
It uses github's search API to find users' open source contributions.

### Usage

this-week-in-open-source is designe to run as a CLI tool either from an executable or directly with `cargo run`.

e.g `cargo run -- -after --date="2021-12-01" --users=BobrImperator` or
`GITHUB_PERSONAL_TOKEN=ghp_xxxxxxxxxx cargo run -- -after --date="2021-12-01" --users=BobrImperator`

### RateLimit
Github has a pretty small rate limit for unathorized requests, with many users or robot-people with many contributions it's really easy to go over the limit.

`GITHUB_PERSONAL_TOKEN` variable can be exported before running the program to authenticate your requests.

In order to get *Personal Access Token*:
- Click on your profile in the top-right corner.
- Go to Settings
- Go to Developer Settings
- Go to Personal access tokens
- Click Generate new token
- Check **ONLY** the `public_repo` to only allow to access "Public repositories"
- Copy your key and save it somewhere safe

### Available arguments

- `--users=user1,user2` - **REQUIRED** A list of comma separated github user names can have 1 or more entries, queries for PRs made by those users.
e.g `--users=BobrImperator,XAMPPRocky`

- `--date=YYYY-MM-DD` - **REQUIRED** It specifies the date of when a PR was *created*
e.g `--date=2021-12-01`.

- `-before` or `-after`- It specifies the direction of query by date.
e.g `-before --date=2021-12-01` = `< 2021-12-01`.

### File configuration
**NOT REQUIRED**

It's possible to configure file header and repository labels via `json` file.

- `labels` A list of objects with `name` and `repos` properties.
It will attempt to group items under their commont label.

e.g 
```json
// sample_config.json

{
  "labels": [
    {
      "name": "Rust",
      "repos": [
        "rust-lang/crates.io"
      ]
    },
    {
      "name": "Ember",
      "repos": [
        "ember-cli/ember-exam"
      ]
    }
  ]
}
```

- `header` A list of strings which then are joined together with a breakline.
```json
// sample_config.json

{
  "header": [
    "",
    "Header",
    ""
  ]
}
```
Should be included at the top of the output file as:
```

Header

```

- `users` A list of strings which are a valid github handles:
This will replace the `--users` cli option when both are present.

```json
// sample_config.json
{
  "users": ["BobrImperator"]
}

```

- `exclude` a list of repository names that should be excluded from the output.

```json
// sample_config.json
{
  "exclude": ["simplabs/simplabs.github.io"]
}

```

### Deploy
So far there isn't anything exciting for deploying it :)
If you wish to create a binary then run: `cargo build --target x86_64-apple-darwin --release --target-dir=bin`
then grab a binary of the specified target located in the target directory e.g `bin/x86_64-apple-darwin/release/this-week-in-open-source`

[sample_output.md](2021-12-01.md)
[sample_config.json](sample_config.json)
