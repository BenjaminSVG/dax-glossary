# Contributing

Thanks for helping to open up DAX knowledge. Good first contributions: a missing template, a corrected syntax,
a clearer description, a translation, a binding for another language.

## The one rule: edit the sources, not `data/*.json`

`data/*.json` and `README*.md` are **generated**. CI fails if the committed files differ from what the generators produce.

| you want to... | edit | then run |
| --- | --- | --- |
| add / fix a function, description or category | `gen/functions_*.txt` (`NAME \| category \| syntax \| returns \| description`) | `python gen/fetch_all.py` (if it is new), `python gen/build_data.py`, `python gen/build_readme.py` |
| add / fix a measure template | `gen/measures.py` (an `M(...)` call) | `python gen/build_data.py`, `python gen/build_readme.py` |
| change wording of the README | `gen/readme/README.*.tpl.md` | `python gen/build_readme.py` |
| change code | `src/`, `tests/` | `cargo test` |

## Checklist before opening a PR

```bash
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
python gen/build_data.py && python gen/build_readme.py && git diff --stat
```

No C toolchain? Run the cargo commands in Docker (see the README, *Development*).

## Data rules

- **Descriptions are original text**, one short sentence. Do not paste text from the Microsoft documentation.
- **Syntax comes from the function's Microsoft Learn page** (the generators fetch it). If you add a function that is not on the category pages, make sure its own Learn page exists.
- **Templates need a worked example** for every parameter, declare every placeholder, and say what they assume about the model (`assumptions`). The integrity tests enforce this.
- Parameter types are `table`, `column`, `measure`, `expression`, `name`, `number`, `text`; pick the strictest that fits.

## Style

Rust: `cargo fmt` (see `rustfmt.toml`), no warnings from clippy, no new dependencies without a strong reason (the crate deliberately depends only on `serde` and `serde_json`).

By contributing you agree that your contribution is licensed under the MIT license of this repository.
