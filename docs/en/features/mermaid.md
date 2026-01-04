# Mermaid Diagrams

guidebook has built-in support for [Mermaid](https://mermaid.js.org/) diagrams.

## Usage

Use a `mermaid` code block:

~~~markdown
```mermaid
graph TD
    A[Start] --> B{Decision}
    B -->|Yes| C[Do something]
    B -->|No| D[Do something else]
    C --> E[End]
    D --> E
```
~~~

## Examples

### Flowchart

```mermaid
graph TD
    A[Start] --> B{Decision}
    B -->|Yes| C[Do something]
    B -->|No| D[Do something else]
```

### Sequence Diagram

```mermaid
sequenceDiagram
    Alice->>Bob: Hello Bob!
    Bob-->>Alice: Hi Alice!
```

### Pie Chart

```mermaid
pie title Favorite Languages
    "Rust" : 40
    "TypeScript" : 30
    "Python" : 20
    "Other" : 10
```

## Disable Mermaid

If you don't need Mermaid support:

```json
{
    "plugins": ["-mermaid-md-adoc"]
}
```
