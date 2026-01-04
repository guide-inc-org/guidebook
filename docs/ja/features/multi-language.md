# 多言語対応

guidebook は複数言語でのブック作成をサポートしています。

## セットアップ

ブックのルートに `LANGS.md` ファイルを作成：

```markdown
# Languages

* [English](en/)
* [日本語](ja/)
* [Tiếng Việt](vi/)
```

コンテンツを整理：

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

## 言語セレクター

左上に言語セレクターが表示され、読者は言語を切り替えることができます。

## ヒント

- 各言語フォルダには独自の `SUMMARY.md` があります
- 言語ごとに異なるコンテンツを持つことができます
- 画像は相対パスを使用して言語間で共有できます
