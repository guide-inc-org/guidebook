# Biểu đồ Mermaid

guidebook hỗ trợ tích hợp sẵn biểu đồ [Mermaid](https://mermaid.js.org/).

## Cách sử dụng

Sử dụng code block `mermaid`:

~~~markdown
```mermaid
graph TD
    A[Bắt đầu] --> B{Quyết định}
    B -->|Có| C[Làm gì đó]
    B -->|Không| D[Làm việc khác]
    C --> E[Kết thúc]
    D --> E
```
~~~

## Ví dụ

### Flowchart

```mermaid
graph TD
    A[Bắt đầu] --> B{Quyết định}
    B -->|Có| C[Làm gì đó]
    B -->|Không| D[Làm việc khác]
```

### Sequence Diagram

```mermaid
sequenceDiagram
    An->>Bình: Xin chào!
    Bình-->>An: Chào bạn!
```

### Pie Chart

```mermaid
pie title Ngôn ngữ yêu thích
    "Rust" : 40
    "TypeScript" : 30
    "Python" : 20
    "Khác" : 10
```

## Tắt Mermaid

Nếu không cần hỗ trợ Mermaid:

```json
{
    "plugins": ["-mermaid-md-adoc"]
}
```
