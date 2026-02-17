# 用語集

ブックのルートに `GLOSSARY.md` ファイルを作成して用語を定義します。用語はブック全体で自動検出され、ツールチップとして表示されます。

## フォーマット

```markdown
## API
Application Programming Interface

## HTML
HyperText Markup Language

## SSR
Server-Side Rendering
```

各用語は `## 用語名` で始まり、次の行に定義を記述します。

## 仕組み

ビルド時に全ページで用語をスキャンし、ツールチップ付きの span に変換します：

```html
<span class="glossary-term" data-definition="Application Programming Interface">API</span>
```

用語にホバーすると定義が表示されます。

## 除外

以下の場所では用語の置換は**行われません**：

- コードブロック（`` ` `` と ` ``` `）
- リンク（`<a>` タグ）
- 見出し（`<h1>` 〜 `<h6>`）
- script タグ
- `class="no-glossary"` を持つ要素

特定の要素で置換を防ぐには、`no-glossary` クラスを追加：

```html
<span class="no-glossary">API</span>
```
