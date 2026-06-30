# AgentPack Catalog

Pre-built `agentpack.json` manifests for 25 popular MCP servers.

## Usage

Copy the servers you need into your project's `packages/` directory:

```bash
# Copy all
cp -r catalog/packages/* packages/

# Or pick specific ones
cp -r catalog/packages/io.github.modelcontextprotocol__filesystem packages/
cp -r catalog/packages/io.github.modelcontextprotocol__fetch packages/
cp -r catalog/packages/io.github.stripe__agent-toolkit packages/
```

Then add them to your manifest:

```bash
agentpack add io.github.modelcontextprotocol/filesystem@^0.2.0
agentpack add io.github.stripe/agent-toolkit@^0.1.5
agentpack install
```

## Available Servers (25)

### Core / Official
| Package | Capabilities | Tools |
|---------|-------------|-------|
| `modelcontextprotocol/filesystem` | filesystem, file-read/write | read_file, write_file, list_directory, search_files, edit_file |
| `modelcontextprotocol/fetch` | web-fetch, http | fetch |
| `modelcontextprotocol/memory` | memory, knowledge-graph | create_entities, create_relations, search_nodes |
| `modelcontextprotocol/git` | git, version-control | git_status, git_diff, git_log, git_commit, git_branch |
| `modelcontextprotocol/time` | time, timezone | get_current_time, convert_time |
| `modelcontextprotocol/everything` | testing, reference | echo, add, longRunningOperation |
| `modelcontextprotocol/sequential-thinking` | reasoning | sequentialthinking |

### Search
| Package | Capabilities | Tools |
|---------|-------------|-------|
| `modelcontextprotocol/brave-search` | web-search | brave_web_search, brave_local_search |
| `tavily/search` | ai-search, research | tavily_search, tavily_extract |
| `exa/search` | neural-search, semantic | exa_search, exa_find_similar, exa_get_contents |

### Browser Automation
| Package | Capabilities | Tools |
|---------|-------------|-------|
| `modelcontextprotocol/puppeteer` | browser, automation | navigate, screenshot, click, fill, evaluate |
| `executeautomation/playwright` | browser, testing | navigate, screenshot, click, fill |

### Databases
| Package | Capabilities | Tools |
|---------|-------------|-------|
| `modelcontextprotocol/postgres` | database, sql | query |
| `modelcontextprotocol/sqlite` | database, sql | read_query, write_query, create_table, list_tables |
| `upstash/redis` | redis, cache | get, set, del, list |

### SaaS / APIs
| Package | Capabilities | Tools |
|---------|-------------|-------|
| `modelcontextprotocol/github` | github, code-hosting | create_file, search_repos, create_issue, create_pr |
| `modelcontextprotocol/slack` | messaging | send_message, list_channels, search_messages |
| `notionhq/notion` | notes, documents | search, create_page, query_database |
| `stripe/agent-toolkit` | payments, billing | create_payment_intent, list_customers, create_invoice |
| `firecrawl/firecrawl` | web-scraping | scrape, crawl, map |
| `modelcontextprotocol/google-maps` | geocoding, maps | geocode, reverse_geocode, search_places, directions |

### Infrastructure
| Package | Capabilities | Tools |
|---------|-------------|-------|
| `cloudflare/cloudflare` | cdn, dns, workers | kv_get, kv_put, dns_list_records, workers_list |
| `modelcontextprotocol/docker` | containers | list_containers, run_container, stop_container |
| `community/kubernetes` | k8s, orchestration | get_pods, get_deployments, apply_manifest, get_logs |
| `aws/aws` | aws, cloud | s3_list_buckets, lambda_invoke, cloudwatch_get_metrics |

## Declaring Dependencies

These manifests include `provides.capabilities` so you can use semantic resolution:

```json
{
  "requires": [
    {"capability": "web-search"},
    {"capability": "database"}
  ]
}
```

AgentPack will automatically find `brave-search` or `tavily` for search, and `postgres` or `sqlite` for database.
