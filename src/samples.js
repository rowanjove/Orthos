// Orthos Built-in Samples Repository
window.OrthosSamples = [
  {
    id: 'package.json',
    name: 'Node.js (package.json)',
    filename: 'package.json',
    format: 'json',
    badge: 'Profile',
    content: `{
  "name": "my-service",
  "version": "1.0.0",
  "description": "本地微服务模块",
  "main": "dist/index.js",
  "scripts": {
    "build": "tsc",
    "start": "node dist/index.js",
    "test": "node --test"
  },
  "dependencies": {
    "express": "^4.19.2",
    "dotenv": "^16.4.5"
  },
  "devDependencies": {
    "typescript": "^5.4.5",
    "@types/node": "^20.12.7"
  },
  "engines": {
    "node": ">=18.0.0"
  }
}`,
    schema: {
      title: "package.json 规范",
      type: "object",
      required: ["name", "version"],
      properties: {
        name: { type: "string", description: "项目包名" },
        version: { type: "string", description: "版本号 (SemVer)" },
        description: { type: "string", description: "项目描述" },
        main: { type: "string", description: "入口文件" },
        private: { type: "boolean", description: "是否私有包" }
      }
    }
  },
  {
    id: 'docker-compose',
    name: 'Docker Compose (YAML)',
    filename: 'docker-compose.yml',
    format: 'yaml',
    badge: 'Profile',
    content: `version: "3.8"
services:
  web:
    image: nginx:alpine
    ports:
      - "80:80"
    environment:
      NODE_ENV: production
      API_SECRET: secret_token_xyz987
    depends_on:
      - redis
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
`
  },
  {
    id: 'json',
    name: 'JSON 基础配置',
    filename: 'appsettings.json',
    format: 'json',
    badge: 'JSON',
    content: `{
  "app": {
    "name": "Orthos Workspace",
    "port": 8080,
    "debug": true
  },
  "database": {
    "host": "127.0.0.1",
    "port": 5432,
    "user": "postgres",
    "password": "super_secret_password_123"
  },
  "features": [
    "realtime_validation",
    "visual_editing",
    "secret_masking"
  ]
}`,
    schema: {
      title: "应用基础配置",
      type: "object",
      properties: {
        app: {
          type: "object",
          title: "应用服务设置",
          properties: {
            name: { type: "string", title: "应用名称" },
            port: { type: "integer", title: "监听端口" },
            debug: { type: "boolean", title: "调试模式" }
          }
        },
        database: {
          type: "object",
          title: "数据库配置",
          properties: {
            host: { type: "string", title: "主机地址" },
            port: { type: "integer", title: "端口" },
            user: { type: "string", title: "用户名" },
            password: { type: "string", title: "连接密码" }
          }
        }
      }
    }
  },
  {
    id: 'yaml',
    name: 'YAML 配置',
    filename: 'config.yaml',
    format: 'yaml',
    badge: 'YAML',
    content: `server:
  host: 0.0.0.0
  port: 9000
  workers: 4

logging:
  level: info
  destination: stdout

auth:
  api_key: secret_api_key_8899aabb
  token_expiry_hours: 24
`
  },
  {
    id: 'toml',
    name: 'TOML (Cargo.toml)',
    filename: 'Cargo.toml',
    format: 'toml',
    badge: 'TOML',
    content: `[package]
name = "my_rust_crate"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }

[features]
default = []
extra = []
`
  },
  {
    id: 'csv',
    name: 'CSV 数据表格',
    filename: 'contacts.csv',
    format: 'csv',
    badge: 'CSV',
    content: `id,name,role,department,status
101,Alice Smith,Engineering Lead,Tech,Active
102,Bob Johnson,Senior Designer,Product,Active
103,Charlie Lee,DevOps Engineer,Infrastructure,OnLeave
`
  },
  {
    id: 'env',
    name: 'ENV 环境变量',
    filename: '.env',
    format: 'env',
    badge: 'ENV',
    content: `APP_ENV=production
APP_PORT=3000
DATABASE_URL=postgres://user:db_password_sec77@localhost:5432/main
ACCESS_TOKEN_SECRET=jwt_access_token_private_key_xyz
DEBUG_MODE=false
`
  },
  {
    id: 'ini',
    name: 'INI 系统配置',
    filename: 'system.ini',
    format: 'ini',
    badge: 'INI',
    content: `[core]
cache_enabled = true
max_connections = 128

[network]
interface = eth0
timeout = 30
`
  },
  {
    id: 'xml',
    name: 'XML DOM 清单',
    filename: 'pom.xml',
    format: 'xml',
    badge: 'XML',
    content: `<?xml version="1.0" encoding="UTF-8"?>
<project xmlns="http://maven.apache.org/POM/4.0.0">
  <modelVersion>4.0.0</modelVersion>
  <groupId>com.orthos</groupId>
  <artifactId>config-service</artifactId>
  <version>1.0.0</version>
  <properties>
    <java.version>17</java.version>
  </properties>
</project>
`
  },
  {
    id: 'jsonc',
    name: 'JSONC (带注释 JSON)',
    filename: 'tsconfig.jsonc',
    format: 'jsonc',
    badge: 'JSONC',
    content: `{
  // TypeScript 编译配置
  "compilerOptions": {
    "target": "es2022",
    "module": "commonjs",
    /* 严格模式检查 */
    "strict": true,
    "skipLibCheck": true
  }
}
`
  },
  {
    id: 'json5',
    name: 'JSON5 (支持尾逗号与单引号)',
    filename: 'config.json5',
    format: 'json5',
    badge: 'JSON5',
    content: `{
  // 前端打包规范
  bundleName: 'orthos-core',
  entry: 'src/index.js',
  minify: true,
  env: 'staging',
}
`
  },
  {
    id: 'jsonl',
    name: 'JSONL 数据行',
    filename: 'events.jsonl',
    format: 'jsonl',
    badge: 'JSONL',
    content: `{"id": 1, "event": "app_open", "timestamp": 1727190000}
{"id": 2, "event": "parse_config", "format": "json"}
{"id": 3, "event": "save_file", "success": true}
`
  },
  {
    id: 'tsv',
    name: 'TSV 制表符矩阵',
    filename: 'metrics.tsv',
    format: 'tsv',
    badge: 'TSV',
    content: `metric\tcpu_usage\tmemory_mb\tstatus
api-gateway\t12.5\t256\toptimal
auth-server\t8.2\t192\toptimal
worker-pool\t45.0\t512\theavy
`
  },
  {
    id: 'properties',
    name: 'Properties 属性文件',
    filename: 'application.properties',
    format: 'properties',
    badge: 'Properties',
    content: `# Spring Boot 服务配置
server.port=8080
spring.application.name=orthos-api
logging.level.root=WARN
security.jwt.secret_key=prod_secret_token_key_99
`
  },
  {
    id: 'editorconfig',
    name: 'EditorConfig 代码风格',
    filename: '.editorconfig',
    format: 'editorconfig',
    badge: 'EditorConfig',
    content: `root = true

[*]
charset = utf-8
indent_style = space
indent_size = 2
end_of_line = lf
insert_final_newline = true
trim_trailing_whitespace = true
`
  },
  {
    id: 'gitconfig',
    name: 'GitConfig 版本控制',
    filename: '.gitconfig',
    format: 'gitconfig',
    badge: 'GitConfig',
    content: `[user]
name = Developer
email = dev@example.com

[core]
autocrlf = false
filemode = false
`
  },
  {
    id: 'hcl',
    name: 'HCL / Terraform 设施配置',
    filename: 'main.tf',
    format: 'hcl',
    badge: 'HCL',
    content: `variable "region" {
  default = "us-west-2"
}

resource "aws_s3_bucket" "config_bucket" {
  bucket = "orthos-backup-vault"
}
`
  }
];
