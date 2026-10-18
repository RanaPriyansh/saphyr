# Saphyr large-integer parsing reproduction

This bundle contains the same public-loader example used for both measurements, a deterministic generator for 17 control corpora, a manifest for all 18 inputs, and two separate sets of 216 raw timing observations.

## Revisions and build

- Upstream baseline commit: `b8205d5f60ea44f757f7d075fe4f3704bbe5dccc`
- Published implementation: `690798c149d28fcd9b222cbe04a646a3990c32c4`
- Optimized candidate: `1800c891e0188a356395f663d58f2384c42d300d`
- Shared `Cargo.lock` SHA-256: `d587efb9546609874e93ad42234c2985b6dc47b6ca25f75c66f046f3d44e2f06`
- Shared loader driver SHA-256: `d8c3f7a3487f228ece7bdceef93e9b4acf65a06b00bae6b9c92408ca3b98c30a`

Make each revision available in a separate clean checkout. Run the commands from each checkout root. Use Rust 1.85.0 and a separate target directory for every revision.

For the upstream baseline:

```sh
mkdir -p saphyr/examples
cp /path/to/public-reproduction/loader_time.rs saphyr/examples/loader_time.rs
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=target/loader-master cargo rustc --locked --release --example loader_time -- "--check-cfg=cfg(saphyr_string_integer)"
```

For the published implementation:

```sh
mkdir -p saphyr/examples
cp /path/to/public-reproduction/loader_time.rs saphyr/examples/loader_time.rs
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=target/loader-published cargo rustc --locked --release --example loader_time -- --cfg=saphyr_string_integer "--check-cfg=cfg(saphyr_string_integer)"
```

For the optimized candidate:

```sh
mkdir -p saphyr/examples
cp /path/to/public-reproduction/loader_time.rs saphyr/examples/loader_time.rs
CARGO_BUILD_JOBS=2 CARGO_TARGET_DIR=target/loader-optimized cargo rustc --locked --release --example loader_time -- --cfg=saphyr_string_integer "--check-cfg=cfg(saphyr_string_integer)"
```

The candidate-only cfg enables the `StringInteger` witness arm. The example source is byte-identical for all three builds. The candidate was tested before commit. The committed source bytes match the validated candidate.

## Inputs

Generate the 17 control files with Python 3:

```sh
python3 /path/to/public-reproduction/generate_controls.py /path/to/public-reproduction/corpora/controls
```

Generate the large `big.yaml` from an upstream checkout. Run this from its repository root:

```sh
cargo build --locked --release --manifest-path bench/tools/gen_large_yaml/Cargo.toml --bin gen_large_yaml
mkdir -p bench_yaml
./target/release/gen_large_yaml
mkdir -p /path/to/public-reproduction/corpora/generated
cp bench_yaml/big.yaml /path/to/public-reproduction/corpora/generated/big.yaml
```

The project generator uses seed 42 and writes 100,000 top-level records to `big.yaml`. It also writes three other files. The manifest lists only `big.yaml`; its bytes and SHA-256 are `232063480` and `864854cd3bd9322374265983018e29e9fb41aa23f5d764e8c6a44ad04973f38c`.

Each length-control file contains one YAML document with a sequence of 16 values. Each timed process parses the complete file. The ordinary scalar and overflow mixture files also parse as complete documents.

## Timing

Each fresh process loads its input before timing, calls `Yaml::load_from_str` twice for warmups, then calls it once for the measured parse. The output records nanoseconds. The returned syntax tree remains live when the timer stops. The process then drops it.

Use six fresh processes for each revision and corpus. Run three four-process ABBA blocks. Start with published, optimized, optimized, published for even-positioned corpora. Reverse that order for the next corpus. The exact order and each raw observation are in `observations.csv`.

Example for one measured process:

```sh
target/loader-published/release/examples/loader_time timing published_current ordinary_scalar /path/to/public-reproduction/corpora/controls/ordinary-scalar.yaml
```

The command emits two warmup records and one measured `parse_ns` record. Repeat it in the listed ABBA order, changing the binary and input for each row.

Run witness mode outside the timing interval when you need to check output types. The upstream baseline at `b8205d5f60ea44f757f7d075fe4f3704bbe5dccc` and the published implementation produce different typed values for overflow inputs. This bundle therefore compares the published implementation with the optimized candidate. It makes no upstream-baseline-to-candidate speedup claim.

## Original upstream-baseline campaign

A separate retained campaign compares upstream baseline `b8205d5f60ea44f757f7d075fe4f3704bbe5dccc` with published implementation `690798c149d28fcd9b222cbe04a646a3990c32c4`. It used the same loader driver SHA-256 listed above. Its 216 raw samples and ABBA order are in `master-published-observations.csv`; every corpus has six fresh-process samples per revision. Use the same build and timing commands above, select the binary by the `revision` field, and use the `corpus` entry and `input_sha256` to select the exact input.

For `ordinary_scalar` (3.67 MB), the medians were 115.75 ms for upstream baseline and 110.00 ms for published implementation. For `large_generated` (232.06 MB), the medians were 1470.65 ms and 1467.40 ms. The CSV preserves all sample values and order, including the full ranges. The overflow witness outputs differ between these revisions. The manifest records 34 compact control-witness hashes and both large-generated canonical witness output hashes. It does not include the large witness output files. Do not compare this campaign's values with the separate published-to-optimized campaign as a timing ratio or claim.

## Measurements

The host was an Apple M3 system with macOS 27.0, 16 GiB of memory, and 8 logical CPUs. Host identity was recorded separately from the timed run. Frequency, thermal state, and background load were not recorded.

Selected six-process medians for the published and optimized implementations:

- Ordinary scalar file: 94.77 ms and 95.72 ms.
- Generated 100,000-record file: 1469.91 ms and 1472.10 ms.
- Overflow mixture: 14.56 ms and 9.84 ms.
- 4096-digit hexadecimal file: 265.06 ms and 4.01 ms.
- 4096-digit octal file: 201.32 ms and 2.58 ms.

These are local parse-time observations for the listed files. They do not establish a general speedup. The CSV retains all six observations and their ABBA order for both revisions in every corpus.
