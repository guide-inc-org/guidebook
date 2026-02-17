# テンプレート

guidebook は Markdown ファイル内で Nunjucks/Jinja2 テンプレート構文をサポートします（Tera テンプレートエンジン使用）。

## 変数

`book.json` で変数を定義し、Markdown 内で使用：

```json
{
    "variables": {
        "version": "2.0.0",
        "appName": "My App"
    }
}
```

```markdown
現在のバージョン: {{ book.version }}

{{ book.appName }} へようこそ！
```

## 条件分岐

```markdown
{% if book.version %}
バージョン: {{ book.version }}
{% else %}
バージョン未設定
{% endif %}
```

`{% elif %}` もサポートされています。

## ループ

```markdown
{% for item in book.features %}
- {{ item }}
{% endfor %}
```

ループ内では `loop.index`（1ベース）が使用可能です。

## フィルター

| フィルター | 説明 |
|-----------|------|
| `upper` | 大文字に変換 |
| `lower` | 小文字に変換 |
| `capitalize` | 先頭文字を大文字化 |
| `length` | 長さを取得 |
| `default(value="fallback")` | フォールバック値を提供 |

例：

```markdown
{{ book.appName | upper }}
{{ book.subtitle | default(value="サブタイトルなし") }}
```

## コードブロックの保護

コードブロック（`` ` `` と ` ``` `）内のテンプレート構文は**処理されません**。テンプレート構文をドキュメント化する際に、評価されずにそのまま表示できます。
