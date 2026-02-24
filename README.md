<p align="center">
  <img src="https://cdn.prod.website-files.com/68e09cef90d613c94c3671c0/697e805a9246c7e090054706_logo_horizontal_grey.png" alt="Yeti" width="200" />
</p>

---

# app-redirector

[![Yeti](https://img.shields.io/badge/Yeti-Application-blue)](https://yetirocks.com)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

> **[Yeti](https://yetirocks.com)** — The Performance Platform for Agent-Driven Development.
> Schema-driven APIs, real-time streaming, and vector search. From prompt to production.

URL redirect management with rule evaluation, versioning, and bulk upload. Create static and regex-based redirect rules, manage host-specific configurations, and activate rule sets atomically via versions.

## Features

- Static and regex redirect rules
- Host-specific configurations
- Time-windowed activation
- Bulk CSV/JSON upload
- Version management
- Analytics/metrics endpoint

## Installation

```bash
cd ~/yeti/applications
git clone https://github.com/yetirocks/app-redirector.git
cd app-redirector/source
npm install
npm run build
```

## Project Structure

```
app-redirector/
├── config.yaml              # App configuration
├── schemas/
│   └── redirect.graphql     # Rule, Hosts, Version tables
├── resources/
│   ├── check_redirect.rs    # Test URL resolution against rules
│   ├── perform_redirect.rs  # Execute redirect with status code
│   ├── redirect_upload.rs   # Bulk CSV/JSON rule import
│   └── redirect_metrics.rs  # Analytics and metrics endpoint
├── data/
│   └── sample-rules.json    # Seed redirect rules
└── source/                  # React/Vite frontend
    ├── index.html
    ├── package.json
    ├── vite.config.ts
    └── src/
```

## Configuration

```yaml
name: "URL Redirector"
app_id: "app-redirector"
version: "1.0.0"
description: "URL redirect management with rule checking, versioning, and bulk CSV upload"
enabled: true
rest: true

schemas:
  - schemas/redirect.graphql

resources:
  - resources/*.rs

dataLoader: data/*.json

static_files:
  path: web
  route: /
  index: index.html
  notFound:
    file: index.html
    statusCode: 200
  build:
    sourceDir: source
    command: npm run build
```

## Schema

**redirect.graphql** -- Rules, Hosts, and Versions:
```graphql
type Rule @table(name: "rule", database: "app-redirector") @export(name: "rule") {
  id: ID!
  staticPath: String
  regexPattern: String
  targetUrl: String!
  statusCode: Int!
  queryStringOp: String
  preserveParams: String
  appendParams: String
  utcStartTime: String
  utcEndTime: String
  host: String
  version: String
  priority: Int
  active: Boolean!
  description: String
  createdAt: String!
  updatedAt: String!
}

type Hosts @table(name: "hosts", database: "app-redirector") @export(name: "hosts") {
  id: ID!
  activeVersion: String
  enabled: Boolean!
  fallbackUrl: String
  description: String
  createdAt: String!
  updatedAt: String!
}

type Version @table(name: "version", database: "app-redirector") @export(name: "version") {
  id: ID!
  name: String!
  active: Boolean!
  activatedAt: String
  description: String
  createdAt: String!
  updatedAt: String!
}
```

## Development

```bash
cd source

# Install dependencies
npm install

# Start dev server with HMR
npm run dev

# Build for production
npm run build
```

---

Built with [Yeti](https://yetirocks.com) | The Performance Platform for Agent-Driven Development
