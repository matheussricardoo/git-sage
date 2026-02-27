<div align="center">

<img width="100%" src="https://capsule-render.vercel.app/api?type=waving&color=222222&height=200&section=header&text=git-sage&fontSize=55&fontColor=fff&animation=twinkling&fontAlignY=40&desc=AI-powered%20Git%20Commit%20Generator%20%7C%20Rust%20%7C%20Ollama&descAlignY=60&descSize=18">

<p align="center">
  <i>🦀 A CLI tool that uses a local LLM via Ollama to automatically generate Conventional Commit messages from your staged git diffs.</i>
</p>

---

### 🌟 Features

<div align="center">

| Feature | Description |
|:---:|:---|
| 🤖 | Local LLM — no API key, runs fully offline |
| ⚡ | GPU-accelerated inference via Ollama |
| 📝 | Follows the Conventional Commits specification |
| 🔀 | Split mode — one commit per file with `-s` |
| ✏️ | Interactive prompt — accept, edit or skip each suggestion |
| 🔒 | Lock file auto-included in the first accepted commit |
| 🎨 | Colored output for quick visual scanning |
| ⏳ | Spinner feedback while the model is generating |
| ⚠️ | Diff truncation warning with suggested `--ctx` value |

</div>

---

### 🛠️ Technologies

<div align="center">
  <a href="https://skillicons.dev">
    <img src="https://skillicons.dev/icons?i=rust,linux&theme=dark" />
  </a>
</div>

<div align="center">
  <sub>Tokio • Reqwest • Clap • Inquire • Serde • Indicatif • owo-colors • Ollama</sub>
</div>

---

### 🚀 Getting Started

#### 1. Install Ollama

```bash
curl -fsSL https://ollama.com/install.sh | sh
```

#### 2. Pull the model

```bash
ollama pull qwen2.5-coder:3b
```

#### 3. Clone & build

```bash
git clone https://github.com/matheussricardoo/git-sage.git
cd git-sage
cargo build --release
```

#### 4. (Optional) Install globally

```bash
cargo install --path .
```

---

### 💻 Usage

```bash
# Stage your files first
git add .

# Generate a single commit for all staged files
git-sage

# Generate one commit per file (split mode)
git-sage -s

# Prompt to push after all commits are done
git-sage -s --push

# Use a different model
git-sage -s --model qwen2.5-coder:7b

# Adjust model creativity (0.0 = deterministic, 1.0 = creative)
git-sage -s --temp 0.2

# Increase context window for large diffs
git-sage -s --ctx 4096

# Run fully on CPU (no GPU)
git-sage --gpu 0 --threads 8
```

#### Interactive prompt

For each suggestion you can choose:

```
--- src/main.rs
⠋ Generating commit message...
Suggestion: feat(main): add --push flag and colored output

> Commit this file?
  Yes    ← commit as-is
  No     ← skip this file
  Edit   ← open the message for editing before committing
```

#### Truncation warning

When a diff exceeds the context window, a warning is shown with a suggested fix:

```
⚠ Diff truncated (6000/12400 bytes). Use --ctx 4133 for full analysis.
```

---

### 📁 Structure

```
src/
├── main.rs           # CLI args, commit workflow, split logic, colored output
├── llm.rs            # Ollama API client, spinner, truncation logic
├── git.rs            # Git diff helpers, generic lock file detection
└── instructions.txt  # System prompt for the LLM
```

---

### 🤖 Model Configuration

All model parameters are controlled via CLI flags — no need to edit source code or recompile.

```bash
git-sage --model <MODEL> --threads <N> --gpu <N> --ctx <N> --temp <F>
```

To see all available models locally:

```bash
ollama list
```

To pull a different model:

```bash
ollama pull <model-name>
# Examples:
ollama pull qwen2.5-coder:7b
ollama pull deepseek-coder:6.7b
ollama pull llama3.2:3b
```

---

### ⚙️ CLI Flags

All parameters have sensible defaults tuned for a **GTX 1050 Ti (4GB VRAM) + i5-7400 + 8GB RAM** setup.

| Flag | Default | Description |
|:---|:---:|:---|
| `-s, --split` | `false` | Generate one commit per staged file |
| `-t, --temp` | `0.0` | Model temperature (`0.0` = deterministic, `1.0` = creative) |
| `--model` | `qwen2.5-coder:3b` | Ollama model to use |
| `--threads` | `4` | CPU threads (set to your core count) |
| `--gpu` | `99` | GPU layers to offload (`99` = all, `0` = CPU only) |
| `--ctx` | `2048` | Context window in tokens (increase for large diffs) |
| `--push` | `false` | Prompt to push commits to remote after finishing |

---

### 🔒 Lock File Support

git-sage automatically detects staged lock files and attaches them to the first accepted commit. No manual handling needed. Supported lock files:

| Language | Lock File |
|:---|:---|
| Rust | `Cargo.lock` |
| Node (npm) | `package-lock.json` |
| Node (yarn) | `yarn.lock` |
| Node (pnpm) | `pnpm-lock.yaml` |
| Python (poetry) | `poetry.lock` |
| Python (pipenv) | `Pipfile.lock` |
| Ruby | `Gemfile.lock` |
| Go | `go.sum` |
| PHP | `composer.lock` |
| Nix | `flake.lock` |

---

### 🧠 Model Selection Guide

Choose based on your available VRAM:

| VRAM | Recommended Model | Notes |
|:---:|:---|:---|
| 2 GB | `qwen2.5-coder:1.5b` | Fast, lower accuracy |
| 4 GB | `qwen2.5-coder:3b` ✅ | Best quality/speed ratio for 4 GB cards |
| 6 GB | `qwen2.5-coder:7b` | Noticeably better descriptions |
| 8 GB+ | `deepseek-coder:6.7b` | High quality output |
| CPU only | `qwen2.5-coder:1.5b` | Use `--gpu 0`, expect slower inference |

> The default `qwen2.5-coder:3b` in Q4 quantization uses ~1.9 GB of VRAM and fits entirely on a 4 GB card.

#### Checking your specs

**GPU VRAM (Linux):**

```bash
nvidia-smi --query-gpu=name,memory.total,memory.free --format=csv
```

**CPU core count:**

```bash
nproc
```

**RAM:**

```bash
free -h
```

**Check if Ollama is using your GPU:**

```bash
ollama ps
# The "Processor" column shows GPU % vs CPU %
```

---

### 👤 Author

<div align="center">
  <a href="https://github.com/matheussricardoo" target="_blank">
    <img src="https://skillicons.dev/icons?i=github" alt="GitHub"/>
  </a>
  <a href="https://www.linkedin.com/in/matheus-ricardo-426452266/" target="_blank">
    <img src="https://skillicons.dev/icons?i=linkedin" alt="LinkedIn"/>
  </a>
</div>

<img width="100%" src="https://capsule-render.vercel.app/api?type=waving&color=222222&height=120&section=footer"/>

</div>