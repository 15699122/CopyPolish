#!/usr/bin/env bash
# 中文文案排版助手 —— 一键启动（customtkinter 版）
# 使用项目自带的 .venv（Python 3.14 + tkinter + customtkinter）。
# 用法：bash run.sh   或   ./run.sh
set -e
DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
exec "$DIR/.venv/bin/python" "$DIR/chinese_copywriting_formatter.py" "$@"
