#!/usr/bin/env bash
# 启动开发模式。在没有图形环境变量的 shell（tty / SSH）里自动探测 Wayland 或 X11。
set -euo pipefail

if [[ -z "${WAYLAND_DISPLAY:-}" && -z "${DISPLAY:-}" ]]; then
  export XDG_RUNTIME_DIR="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}"

  for socket in "${XDG_RUNTIME_DIR}"/wayland-*; do
    [[ -S "$socket" ]] || continue
    export WAYLAND_DISPLAY="$(basename "$socket")"
    break
  done

  if [[ -z "${WAYLAND_DISPLAY:-}" ]]; then
    for socket in /tmp/.X11-unix/X*; do
      [[ -S "$socket" && -O "$socket" ]] || continue
      export DISPLAY=":${socket##*/X}"
      break
    done
  fi
fi

if [[ -z "${WAYLAND_DISPLAY:-}" && -z "${DISPLAY:-}" ]]; then
  echo "找不到可用的 Wayland / X11 显示。" >&2
  echo "请在桌面终端运行，或手动设置 WAYLAND_DISPLAY / DISPLAY。" >&2
  exit 1
fi

echo "图形环境：WAYLAND_DISPLAY=${WAYLAND_DISPLAY:-未设置} DISPLAY=${DISPLAY:-未设置}"
exec npm run tauri dev
