## This-Week-in-Open-Source
It's a tool written on top of [XAMPPRocky/octocrab](https://github.com/XAMPPRocky/octocrab/tree/master/src).
It uses github's search API to find users' open source contributions.

### Usage

this-week-in-open-source is designe to run as a CLI tool either from an executable or directly with `cargo run`.

e.g `cargo run -after --date="2021-12-01" --users=BobrImperator`

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
