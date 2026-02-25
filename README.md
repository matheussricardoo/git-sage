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
| 🤖 | Local LLM (no API key needed) |
| ⚡ | GPU-accelerated inference |
| 📝 | Follows the Conventional Commits specification |
| 🔀 | Split mode — one commit per file with `-s` |
| ✏️ | Interactive prompt — accept, edit or skip each suggestion |
| 🔒 | Cargo.lock auto-included in the first accepted commit |

</div>

---

### 🛠️ Technologies

<div align="center">
  <a href="https://skillicons.dev">
    <img src="https://skillicons.dev/icons?i=rust,linux&theme=dark" />
  </a>
</div>

<div align="center">
  <sub>Tokio • Reqwest • Clap • Inquire • Serde • Ollama</sub>
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
git-sage --split
git-sage -s

# Adjust model creativity (0.0 = deterministic, 1.0 = creative)
git-sage -s --temp 0.2
```

#### Interactive prompt

For each suggestion you can choose:

```
Suggestion: feat(main): add interactive commit workflow

> Commit this file?
  Yes    ← commit as-is
  No     ← skip this file
  Edit   ← open the message for editing before committing
```

---

### 📁 Structure

```
src/
├── main.rs           # CLI args, commit workflow, split logic
├── llm.rs            # Ollama API client & request builder
├── git.rs            # Git diff helpers
└── instructions.txt  # System prompt for the LLM
```

---

### 🤖 Model Configuration

The model and all inference parameters are defined in `src/llm.rs`.

#### Changing the model

Open `src/llm.rs` and edit the `model` field:

```rust
let request = Request {
    model: "qwen2.5-coder:3b".to_string(), // <-- change this
    ...
};
```

Then rebuild:

```bash
cargo build --release
```

To see all models available locally:

```bash
ollama list
```

To pull a different model:

```bash
ollama pull <model-name>
# Examples:
ollama pull qwen2.5-coder:7b
ollama pull deepseek-coder:6.7b
ollama pull phi3:mini
```

---

### ⚙️ Inference Parameters

All parameters live in the `Options` struct in `src/llm.rs`. The current values were chosen for a **GTX 1050 Ti (4GB VRAM) + i5-7400 + 8GB RAM** setup. Adjust them to match your hardware.

| Parameter | Current Value | What it does | How to tune |
|:---|:---:|:---|:---|
| `num_predict` | `80` | Max tokens the model can generate. Commit messages are short, so 80 is more than enough. | Increase if messages get cut off. |
| `num_ctx` | `2048` | Context window size (tokens). Limits how much of the diff the model reads. | Increase for large diffs if you have VRAM. Try `4096`. |
| `num_gpu` | `99` | Model layers offloaded to the GPU. `99` means all layers. | Set to `0` to run fully on CPU. |
| `num_thread` | `4` | CPU threads used for non-GPU parts. | Set to your CPU core count. |
| `temperature` | `0.0` (default) | Creativity of the output. `0.0` = fully deterministic. | Pass `--temp 0.2` via CLI for slight variation. |

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

### 🧠 Model Selection Guide

Choose based on your available VRAM:

| VRAM | Recommended Model | Notes |
|:---:|:---|:---|
| 2 GB | `qwen2.5-coder:1.5b` | Fast, lower accuracy |
| 4 GB | `qwen2.5-coder:3b` ✅ | Best quality/speed ratio for 4 GB cards |
| 6 GB | `qwen2.5-coder:7b` | Noticeably better descriptions |
| 8 GB+ | `deepseek-coder:6.7b` | High quality output |
| CPU only | `qwen2.5-coder:1.5b` | Set `num_gpu: 0`, expect slower inference |

> The default `qwen2.5-coder:3b` in Q4 quantization uses ~1.9 GB of VRAM and fits entirely on a 4 GB card.

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