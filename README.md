# zhe'r (这儿)

[English](./README.md) | [中文](./README_cn.md)

A lightweight, pure LAN application for instant chat and file sharing. No internet required, no installation needed—what you see is what you get.

## ✨ Features

- **🚀 Pure LAN Experience**: Works entirely within your local network. No external servers, no internet dependency. Just run and connect.
- **📂 Fast File Transfer**: Drag and drop files to share them instantly with everyone on the network. High-speed transfer limited only by your WiFi/LAN speed. Supports both files and folders.
- **📋 Rich Text Sharing**: Send text, links, code snippets, and formatted content across devices. What you see is what you get - the formatting stays intact during sharing.
- **📱 Cross-Platform & Mobile Ready**: Run the server on your PC (Windows/Linux/macOS) and access it from any device (iOS, Android, Tablets) via browser. Responsive design adapts to any screen size.

![share text](assets/share_text.webp)
![share file to phone](assets/2android.webp)

## 🛠️ Technology Stack

- **Backend**: Rust (Axum + Socketioxide) - High performance and safety.
- **Frontend**: Vue 3 + Tailwind CSS - Modern, reactive, and beautiful UI.

## 🏗️ Building

Requirements: Node.js and Rust.

### Windows

```bat
build.bat
```

### Linux / macOS

```bash
./build.sh
```

## 🚀 Usage

1. Run the generated executable (`zher.exe` or `zher`).
2. Open your browser and go to `http://localhost:4836`.
3. Share the URL (e.g., `http://192.168.1.x:4836`) with other devices on the same WiFi/LAN.

## 🧪 Testing

The project includes comprehensive test suites for both frontend and backend.

### Quick Test

**Windows:**
```bat
run-tests.bat
```

**Linux/macOS:**
```bash
./run-tests.sh
```

### Individual Tests

**Frontend:**
```bash
cd frontend
npm test              # Watch mode
npm run test:run      # Single run
npm run test:coverage # With coverage
```

**Backend:**
```bash
cd backend
cargo test            # All tests
cargo test --lib      # Unit tests only
```

See [TESTING.md](./TESTING.md) for detailed testing documentation.

### Test Coverage

- ✅ Frontend: 46+ tests (Components, Composables, Utils, E2E)
- ✅ Backend: 27 tests (State, Utils, Integration, Discovery)
- ✅ CI/CD: Automated testing via GitHub Actions

## License

MIT
