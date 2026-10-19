# git-sage

Rust command-line tool that generates commit-message suggestions from staged Git diffs using a local Ollama model. The user can accept, edit or skip suggestions. This is a development utility, not a hosted AI service.

## Implemented

- Clap command-line arguments and interactive review.
- Local HTTP streaming through Reqwest/Tokio, with NDJSON parsing.
- Single-commit mode or one commit per staged file with `--split`.
- Detection of common dependency lock files, attached to the first accepted split commit.
- UTF-8-safe diff truncation and explicit failures for incomplete/empty model output.
- Optional interactive push confirmation when `--push` is supplied.

## Run locally

Requires Rust, Git, Ollama and a downloaded model. Initial package/model downloads need network access; generation uses localhost:11434. No paid API key is required. Hardware requirements depend on the selected model and are not benchmarked by this repository.

```bash
git clone https://github.com/matheussricardoo/git-sage.git
cd git-sage
cargo build --release --locked
ollama pull qwen2.5-coder:3b
```

Start Ollama separately. From a repository containing staged changes, run the built binary or install it explicitly with `cargo install --path . --locked`.

```bash
git add path/to/file
git-sage                 # review a single suggested commit
git-sage --split         # review each file separately
git-sage --gpu 0         # CPU inference
git-sage --ctx 4096      # change the context estimate
git-sage --help
```

| Option | Default | Purpose |
|---|---|---|
| --model | qwen2.5-coder:3b | Local Ollama model |
| --threads | 4 | Inference thread setting |
| --gpu | 99 | GPU layer setting; 0 requests CPU |
| --ctx | 2048 | Context setting and diff-size estimate |
| --temp | 0.0 | Model temperature |
| --split | false | One commit per staged non-lock file |
| --push | false | Offer interactive Git push after commits |

## Validation

```bash
cargo test --locked
```

Four unit tests cover UTF-8 truncation and stream parsing. Two integration tests use temporary Git repositories to verify that split mode rejects unstaged changes and staged renames without calling a model. No push or model inference occurs in these tests.

## Limitations and decisions

Git uses NUL-delimited filenames. Split mode refuses files whose working-tree content differs from staged content because a pathspec commit can otherwise include changes outside the analyzed staged diff. Staged renames are refused in split mode; use single-commit mode to keep a rename atomic. Avoid editing files concurrently with a commit.

The byte/context ratio is an estimate and does not account precisely for prompt overhead or model tokenization. Large diffs can still exceed a model's context. Generated messages require review; output is not guaranteed to describe every change accurately.

Only the local Ollama endpoint is used for generation. Git push contacts the configured remote only after the optional interactive confirmation. Local inference, GPU behavior and a full interactive session were not exercised during the portfolio review.

## Author and license

[Matheus Ricardo](https://github.com/matheussricardoo) · [LinkedIn](https://www.linkedin.com/in/matheussricardoo/). See [LICENSE](LICENSE).
