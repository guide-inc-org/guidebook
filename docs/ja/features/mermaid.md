# Mermaid 図

guidebook は [Mermaid](https://mermaid.js.org/) 図を組み込みでサポートしています。

## 使い方

`mermaid` コードブロックを使用：

~~~markdown
```mermaid
graph TD
    A[開始] --> B{判断}
    B -->|はい| C[何かする]
    B -->|いいえ| D[別のことをする]
    C --> E[終了]
    D --> E
```
~~~

## 例

### フローチャート

```mermaid
graph TD
    A[開始] --> B{判断}
    B -->|はい| C[何かする]
    B -->|いいえ| D[別のことをする]
```

### シーケンス図

```mermaid
sequenceDiagram
    太郎->>花子: こんにちは！
    花子-->>太郎: やあ！
```

### 円グラフ

```mermaid
pie title 好きな言語
    "Rust" : 40
    "TypeScript" : 30
    "Python" : 20
    "その他" : 10
```

## Mermaid を無効化

Mermaid サポートが不要な場合：

```json
{
    "plugins": ["-mermaid-md-adoc"]
}
```
