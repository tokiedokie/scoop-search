![Build](https://github.com/tokiedokie/scoop-search/workflows/Build/badge.svg)
![GitHub release (latest by date including pre-releases)](https://img.shields.io/github/v/release/tokiedokie/scoop-search?include_prereleases)

# scoop-search

scoop-search is a tool for a windows package maneger [scoop](https://scoop.sh/)

`scoop-search` instead of `scoop search`

## Installation

```sh
scoop install https://raw.githubusercontent.com/tokiedokie/scoop-search/master/scoop-search.json
```

## Usege

```sh
scoop-search <query>
```
this command search apps very quick.
this search app manufect file names and remote buckets apps, if there is no app which contains wuery, then search binary file.


```sh
scoop-search --bin <query>
```
same as `scoop search`
