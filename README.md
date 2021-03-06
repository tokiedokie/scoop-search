# scoop-search

![Build](https://github.com/tokiedokie/scoop-search/workflows/Build/badge.svg)
![GitHub release (latest by date including pre-releases)](https://img.shields.io/github/v/release/tokiedokie/scoop-search?include_prereleases)
![GitHub All Releases](https://img.shields.io/github/downloads/tokiedokie/scoop-search/total)

scoop-search is a tool for a windows package manager [scoop](https://scoop.sh/)

`scoop-search` instead of `scoop search`

![sample](https://raw.githubusercontent.com/tokiedokie/scoop-search/master/images/sample.gif)

## Installation

```sh
scoop install https://raw.githubusercontent.com/tokiedokie/scoop-search/master/scoop-search.json
```

## Usage

```sh
scoop-search <query>
```
This command searches apps very quickly.
This searches local app manifest files and remote bucket apps. If there is no app manifest filename which contains `query`, then it searches for binary files.


```sh
scoop-search --bin <query>
```
Almost same as `scoop search`, but faster.
