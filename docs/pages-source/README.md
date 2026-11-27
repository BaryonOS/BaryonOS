# Editing the visitor docs

The topic text is in `topics/`. The shared layout and topic directory metadata are in `layout.html` and `build_docs.py`. After an edit, generate and check the committed site output from the repository root:

```sh
python3 -B docs/pages-source/build_docs.py
python3 -B docs/pages-source/build_docs.py --check
```

The publishing branch receives the files under `docs/pages/`. Keep the chapter summaries and source links current. The project contracts, roadmap and verification receipts remain authoritative for product claims. The website loads the generated roadmap graphic from `main`.
