#!/bin/sh
# ---------------------------------------------------------------------------
# Arduino Uno (ATmega328P) Rust 開發環境安裝腳本
# 不依賴 pacman，改用獨立 AVR toolchain 壓縮包安裝。
# 目標環境：Windows + Git for Windows SDK / MSYS2 bash
# ---------------------------------------------------------------------------
set -eu

TOOLS_DIR="/c/tools"

AVR_GCC_VERSION="15.2.0"
AVR_GCC_TAG="v15.2.0-1"
AVR_GCC_DIR="${TOOLS_DIR}/avr-gcc-${AVR_GCC_VERSION}-x64-windows"
AVR_GCC_URL="https://github.com/ZakKemble/avr-gcc-build/releases/download/${AVR_GCC_TAG}/avr-gcc-${AVR_GCC_VERSION}-x64-windows.zip"

AVRDUDE_VERSION="8.1"
AVRDUDE_DIR="${TOOLS_DIR}/avrdude"
AVRDUDE_URL="https://github.com/avrdudes/avrdude/releases/download/v${AVRDUDE_VERSION}/avrdude-v${AVRDUDE_VERSION}-windows-x64.zip"

# 對應 rust-toolchain.toml 所指定的版本
RUST_NIGHTLY="nightly-2025-04-27"

need() {
    command -v "$1" >/dev/null 2>&1 || {
        echo "錯誤：找不到必要指令 '$1'，請先安裝後再執行本腳本。" >&2
        exit 1
    }
}

need curl
need unzip
need rustup
need cargo

echo "==> [1/5] 安裝 Rust nightly toolchain (${RUST_NIGHTLY}) 與 rust-src"
rustup toolchain install "${RUST_NIGHTLY}" --profile minimal
rustup component add rust-src --toolchain "${RUST_NIGHTLY}"

mkdir -p "${TOOLS_DIR}"

echo "==> [2/5] 安裝 AVR-GCC ${AVR_GCC_VERSION} 到 ${AVR_GCC_DIR}"
if [ -x "${AVR_GCC_DIR}/bin/avr-gcc.exe" ]; then
    echo "    已存在，略過下載。"
else
    tmp_zip="/tmp/avr-gcc-${AVR_GCC_VERSION}.zip"
    curl -L --fail --progress-bar -o "${tmp_zip}" "${AVR_GCC_URL}"
    unzip -q -o "${tmp_zip}" -d "${TOOLS_DIR}"
    rm -f "${tmp_zip}"
fi

echo "==> [3/5] 安裝 avrdude ${AVRDUDE_VERSION} 到 ${AVRDUDE_DIR}"
if [ -x "${AVRDUDE_DIR}/avrdude.exe" ]; then
    echo "    已存在，略過下載。"
else
    tmp_zip="/tmp/avrdude-${AVRDUDE_VERSION}.zip"
    curl -L --fail --progress-bar -o "${tmp_zip}" "${AVRDUDE_URL}"
    unzip -q -o "${tmp_zip}" -d "${AVRDUDE_DIR}"
    rm -f "${tmp_zip}"
fi

echo "==> [4/5] 將 AVR toolchain 加入 ~/.bashrc PATH"
PATH_LINE="export PATH=\"${AVR_GCC_DIR}/bin:${AVRDUDE_DIR}:\$PATH\""
if [ -f "${HOME}/.bashrc" ] && grep -qxF "${PATH_LINE}" "${HOME}/.bashrc"; then
    echo "    ~/.bashrc 已含對應 PATH，略過。"
else
    {
        echo ""
        echo "# AVR toolchain (avr-gcc + avrdude) for Rust AVR embedded dev"
        echo "${PATH_LINE}"
    } >> "${HOME}/.bashrc"
    echo "    已附加到 ~/.bashrc（重開 shell 或 source ~/.bashrc 生效）"
fi

echo "==> [5/5] 安裝 ravedude（AVR 燒錄工具）"
if command -v ravedude >/dev/null 2>&1; then
    echo "    已安裝：$(ravedude --version 2>/dev/null || echo 'ravedude')"
else
    cargo install ravedude
fi

echo ""
echo "----------------------------------------------------------------------"
echo "安裝完成。請執行下列其一以啟用新的 PATH："
echo "    source ~/.bashrc        # 於當前 shell 生效"
echo "    （或重開一個新的 bash shell）"
echo ""
echo "驗證："
echo "    avr-gcc --version"
echo "    avrdude -?            # 第一行應為 'Usage: avrdude [options]'"
echo "    ravedude --version"
echo ""
echo "編譯與燒錄："
echo "    cargo build --release"
echo "    RAVEDUDE_PORT=COM3 cargo run --release   # COM3 換成你的 Arduino port"
echo "----------------------------------------------------------------------"
