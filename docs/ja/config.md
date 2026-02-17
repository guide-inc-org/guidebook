# 設定

設定はオプションです。guidebook は設定なしでも動作します。

## book.json

ブックのルートディレクトリに `book.json` ファイルを作成：

```json
{
    "title": "私のブック",
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
| `plugins` | 有効なプラグイン | 下記参照 |
| `styles.website` | カスタム CSS ファイル | `null` |
| `variables` | ユーザー定義変数 (`{{ book.xxx }}`) | `{}` |
| `hardbreaks` | 単一改行を `<br>` として扱う | `false` |
| `math` | KaTeX 数式レンダリングを有効化 | `false` |
| `externalize_svg` | インライン SVG を外部ファイルに分離 | `false` |
| `inline_svg` | SVG ファイルを HTML にインライン化 | `false` |
| `fetchRemoteImages` | ビルド時にリモート画像をダウンロード | `false` |
| `openapi` | OpenAPI/Swagger UI 仕様ファイル | `null` |

## デフォルトプラグイン

以下のプラグインはデフォルトで有効（設定不要）：

- `collapsible-chapters` - 折りたたみサイドバー
- `back-to-top-button` - トップに戻るボタン
- `mermaid-md-adoc` - Mermaid 図のサポート
- `fontsettings` - フォントサイズ・テーマ切替（白/セピア/ナイト）

デフォルトプラグインを無効にするには、`-` をプレフィックスに：

```json
{
    "plugins": ["-mermaid-md-adoc"]
}
```

## 変数

カスタム変数を定義し、Nunjucks/Jinja2 構文で Markdown 内で使用：

```json
{
    "variables": {
        "version": "1.0.0",
        "appName": "My App"
    }
}
```

Markdown 内での使用：

```markdown
現在のバージョン: {{ book.version }}

{{ book.appName }} へようこそ！
```

コードブロック（`` ` `` と ` ``` `）内の変数は展開されません。

## 数式（KaTeX）

数式レンダリングを有効化：

```json
{
    "math": true
}
```

インライン数式には `$...$`、ディスプレイ数式には `$$...$$` を使用。

## リモート画像ダウンロード

ビルド時にリモート HTTPS 画像をダウンロードしてオフラインアクセス可能に：

```json
{
    "fetchRemoteImages": true
}
```

画像は `_remote_images/` に CRC32 ベースのファイル名でキャッシュされます。1画像あたり最大 50 MB。

## OpenAPI / Swagger UI

OpenAPI 仕様から Swagger UI を生成：

```json
{
    "openapi": "swagger.json"
}
```

複数 API の場合：

```json
{
    "openapi": {
        "api-docs": "swagger/v1.json",
        "admin-api": "swagger/admin.json"
    }
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
