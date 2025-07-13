# Stepflow Tool System - 文档目录

## 📚 文档结构

### 🎯 核心包文档 (Core Packages)

#### 1. [stepflow-core](./core/stepflow-core.md)
- **职责**: 核心数据结构和接口定义
- **内容**: 
  - 核心数据结构 (ToolSpec, ToolType, etc.)
  - 核心 trait 定义 (ToolHandler, ToolRegistry)
  - 错误类型定义
  - 类型定义和常量
- **接口标准**: 所有工具包必须实现的接口
- **测试标准**: 单元测试覆盖率 > 90%

#### 2. [stepflow-database](./core/stepflow-database.md)
- **职责**: 数据库连接和操作
- **内容**:
  - SQLite 连接管理
  - 数据库模型定义
  - 迁移脚本
  - 数据访问层 (Repository Pattern)
- **接口标准**: 数据库操作的抽象接口
- **测试标准**: 集成测试 + 事务测试

#### 3. [stepflow-registry](./core/stepflow-registry.md)
- **职责**: 工具注册、发现和管理
- **内容**:
  - 工具注册中心实现
  - 工具发现机制
  - 工具验证逻辑
  - 缓存管理
- **接口标准**: 注册和发现 API
- **测试标准**: 注册/发现功能测试 + 性能测试

#### 4. [stepflow-executor](./core/stepflow-executor.md)
- **职责**: 工具执行和调度
- **内容**:
  - 执行引擎实现
  - 任务调度器
  - 执行指标收集
  - 限流管理
- **接口标准**: 执行引擎 API
- **测试标准**: 执行功能测试 + 压力测试

#### 5. [stepflow-sandbox](./core/stepflow-sandbox.md)
- **职责**: 安全执行环境
- **内容**:
  - 沙箱实现
  - 隔离管理
  - 资源限制
  - 安全检查
- **接口标准**: 沙箱管理 API
- **测试标准**: 安全测试 + 隔离测试

#### 6. [stepflow-api](./core/stepflow-api.md)
- **职责**: HTTP API 服务
- **内容**:
  - REST API 路由
  - 中间件实现
  - 请求处理器
  - 认证机制
- **接口标准**: REST API 规范
- **测试标准**: API 测试 + 认证测试

---

### 🛠️ 工具包文档 (Tool Packages)

#### 7. [stepflow-openapi](./tools/stepflow-openapi.md)
- **职责**: OpenAPI/Swagger 工具处理
- **内容**:
  - OpenAPI 规范解析
  - HTTP 客户端实现
  - 认证处理
  - 请求/响应验证
- **接口标准**: OpenAPI 工具接口
- **测试标准**: 集成测试 + Schema 验证

#### 8. [stepflow-asyncapi](./tools/stepflow-asyncapi.md)
- **职责**: AsyncAPI 工具处理
- **内容**:
  - AsyncAPI 规范解析
  - 多协议支持 (MQTT, Kafka, WebSocket)
  - 消息处理
  - 连接管理
- **接口标准**: AsyncAPI 工具接口
- **测试标准**: 协议测试 + 连接验证

#### 9. [stepflow-python](./tools/stepflow-python.md)
- **职责**: Python 脚本执行
- **内容**:
  - Python 运行时管理
  - 虚拟环境管理
  - 依赖安装
  - 安全控制
- **接口标准**: Python 工具接口
- **测试标准**: 环境测试 + 安全验证

#### 10. [stepflow-shell](./tools/stepflow-shell.md)
- **职责**: Shell 命令执行
- **内容**:
  - 多 Shell 支持
  - 命令验证
  - 安全控制
  - 输出处理
- **接口标准**: Shell 工具接口
- **测试标准**: 命令验证 + 安全测试

#### 11. [stepflow-ai](./tools/stepflow-ai.md)
- **职责**: AI 模型调用
- **内容**:
  - 多 AI 提供商支持
  - API 密钥管理
  - 成本跟踪
  - 结构化输出解析
- **接口标准**: AI 工具接口
- **测试标准**: Provider 测试 + 成本跟踪

#### 12. [stepflow-system](./tools/stepflow-system.md)
- **职责**: 系统级操作
- **内容**:
  - 文件系统操作
  - 进程管理
  - 网络操作
  - 数据库查询
- **接口标准**: 系统工具接口
- **测试标准**: 权限测试 + 操作验证

---

### 🔧 服务包文档 (Service Packages)

#### 13. [stepflow-embedding](./services/stepflow-embedding.md)
- **职责**: 文本嵌入和向量化
- **内容**:
  - 本地嵌入模型
  - 向量化服务
  - 相似度计算
- **接口标准**: 嵌入服务接口
- **测试标准**: 向量化测试 + 性能测试

#### 14. [stepflow-search](./services/stepflow-search.md)
- **职责**: 工具搜索和发现
- **内容**:
  - 本地搜索索引
  - 全文搜索
  - 标签搜索
  - 推荐算法
- **接口标准**: 搜索服务接口
- **测试标准**: 搜索功能测试 + 推荐测试

#### 15. [stepflow-monitoring](./services/stepflow-monitoring.md)
- **职责**: 监控和指标收集
- **内容**:
  - 指标收集
  - 日志管理
  - 健康检查
  - 告警机制
- **接口标准**: 监控服务接口
- **测试标准**: 监控测试 + 告警测试

---

### 📦 可执行文件文档 (Binary Packages)

#### 16. [stepflow-server](./binaries/stepflow-server.md)
- **职责**: 主服务器
- **内容**:
  - 服务器启动配置
  - 动态工具加载
  - 服务集成
- **接口标准**: 服务器配置接口
- **测试标准**: 启动测试 + 集成测试

#### 17. [stepflow-cli](./binaries/stepflow-cli.md)
- **职责**: 命令行工具
- **内容**:
  - 命令实现
  - 插件系统 (未来)
  - 交互式界面
- **接口标准**: CLI 命令接口
- **测试标准**: 命令测试 + 交互测试

#### 18. [stepflow-admin](./binaries/stepflow-admin.md)
- **职责**: 管理工具
- **内容**:
  - 系统管理
  - 监控界面
  - 配置管理
- **接口标准**: 管理工具接口
- **测试标准**: 管理功能测试

---

### 📋 通用文档

#### 19. [API 规范](./api/README.md)
- **职责**: API 设计规范
- **内容**:
  - REST API 设计原则
  - 错误处理规范
  - 认证机制
  - 版本控制策略

#### 20. [测试规范](./testing/README.md)
- **职责**: 测试标准和规范
- **内容**:
  - 单元测试规范
  - 集成测试规范
  - 性能测试规范
  - 安全测试规范

#### 21. [部署指南](./deployment/README.md)
- **职责**: 部署和运维
- **内容**:
  - 开发环境配置
  - 生产环境部署
  - 监控和日志
  - 故障排查

#### 22. [开发指南](./development/README.md)
- **职责**: 开发流程和规范
- **内容**:
  - 代码规范
  - Git 工作流
  - 代码审查流程
  - 发布流程

---

## 🚀 文档生成和维护

### 自动化文档生成
```bash
# 生成所有包文档
cargo run -p stepflow-cli doc generate

# 生成特定包文档
cargo run -p stepflow-cli doc generate --package stepflow-openapi

# 生成 API 文档
cargo run -p stepflow-cli doc api

# 生成测试报告
cargo run -p stepflow-cli test report
```

### 文档维护
- 每个包都有独立的文档文件
- 文档与代码同步更新
- 自动生成 API 文档
- 定期审查和更新

---

## 📊 文档状态

| 包名 | 文档状态 | 接口定义 | 测试标准 | 完成度 |
|------|----------|----------|----------|--------|
| stepflow-core | ⏳ 待创建 | ⏳ 待定义 | ⏳ 待制定 | 0% |
| stepflow-database | ⏳ 待创建 | ⏳ 待定义 | ⏳ 待制定 | 0% |
| stepflow-registry | ⏳ 待创建 | ⏳ 待定义 | ⏳ 待制定 | 0% |
| stepflow-executor | ⏳ 待创建 | ⏳ 待定义 | ⏳ 待制定 | 0% |
| stepflow-sandbox | ⏳ 待创建 | ⏳ 待定义 | ⏳ 待制定 | 0% |
| stepflow-api | ⏳ 待创建 | ⏳ 待定义 | ⏳ 待制定 | 0% |
| stepflow-openapi | ⏳ 待创建 | ⏳ 待定义 | ⏳ 待制定 | 0% |
| stepflow-asyncapi | ⏳ 待创建 | ⏳ 待定义 | ⏳ 待制定 | 0% |
| stepflow-python | ⏳ 待创建 | ⏳ 待定义 | ⏳ 待制定 | 0% |
| stepflow-shell | ⏳ 待创建 | ⏳ 待定义 | ⏳ 待制定 | 0% |
| stepflow-ai | ⏳ 待创建 | ⏳ 待定义 | ⏳ 待制定 | 0% |
| stepflow-system | ⏳ 待创建 | ⏳ 待定义 | ⏳ 待制定 | 0% |
| stepflow-embedding | ⏳ 待创建 | ⏳ 待定义 | ⏳ 待制定 | 0% |
| stepflow-search | ⏳ 待创建 | ⏳ 待定义 | ⏳ 待制定 | 0% |
| stepflow-monitoring | ⏳ 待创建 | ⏳ 待定义 | ⏳ 待制定 | 0% |
| stepflow-server | ⏳ 待创建 | ⏳ 待定义 | ⏳ 待制定 | 0% |
| stepflow-cli | ⏳ 待创建 | ⏳ 待定义 | ⏳ 待制定 | 0% |
| stepflow-admin | ⏳ 待创建 | ⏳ 待定义 | ⏳ 待制定 | 0% |

**图例**:
- ⏳ 待创建: 文档尚未创建
- 🔄 进行中: 文档正在编写
- ✅ 已完成: 文档已完成
- 📝 待更新: 文档需要更新

---

## 🎯 下一步计划

1. **优先创建核心包文档** (stepflow-core, stepflow-database)
2. **定义接口标准** (所有包必须实现的接口)
3. **制定测试标准** (单元测试、集成测试、性能测试)
4. **逐个完善工具包文档**
5. **建立文档维护流程**

---

End of Document. 