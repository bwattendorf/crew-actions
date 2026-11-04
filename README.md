# crew-actions

Shared, reusable GitHub Actions workflows for every repo under Stitchwright,
SiteSentrix and bwattendorf. Each repo's CI file becomes ~20 lines that call
these workflows with a few inputs.

**The rule: fix CI here once, every repo gets it.** Callers track `@main` on
purpose, so a fix merged here reaches every repo on its next run. That is also
why repos never add their own CI jobs inside a feature PR: CI changes come here
as their own task.

This repo is public because GitHub only lets repos of the *same owner* call a
private reusable workflow, and ours live under three owners. It holds no
secrets; callers pass their own when a workflow needs one.

## Workflows

| File | What it does | Secrets |
|---|---|---|
| `rust-gate.yml` | Checkout, Rust toolchain from the caller's `rust-toolchain(.toml)`, cargo cache (restore always, save only on the default branch), runs the gate command (default `cargo xtask ci`). Linux only. | none |
| `node-gate.yml` | Checkout, Node from `node-version` or `.nvmrc`, npm cache (restore always, save only on the default branch), `npm ci`, runs the gate command (default `npm run ci`). Linux only. | none |
| `public-guard.yml` | For public repos: gitleaks secret scan (pinned release, checksum verified, redacted output) plus your list of forbidden words and forbidden paths. | none |
| `claude-queue-pickup.yml` | Optional. When an issue gets `claude-queue`, posts a wake comment and the `claude-wake` label so the repo's own claude-fix workflow picks it up. Never runs an agent itself. | optional `wake-token` |

All third-party actions inside are pinned to full commit SHAs (tag in a
comment), and downloaded tools (gitleaks, actionlint) are checked against a
SHA-256 before they run. Every job declares minimal `permissions:`.

### rust-gate inputs
| Input | Default | Notes |
|---|---|---|
| `gate-command` | `cargo xtask ci` | Whatever decides green/red; agents run the same command locally |
| `working-directory` | `.` | Folder with the Cargo workspace |
| `timeout-minutes` | `15` | Set ~8 for private-repo PR gates |
| `fallback-toolchain` | `stable` | Only used when there is no toolchain file |
| `fetch-depth` | `1` | `0` for full history |

### node-gate inputs
| Input | Default | Notes |
|---|---|---|
| `gate-command` | `npm run ci` | |
| `working-directory` | `.` | Folder with `package.json` / `package-lock.json` |
| `node-version` | empty | Empty = read `.nvmrc` in `working-directory` |
| `install-command` | `npm ci` | |
| `timeout-minutes` | `15` | |
| `fetch-depth` | `1` | |

### public-guard inputs
| Input | Default | Notes |
|---|---|---|
| `forbidden-words` | empty | One per line, case-insensitive. The log shows file:line only, never the text |
| `forbidden-paths` | empty | One git glob per line, e.g. `**/*.pem` |
| `exclude-paths` | `.github/workflows/**` | Skipped by the word check (the word list lives there) |
| `scan-history` | `true` | `false` = scan current files only (faster) |
| `timeout-minutes` | `10` | |

A `.gitleaks.toml` in the caller's root is picked up automatically for
allowlists.

### claude-queue-pickup inputs
| Input | Default |
|---|---|
| `comment` | `@claude continue: pick up this claude-queue task and run the full loop` |
| `trigger-label` | `claude-queue` |
| `wake-label` | `claude-wake` (empty = no label) |

Heads-up: comments and labels made with the default `GITHUB_TOKEN` do **not**
start other workflows (GitHub's loop protection). If your claude-fix workflow
reacts to the comment, pass a `wake-token` (fine-grained PAT or GitHub App
token, Issues: write, that repo only). Simpler: let claude-fix listen to
`issues: labeled` directly and skip this workflow.

## Caller snippets

### Rust repo: `.github/workflows/pr.yml`
```yaml
name: pr
on:
  pull_request:
concurrency:
  group: pr-${{ github.event.pull_request.number }}
  cancel-in-progress: true          # a newer push cancels the older run
permissions:
  contents: read
jobs:
  gate:
    uses: bwattendorf/crew-actions/.github/workflows/rust-gate.yml@main
    with:
      timeout-minutes: 8
```

### Rust repo: `.github/workflows/main.yml` (runs to the end, writes the cache)
```yaml
name: main
on:
  push:
    branches: [main]
concurrency:
  group: main-${{ github.sha }}
  cancel-in-progress: false
permissions:
  contents: read
jobs:
  gate:
    uses: bwattendorf/crew-actions/.github/workflows/rust-gate.yml@main
    with:
      timeout-minutes: 30
```

### Node repo
```yaml
name: pr
on:
  pull_request:
concurrency:
  group: pr-${{ github.event.pull_request.number }}
  cancel-in-progress: true
permissions:
  contents: read
jobs:
  gate:
    uses: bwattendorf/crew-actions/.github/workflows/node-gate.yml@main
    with:
      timeout-minutes: 8
      # node-version: "22"           # or leave empty to read .nvmrc
      # gate-command: npm run ci
```

### Public repo guard
```yaml
name: public-guard
on:
  pull_request:
  push:
    branches: [main]
permissions:
  contents: read
jobs:
  guard:
    uses: bwattendorf/crew-actions/.github/workflows/public-guard.yml@main
    with:
      forbidden-words: |
        some-private-codename
      forbidden-paths: |
        **/*.pem
        **/.env
```

### claude-queue pickup (optional)
```yaml
name: claude-queue-pickup
on:
  issues:
    types: [labeled]
permissions:
  issues: write
jobs:
  wake:
    uses: bwattendorf/crew-actions/.github/workflows/claude-queue-pickup.yml@main
    secrets:
      wake-token: ${{ secrets.CLAUDE_WAKE_TOKEN }}   # optional, see heads-up above
```

The caller's top-level `permissions:` must be at least what the called job
asks for (`contents: read` for the gates and guard, `issues: write` for pickup).

## GitHub Free notes

- **Private repos share ~2,000 Linux minutes a month** on the Free plan
  (Windows bills 2x, macOS 10x). A 30-minute PR gate would use that up in ~65
  runs. Keep private PR gates **under ~8 minutes, Linux only**.
- **Cancel superseded runs**: set `concurrency` with `cancel-in-progress: true`
  in the caller's PR workflow (snippets above). Pushes to main run to the end.
- **Cache**: saved only from the default branch; PR runs read it but never write
  it, so the 10 GB per-repo cache stays useful.
- Slow suites, other OS builds and dependency audits belong in a nightly
  workflow, not on PRs.
- Public repos get unlimited standard-runner minutes, branch protection and
  required checks for free: require the gate check there.

## Changing this repo

- Every change goes through a PR. `self-test.yml` runs actionlint, checks every
  third-party action is pinned to a SHA, and runs each reusable workflow against
  the samples in `examples/` from the PR's own commit.
- Because callers track `@main`, a merge here is live everywhere. After a
  change merges, watch one caller (stitch-e2e first) before relying on it
  across all repos.
- Bumping a pinned action: update the SHA and the tag comment together.

## License

MIT, see [LICENSE](LICENSE).
