# Multi-language Support

guidebook supports building books in multiple languages.

## Setup

Create a `LANGS.md` file in your book's root:

```markdown
# Languages

* [English](en/)
* [日本語](ja/)
* [Tiếng Việt](vi/)
```

Then organize your content:

```
my-book/
├── LANGS.md
├── book.json
├── en/
│   ├── README.md
│   ├── SUMMARY.md
│   └── ...
├── ja/
│   ├── README.md
│   ├── SUMMARY.md
│   └── ...
└── vi/
    ├── README.md
    ├── SUMMARY.md
    └── ...
```

## Language Selector

A language selector appears in the top-left corner, allowing readers to switch languages.

## Tips

- Each language folder has its own `SUMMARY.md`
- You can have different content per language
- Images can be shared across languages using relative paths
