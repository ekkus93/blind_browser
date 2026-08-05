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
    "",
)
