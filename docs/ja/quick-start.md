# クイックスタート

## 新しいブックを作成

以下の構造でフォルダを作成します：

```
my-book/
├── README.md       # イントロダクション（index.html になる）
├── SUMMARY.md      # 目次
└── chapter1.md     # コンテンツ
```

### README.md

```markdown
# 私のブック

ようこそ！
```

### SUMMARY.md

```markdown
# 目次

* [はじめに](README.md)
* [第1章](chapter1.md)
```

### chapter1.md

```markdown
# 第1章

これは最初の章です。
```

## プレビュー

```bash
cd my-book
guidebook serve
```

ブラウザで http://localhost:4000 を開きます。

ファイルの変更を監視し、自動的にリロードします。

## 本番用ビルド

```bash
guidebook build -o _book
```

`_book` フォルダに静的 HTML ファイルが生成されます。

## デプロイ

`_book` フォルダを任意の静的ホスティングにアップロード：

- GitHub Pages
- Netlify
- Vercel
- Cloudflare Pages
- 任意の Web サーバー
