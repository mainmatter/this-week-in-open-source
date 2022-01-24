## This-Week-in-Open-Source
It's a tool written on top of [XAMPPRocky/octocrab](https://github.com/XAMPPRocky/octocrab/tree/master/src).
It uses github's search API to find users' open source contributions.

### Usage

this-week-in-open-source is designe to run as a CLI tool either from an executable or directly with `cargo run`.

e.g `cargo run -- -after --date="2021-12-01" --users=BobrImperator` or
`GITHUB_PERSONAL_TOKEN=ghp_xxxxxxxxxx cargo run -- -after --date="2021-12-01" --users=BobrImperator`

## RateLimit
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

#### Available arguments

- `--users=user1,user2` - **REQUIRED** A list of comma separated github user names can have 1 or more entries, queries for PRs made by those users.
e.g `--users=BobrImperator,XAMPPRocky`

- `--date=YYYY-MM-DD` - **REQUIRED** It specifies the date of when a PR was *created*
e.g `--date=2021-12-01`.

- `-before` - **REQUIRED** It specifies the direction of query by date.
e.g `-before --date=2021-12-01` = `< 2021-12-01`.

- `-after` - **REQUIRED** It specifies the direction of query by date.
e.g `-after --date=2021-12-01` = `> 2021-12-01`.

[sample_output.md](2021-12-01.md)
