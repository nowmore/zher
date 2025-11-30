# zhe'r (这儿)

[English](./README.md) | [中文](./README_cn.md)

一款轻量级的、所见即所得的纯局域网文本/文件共享应用。无需联网，无需安装，开箱即用。

## ✨ 功能特点

- **🚀 纯净局域网体验**：完全在本地网络运行，不依赖互联网，无需外部服务器。数据就在“这儿”，安全、私密。
- **📂 极速文件/文件夹传输**：支持拖拽上传，局域网内满速传输。大文件、文件夹轻松分享给所有人。
- **📋 富文本共享**：支持文本、链接、代码片段和格式化内容的分享。所见即所得 - 分享过程中保持原始格式不变。
- **📱 全平台兼容**：只需在一台电脑（Windows/Mac/Linux）上运行，任何设备（iPhone、Android、iPad、平板等）通过浏览器即可访问，界面完美适配移动端。

![share text](assets/share_text.webp)
![share file to phone](assets/2android.webp)
## 🛠️ 技术栈

- **后端**：Rust (Axum + Socketioxide) - 提供高性能、高并发支持。
- **前端**：Vue 3 + Tailwind CSS - 打造流畅、美观的用户界面。

## 🏗️ 构建指南

需预先安装 Node.js 和 Rust 开发环境。

### Windows

直接运行批处理脚本：
```bat
build.bat
```

### Linux / macOS

运行 Shell 脚本：
```bash
./build.sh
```

## 🚀 使用说明

1. 运行构建生成的 `zher` (或 `zher.exe`) 程序。
2. 本机浏览器访问 `http://localhost:4836`。
3. 在同一局域网下的其他设备（如手机），输入运行服务的电脑 IP 地址访问（例如 `http://192.168.1.100:4836`）。

## 许可证

MIT
