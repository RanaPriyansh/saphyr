import hashlib
import json
import sys
from pathlib import Path


ROWS_SCALAR = 65_536
ROWS_OVERFLOW = 8_192
LENGTHS = (64, 256, 1024, 4096)


def write_case(root, filename, row, count):
    data = (f"- {row}\n" * count).encode("ascii")
    path = root / filename
    path.write_bytes(data)
    return {"file": filename, "bytes": len(data), "sha256": hashlib.sha256(data).hexdigest()}


def generate(root):
    root = Path(root)
    root.mkdir(parents=True, exist_ok=True)
    corpora = {}
    corpora["ordinary_scalar"] = write_case(
        root,
        "ordinary-scalar.yaml",
        "[null, true, false, 0, 42, -7, 0x2A, 0o52, 3.5, text]",
        ROWS_SCALAR,
    )
    corpora["overflow_mixture"] = write_case(
        root,
        "overflow-mixture.yaml",
        "[9223372036854775808, 9223372036854775809, 0x8000000000000000, 0o1000000000000000000000]",
        ROWS_OVERFLOW,
    )
    for radix, digit, prefix in ((10, "9", ""), (16, "f", "0x"), (8, "7", "0o")):
        for length in LENGTHS:
            name = f"length_{radix}_{length}"
            filename = f"length-{radix}-{length}.yaml"
            corpora[name] = write_case(root, filename, prefix + digit * length, 16)
    for radix, digit, prefix in ((10, "9", ""), (16, "f", "0x"), (8, "7", "0o")):
        name = f"late_invalid_{radix}"
        filename = f"late-invalid-{radix}-4096.yaml"
        corpora[name] = write_case(root, filename, prefix + digit * 4096 + "g", 16)
    if len(corpora) != 17:
        raise RuntimeError(f"expected 17 control corpora, got {len(corpora)}")
    manifest = {
        "format": 1,
        "rows_ordinary_scalar": ROWS_SCALAR,
        "rows_overflow_mixture": ROWS_OVERFLOW,
        "length_values_per_file": 16,
        "python_integer_conversion_limit": 8192,
        "corpora": corpora,
    }
    (root / "controls-manifest.json").write_text(json.dumps(manifest, indent=2, sort_keys=True) + "\n")
    return manifest


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("usage: generate_controls.py OUTPUT_DIR")
    print(json.dumps(generate(sys.argv[1]), sort_keys=True))
