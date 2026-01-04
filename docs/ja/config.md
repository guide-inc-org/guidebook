# 設定

設定はオプションです。guidebook は設定なしでも動作します。

## book.json

ブックのルートディレクトリに `book.json` ファイルを作成：

```json
{
    "title": "私のブック",
    "description": "ブックの説明",
    "author": "著者名",
    "plugins": [
        "collapsible-chapters",
        "back-to-top-button",
        "mermaid-md-adoc"
    ],
    "styles": {
        "website": "styles/website.css"
    }
}
```

## オプション

| オプション | 説明 | デフォルト |
|-----------|------|----------|
| `title` | ブックタイトル | `"My Book"` |
| `description` | ブックの説明 | `""` |
| `author` | 著者名 | `""` |
| `plugins` | 有効なプラグイン | 下記参照 |
| `styles.website` | カスタム CSS ファイル | `null` |

## デフォルトプラグイン

以下のプラグインはデフォルトで有効（設定不要）：

- `collapsible-chapters` - 折りたたみサイドバー
- `back-to-top-button` - トップに戻るボタン
- `mermaid-md-adoc` - Mermaid 図のサポート

デフォルトプラグインを無効にするには、`-` をプレフィックスに：

```json
{
    "plugins": ["-mermaid-md-adoc"]
}
```

## カスタムスタイル

CSS ファイルを作成し、`book.json` で参照：

```css
/* styles/website.css */
.book {
    font-family: "Noto Sans JP", sans-serif;
}

.markdown-section h2 {
    border-left: 4px solid #007acc;
    padding-left: 10px;
}
```
