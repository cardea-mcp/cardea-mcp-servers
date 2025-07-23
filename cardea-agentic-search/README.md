# Cardea Agentic Search MCP Server

A Model Context Protocol (MCP) server that provides agentic search capabilities with support for vector search using Qdrant, keyword search using TiDB, or both combined.

## Features

- **Vector Search**: Semantic search using Qdrant vector database with embedding services
- **Keyword Search**: Full-text search using TiDB with intelligent keyword extraction
- **Combined Search**: Use both vector and keyword search simultaneously for comprehensive results
- **Flexible Configuration**: Choose your search mode via command-line subcommands
- **Multiple Transport Types**: Support for both SSE and Streamable HTTP MCP transports

## Architecture

The server is designed with a modular architecture that supports different search backends:

- **Vector Search**: Uses Qdrant for semantic/vector-based search with embedding services
- **Keyword Search**: Uses TiDB for full-text search with intelligent keyword extraction via chat services
- **Combined Search**: Merges results from both vector and keyword search for comprehensive results

## Usage

### Command Line Options

The server supports three search modes through subcommands:

#### Global Options

These options apply to all search modes:

- `-s, --socket-addr`: Socket address to bind to (default: 127.0.0.1:8009)
- `-t, --transport`: Transport type (sse, stream-http) (default: stream-http)

#### 1. Qdrant Vector Search Only

```bash
./cardea-agentic-search-mcp-server qdrant \
    --qdrant-collection my_collection \
    --qdrant-payload-field "full_text" \
    --embedding-service http://localhost:8081/v1 \
    --limit 20 \
    --score-threshold 0.7
```

**Options:**

- `--qdrant-collection`: Collection name in Qdrant (**required**)
- `--qdrant-payload-field`: The name of the field in the payload that contains the source of the document (**required**)
- `--embedding-service`: Embedding service base URL (**required**)
- `--limit`: Maximum number of results (default: 10)
- `--score-threshold`: Score threshold for results (default: 0.5)

**Note:** Qdrant base URL is configured via the `QDRANT_BASE_URL` environment variable (default: http://127.0.0.1:6333)

#### 2. TiDB Keyword Search Only

```bash
./cardea-agentic-search-mcp-server tidb \
    --tidb-ssl-ca /path/to/ca.pem \
    --tidb-table-name my_table \
    --chat-service http://localhost:8080/v1 \
    --limit 15
```

**Options:**

- `--tidb-ssl-ca`: TiDB SSL CA certificate path (**required**)
  - On macOS: typically `/etc/ssl/cert.pem`
  - On Debian/Ubuntu/Arch Linux: typically `/etc/ssl/certs/ca-certificates.crt`
- `--tidb-table-name`: Table name in TiDB (**required**)
- `--chat-service`: Chat service base URL (**required**)
- `--limit`: Maximum number of results (default: 10)
- `--score-threshold`: Score threshold for results (default: 0.5)

#### 3. Combined Search (Both Vector and Keyword)

```bash
./cardea-agentic-search-mcp-server search \
    --qdrant-collection my_collection \
    --qdrant-payload-field "full_text" \
    --tidb-ssl-ca /path/to/ca.pem \
    --tidb-table-name my_table \
    --chat-service http://localhost:8080/v1 \
    --embedding-service http://localhost:8081/v1 \
    --limit 25
```

**Options:**

- `--qdrant-collection`: Collection name in Qdrant (**required**)
- `--qdrant-payload-field`: The name of the field in the payload that contains the source of the document (**required**)
- `--tidb-ssl-ca`: TiDB SSL CA certificate path (**required**)
  - On macOS: typically `/etc/ssl/cert.pem`
  - On Debian/Ubuntu/Arch Linux: typically `/etc/ssl/certs/ca-certificates.crt`
- `--tidb-table-name`: Table name in TiDB (**required**)
- `--chat-service`: Chat service base URL (**required**)
- `--embedding-service`: Embedding service base URL (**required**)
- `--limit`: Maximum number of results (default: 10)
- `--score-threshold`: Score threshold for results (default: 0.5)

**Note:** Qdrant base URL is configured via the `QDRANT_BASE_URL` environment variable (default: http://127.0.0.1:6333)

### Environment Variables

#### For Qdrant Vector Search

- `QDRANT_BASE_URL`: Qdrant database URL (default: http://127.0.0.1:6333)
- `QDRANT_API_KEY`: API key for Qdrant (optional)

#### For TiDB Keyword Search

- `TIDB_CONNECTION`: TiDB connection string in format `mysql://<USERNAME>:<PASSWORD>@<HOST>:<PORT>/<DATABASE>` (required)

#### For External Services

- `CHAT_SERVICE_API_KEY`: API key for chat service (optional)
- `EMBEDDING_SERVICE_API_KEY`: API key for embedding service (optional)

## Examples

### Qdrant Vector Search Example

```bash
export QDRANT_BASE_URL=http://localhost:6333
export QDRANT_API_KEY=your_qdrant_api_key
export EMBEDDING_SERVICE_API_KEY=your_embedding_api_key

./cardea-agentic-search-mcp-server qdrant \
    --qdrant-collection documents \
    --qdrant-payload-field "full_text" \
    --embedding-service http://localhost:8081/v1 \
    --limit 10 \
    --score-threshold 0.6
```

### TiDB Keyword Search Example

```bash
export TIDB_CONNECTION="mysql://root:mypassword@localhost:4000/search_db"
export CHAT_SERVICE_API_KEY=your_chat_api_key

./cardea-agentic-search-mcp-server tidb \
    --tidb-ssl-ca /etc/ssl/certs/ca.pem \
    --tidb-table-name documents \
    --chat-service http://localhost:8080/v1 \
    --limit 20 \
    --score-threshold 0.4
```

### Combined Search Example

```bash
export TIDB_CONNECTION="mysql://root:mypassword@localhost:4000/search_db"
export QDRANT_BASE_URL=http://localhost:6333
export QDRANT_API_KEY=your_qdrant_api_key
export CHAT_SERVICE_API_KEY=your_chat_api_key
export EMBEDDING_SERVICE_API_KEY=your_embedding_api_key

./cardea-agentic-search-mcp-server search \
    --qdrant-collection documents \
    --qdrant-payload-field "full_text" \
    --tidb-ssl-ca /etc/ssl/certs/ca.pem \
    --tidb-table-name documents \
    --chat-service http://localhost:8080/v1 \
    --embedding-service http://localhost:8081/v1 \
    --limit 30 \
    --score-threshold 0.5
```

## How It Works

### Vector Search Process

1. **Query Processing**: The user query is sent to the embedding service to generate a vector representation
2. **Vector Search**: The generated vector is used to search the Qdrant collection for similar documents
3. **Result Formatting**: Results are formatted and returned with scores and metadata

### Keyword Search Process

1. **Keyword Extraction**: The user query is sent to the chat service to extract relevant keywords
2. **Full-text Search**: The extracted keywords are used to perform full-text search in TiDB
3. **Result Formatting**: Results are formatted and returned with document content

### Combined Search Process

1. **Parallel Execution**: Both vector and keyword search are executed in parallel
2. **Result Merging**: Results from both searches are combined and formatted
3. **Comprehensive Results**: Users get both semantic and keyword-based search results

## Development

### Building

```bash
cargo build --package cardea-agentic-search-mcp-server --release
```

### Configuration

The server uses a flexible configuration system that allows you to:

1. Choose your search mode at runtime
2. Configure different backends independently
3. Set appropriate limits and thresholds for each search type
4. Use environment variables for sensitive configuration (like API keys and database credentials)
5. Configure external services for embedding and keyword extraction

### Dependencies

- **Qdrant**: Vector database for semantic search
- **TiDB**: MySQL-compatible database for full-text search
- **Chat Service**: External service for intelligent keyword extraction
- **Embedding Service**: External service for vector generation
