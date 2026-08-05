from pathlib import Path


def replace_exact(path: str, old: str, new: str) -> None:
    target = Path(path)
    content = target.read_text(encoding="utf-8")
    count = content.count(old)
    if count != 1:
        raise RuntimeError(f"expected exactly one match in {path}, found {count}")
    target.write_text(content.replace(old, new, 1), encoding="utf-8")


replace_exact(
    "src-tauri/src/app_core/tests/remote_privacy_api_tests.rs",
    '        "http://api.example.com/v1",\n',
    '        "http://127.0.0.1:11434/v1",\n',
)
replace_exact(
    ".github/workflows/ralph-loop-apply.yml",
    "            libtesseract-dev \\\n            tesseract-ocr\n",
    "            libtesseract-dev \\\n            tesseract-ocr \\\n            xvfb\n",
)
replace_exact(
    ".github/workflows/ralph-loop-apply.yml",
    "          cargo test --manifest-path src-tauri/Cargo.toml --all-features 2>&1 \\\n",
    "          xvfb-run -a cargo test --manifest-path src-tauri/Cargo.toml --all-features 2>&1 \\\n",
)
