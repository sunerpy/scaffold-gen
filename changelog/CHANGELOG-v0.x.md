# Changelog

## [0.10.7](https://github.com/sunerpy/scaffold-gen/compare/v0.10.6...v0.10.7) (2026-07-09)


### Bug Fixes

* **self-update:** 读取 GITHUB_TOKEN 认证并对限流给可操作提示 ([#43](https://github.com/sunerpy/scaffold-gen/issues/43)) ([a0c555d](https://github.com/sunerpy/scaffold-gen/commit/a0c555deae31be6417f55960fc8b21fe239b366d))

## [0.10.6](https://github.com/sunerpy/scaffold-gen/compare/v0.10.5...v0.10.6) (2026-07-09)


### Bug Fixes

* **mcp-python:** 静默 streamable_http 传输层 INFO 噪音(Terminating session) ([#41](https://github.com/sunerpy/scaffold-gen/issues/41)) ([5e2cbec](https://github.com/sunerpy/scaffold-gen/commit/5e2cbec7d1a14e16bb2ed544417814ccf5db726e))

## [0.10.5](https://github.com/sunerpy/scaffold-gen/compare/v0.10.4...v0.10.5) (2026-07-08)


### Bug Fixes

* **mcp-python:** 统一日志时间戳为本地带时区,覆盖 reload worker 子进程 ([#39](https://github.com/sunerpy/scaffold-gen/issues/39)) ([11179e5](https://github.com/sunerpy/scaffold-gen/commit/11179e5e92374bd1bff379dd3fb66d73aeb0a271))

## [0.10.4](https://github.com/sunerpy/scaffold-gen/compare/v0.10.3...v0.10.4) (2026-07-08)


### Bug Fixes

* **mcp-python:** 生成项目开箱通过 ruff check(消除 E501/E402/I001) ([#37](https://github.com/sunerpy/scaffold-gen/issues/37)) ([5e16c4a](https://github.com/sunerpy/scaffold-gen/commit/5e16c4a7bc8cebc47181bad815d604ad95864227))

## [0.10.3](https://github.com/sunerpy/scaffold-gen/compare/v0.10.2...v0.10.3) (2026-07-07)


### Bug Fixes

* **new:** 等价命令无条件输出会触发交互的 flag,保证复跑零交互 ([#34](https://github.com/sunerpy/scaffold-gen/issues/34)) ([3af6a58](https://github.com/sunerpy/scaffold-gen/commit/3af6a58053061f0ba986958c113442f9e1c28f38))

## [0.10.2](https://github.com/sunerpy/scaffold-gen/compare/v0.10.1...v0.10.2) (2026-07-07)


### Bug Fixes

* **mcp-python:** 统一 uvicorn 日志到 structlog 并消除 make test 的 authlib 告警 ([#31](https://github.com/sunerpy/scaffold-gen/issues/31)) ([14ebbbe](https://github.com/sunerpy/scaffold-gen/commit/14ebbbe5842b03ac126eb8cd442f72c7e29ed7c5))

## [0.10.1](https://github.com/sunerpy/scaffold-gen/compare/v0.10.0...v0.10.1) (2026-07-07)


### Bug Fixes

* **frontend:** 补齐 React/Vue3 模板的 eslint + prettier 依赖与配置 ([#29](https://github.com/sunerpy/scaffold-gen/issues/29)) ([9c9b24e](https://github.com/sunerpy/scaffold-gen/commit/9c9b24e9fa198f37d752c1b8aa6d1795a7b2a086))

## [0.10.0](https://github.com/sunerpy/scaffold-gen/compare/v0.9.0...v0.10.0) (2026-07-07)


### Features

* **react:** 将 React 迁移为内嵌离线生成(镜像 Vue3) ([#27](https://github.com/sunerpy/scaffold-gen/issues/27)) ([48efbfb](https://github.com/sunerpy/scaffold-gen/commit/48efbfb1502ebafc792035b9d339fb7ed77671f7))

## [0.9.0](https://github.com/sunerpy/scaffold-gen/compare/v0.8.0...v0.9.0) (2026-07-06)


### Features

* **python:** harden FastAPI/mcp-python generators and centralize versioning ([#24](https://github.com/sunerpy/scaffold-gen/issues/24)) ([bdbbfb2](https://github.com/sunerpy/scaffold-gen/commit/bdbbfb2e9da393ffb7237456bd2203284fb7df3a))

## [0.8.0](https://github.com/sunerpy/scaffold-gen/compare/v0.7.1...v0.8.0) (2026-07-01)


### Features

* 恢复并落地 Python MCP server 脚手架 (mcp-python) ([#22](https://github.com/sunerpy/scaffold-gen/issues/22)) ([ed11d18](https://github.com/sunerpy/scaffold-gen/commit/ed11d18d29159ddf3072752ef4e1a6a3c76bcb45))

## [0.7.1](https://github.com/sunerpy/scaffold-gen/compare/v0.7.0...v0.7.1) (2026-07-01)


### Bug Fixes

* self-update 已是最新版本时不再提示下载升级 ([#19](https://github.com/sunerpy/scaffold-gen/issues/19)) ([c30d798](https://github.com/sunerpy/scaffold-gen/commit/c30d798d3149768767ec3cfac9fa9b59a8cc1c4e))

## [0.7.0](https://github.com/sunerpy/scaffold-gen/compare/v0.6.0...v0.7.0) (2026-06-25)


### Features

* 新增 skill 子命令,安装引导式项目脚手架 agent skill ([#17](https://github.com/sunerpy/scaffold-gen/issues/17)) ([73d7e57](https://github.com/sunerpy/scaffold-gen/commit/73d7e573426e9c061641e27e27b34ffd6bba6cf1))

## [0.6.0](https://github.com/sunerpy/scaffold-gen/compare/v0.5.0...v0.6.0) (2026-06-25)


### Features

* 交互式 new 结束后打印等价的非交互命令 ([#15](https://github.com/sunerpy/scaffold-gen/issues/15)) ([294179e](https://github.com/sunerpy/scaffold-gen/commit/294179e5638ae59a1a1c675f6e107542a6a7e7d9))

## [0.5.0](https://github.com/sunerpy/scaffold-gen/compare/v0.4.1...v0.5.0) (2026-06-25)


### Features

* Vue3 脚手架支持 .env 配置驱动 (host/port/allowedHosts/API) ([#12](https://github.com/sunerpy/scaffold-gen/issues/12)) ([a128ede](https://github.com/sunerpy/scaffold-gen/commit/a128ede56abbf53b60b81de1c9054400c5d7dcb9))

## [0.4.1](https://github.com/sunerpy/scaffold-gen/compare/v0.4.0...v0.4.1) (2026-06-25)


### Bug Fixes

* 修复 FastAPI 脚手架 uvicorn reload 死循环 ([#10](https://github.com/sunerpy/scaffold-gen/issues/10)) ([1f3576b](https://github.com/sunerpy/scaffold-gen/commit/1f3576ba32589374995c0fe4b760e1216bbfbc91))

## [0.4.0](https://github.com/sunerpy/scaffold-gen/compare/v0.3.0...v0.4.0) (2026-06-25)


### Features

* Python structlog 日志 + CodeGraph 接入说明 + 可选 Makefile/Dockerfile 构建 ([#8](https://github.com/sunerpy/scaffold-gen/issues/8)) ([9659a72](https://github.com/sunerpy/scaffold-gen/commit/9659a72d4b779ea45d9cb3ab54409c7a99e4d9de))

## [0.3.0](https://github.com/sunerpy/scaffold-gen/compare/v0.2.0...v0.3.0) (2026-06-25)


### Features

* 新增 self-update / completions / version / list 子命令 ([#6](https://github.com/sunerpy/scaffold-gen/issues/6)) ([b9b38c9](https://github.com/sunerpy/scaffold-gen/commit/b9b38c987b2663e5d27c51a753e539bbfbabe005))

## [0.2.0](https://github.com/sunerpy/scaffold-gen/compare/v0.1.1...v0.2.0) (2026-06-24)


### Features

* Vue3 改为内嵌模板快速脚手架(离线可用) ([7e24858](https://github.com/sunerpy/scaffold-gen/commit/7e2485837d381667d3f5d89d4895eebea59a64ff))
* 新增 Go + Gin MCP server 脚手架 ([ed0aa68](https://github.com/sunerpy/scaffold-gen/commit/ed0aa681ab36c2594bb298ac3b3966cc7896fe38))
* 新增 Python + FastAPI 配置驱动脚手架 ([3c9fc06](https://github.com/sunerpy/scaffold-gen/commit/3c9fc06ee29de2358a97e978a13be3e727c1bca5))

## [0.1.1](https://github.com/sunerpy/scaffold-gen/compare/v0.1.0...v0.1.1) (2026-06-24)


### Bug Fixes

* **ci:** 发布到 crates.io 时允许工作区脏状态 ([99ae34a](https://github.com/sunerpy/scaffold-gen/commit/99ae34a475b91d5e5866ae2e88bc7250a21ced37))
* 修复真实缺陷与错误处理 ([68afe24](https://github.com/sunerpy/scaffold-gen/commit/68afe24a68a6cc1e774bdbc0c37d96e251826f70))

## [0.1.0](https://github.com/sunerpy/scaffold-gen/compare/v0.0.8...v0.1.0) (2026-06-24)


### Features

* **ci:** 改进 CI 和 Release 工作流触发机制 ([ad7406c](https://github.com/sunerpy/scaffold-gen/commit/ad7406cc607d1a6d27a5ac614ee8666a8765fca2))
* **ci:** 改进 CI 和 Release 工作流触发机制 ([64bd5e9](https://github.com/sunerpy/scaffold-gen/commit/64bd5e9cc0bfacec3889da463cdf5137a1f85a4b))
* 工程化项目发布与文档体系 ([04a6a89](https://github.com/sunerpy/scaffold-gen/commit/04a6a891ad5c57f284c65c3f00c01e8d8273feb3))

## Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).
