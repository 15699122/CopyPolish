# -*- coding: utf-8 -*-
"""customtkinter 主界面。"""

import os
import sys

from ccw_engine import (
    format_text,
    get_enabled_defaults,
    initialize,
    load_rules,
    load_settings,
    save_settings,
)

from .settings_window import SettingsWindow
from .widgets import APP_NAME, BODY_FONT, SMALL_FONT, TITLE_FONT, make_card


def configure_windows_dpi():
    """在 Windows 下尽量启用 DPI 感知，失败不影响启动。"""
    if sys.platform != "win32":
        return
    try:
        import ctypes
        ctypes.windll.shcore.SetProcessDpiAwareness(1)
    except Exception:
        try:
            import ctypes
            ctypes.windll.user32.SetProcessDPIAware()
        except Exception:
            pass


def _import_gui():
    """惰性导入 customtkinter，保证引擎测试不依赖 GUI。"""
    import tkinter as tk  # noqa: F401
    import customtkinter as ctk
    return ctk


class FormatterApp:
    """中文文案排版助手主窗口。"""

    def __init__(self, ctk=None):
        self.ctk = ctk or _import_gui()
        self.ctk.set_appearance_mode("Light")
        self.ctk.set_widget_scaling(1.0)
        self.ctk.set_window_scaling(1.0)
        # 显式初始化（原为模块导入时自动 ensure_defaults，已改为按需调用）
        initialize()
        self.rules = load_rules()
        saved_enabled, saved_last = load_settings()
        self.enabled = saved_enabled if saved_enabled else set(get_enabled_defaults())
        self._last_input = saved_last or ""
        self._format_job = None
        self._drag_start_x = 0
        self._drag_start_y = 0
        self._is_maximized = False
        self._normal_geometry = "920x720"
        self._settings_win = None
        self._settings_controller = SettingsWindow(self)
        self._rule_vars = {}
        self._build_window()

    def _build_window(self):
        ctk = self.ctk
        self.root = ctk.CTk()
        self.root.title(APP_NAME)
        self.root.geometry("920x720")
        self.root.minsize(720, 540)
        self.root.overrideredirect(True)
        self.root.protocol("WM_DELETE_WINDOW", self._on_close)
        self.root.bind("<Map>", self._on_map)
        self.root.grid_columnconfigure(0, weight=1)
        self.root.grid_rowconfigure(0, weight=1)

        self.main = ctk.CTkFrame(self.root, corner_radius=18)
        self.main.grid(row=0, column=0, sticky="nsew")
        self.main.grid_columnconfigure(0, weight=1)
        self.main.grid_rowconfigure(0, weight=0)
        self.main.grid_rowconfigure(1, weight=1)
        self.main.grid_rowconfigure(2, weight=1)

        self.title_bar = ctk.CTkFrame(self.main, fg_color="transparent", height=74)
        self.title_bar.grid(row=0, column=0, sticky="ew", padx=18, pady=(14, 8))
        self.title_bar.grid_columnconfigure(0, weight=1)
        self.title_bar.bind("<ButtonPress-1>", self._start_move)
        self.title_bar.bind("<B1-Motion>", self._do_move)
        self.title_bar.bind("<Double-Button-1>", self._toggle_maximize)

        title_group = ctk.CTkFrame(self.title_bar, fg_color="transparent")
        title_group.grid(row=0, column=0, sticky="w")
        title_group.bind("<ButtonPress-1>", self._start_move)
        title_group.bind("<B1-Motion>", self._do_move)
        title_group.bind("<Double-Button-1>", self._toggle_maximize)
        title_label = ctk.CTkLabel(title_group, text=APP_NAME, font=TITLE_FONT, anchor="w")
        title_label.grid(row=0, column=0, sticky="w")
        title_label.bind("<ButtonPress-1>", self._start_move)
        title_label.bind("<B1-Motion>", self._do_move)
        title_label.bind("<Double-Button-1>", self._toggle_maximize)
        subtitle = ctk.CTkLabel(
            title_group,
            text="Light 模式 · 实时保护 LaTeX / Markdown 结构",
            font=SMALL_FONT,
            text_color="#666666",
            anchor="w",
        )
        subtitle.grid(row=1, column=0, sticky="w", pady=(2, 0))
        subtitle.bind("<ButtonPress-1>", self._start_move)
        subtitle.bind("<B1-Motion>", self._do_move)
        subtitle.bind("<Double-Button-1>", self._toggle_maximize)

        controls = ctk.CTkFrame(self.title_bar, fg_color="transparent")
        controls.grid(row=0, column=1, sticky="ne", padx=(12, 0), pady=(2, 0))
        self.minimize_button = self._make_window_button(controls, "#FEBC2E", "#FFD15C", self._minimize_window)
        self.maximize_button = self._make_window_button(controls, "#28C840", "#5DDF73", self._toggle_maximize)
        self.close_button = self._make_window_button(controls, "#FF5F57", "#FF7B73", self._on_close)
        self.minimize_button.grid(row=0, column=0, padx=(0, 8))
        self.maximize_button.grid(row=0, column=1, padx=(0, 8))
        self.close_button.grid(row=0, column=2)

        input_card = make_card(ctk, self.main, "输入文字（粘贴后自动排版）")
        input_card.grid(row=1, column=0, sticky="nsew", padx=18, pady=(0, 10))
        self.input_text = ctk.CTkTextbox(input_card, wrap="word", font=BODY_FONT, undo=True)
        self.input_text.grid(row=1, column=0, sticky="nsew", padx=12, pady=(0, 12))
        self.input_text.bind("<KeyRelease>", self._on_text_changed)
        self.input_text.bind("<<Paste>>", self._on_text_changed)
        if self._last_input:
            self.input_text.insert("1.0", self._last_input)

        output_card = make_card(ctk, self.main, "规范化结果（实时）")
        output_card.grid(row=2, column=0, sticky="nsew", padx=18, pady=(0, 10))
        self.output_text = ctk.CTkTextbox(output_card, wrap="word", font=BODY_FONT)
        self.output_text.grid(row=1, column=0, sticky="nsew", padx=12, pady=(0, 12))

        bar = ctk.CTkFrame(self.main, fg_color="transparent")
        bar.grid(row=3, column=0, sticky="ew", padx=18, pady=(0, 10))
        bar.grid_columnconfigure(2, weight=1)
        self.settings_button = ctk.CTkButton(bar, text="设置", width=96, height=38, command=self._open_settings)
        self.clear_button = ctk.CTkButton(bar, text="清除输入", width=104, height=38, command=self._clear_input)
        self.copy_button = ctk.CTkButton(bar, text="复制结果", width=112, height=38, command=self._copy_output)
        self.settings_button.grid(row=0, column=0, padx=(0, 8), sticky="w")
        self.clear_button.grid(row=0, column=1, sticky="w")
        self.copy_button.grid(row=0, column=3, sticky="e")

        if self._last_input:
            self._set_output(format_text(self._last_input, self.enabled))
        else:
            self._set_output("")

    def _make_window_button(self, parent, color, hover_color, command):
        return self.ctk.CTkButton(
            parent,
            text="",
            width=16,
            height=16,
            corner_radius=999,
            border_width=0,
            fg_color=color,
            hover_color=hover_color,
            command=command,
        )

    def _start_move(self, event):
        if self._is_maximized:
            return
        self._drag_start_x = event.x_root - self.root.winfo_x()
        self._drag_start_y = event.y_root - self.root.winfo_y()

    def _do_move(self, event):
        if self._is_maximized:
            return
        x = event.x_root - self._drag_start_x
        y = event.y_root - self._drag_start_y
        self.root.geometry("+%d+%d" % (x, y))

    def _on_map(self, _event=None):
        try:
            self.root.overrideredirect(True)
        except Exception:
            pass

    def _minimize_window(self):
        try:
            self.root.overrideredirect(False)
            self.root.iconify()
        except Exception:
            pass

    def _toggle_maximize(self, _event=None):
        if self._is_maximized:
            self.root.geometry(self._normal_geometry)
            self._is_maximized = False
            return
        current_geometry = self.root.geometry()
        if isinstance(current_geometry, str):
            self._normal_geometry = current_geometry
        width = self.root.winfo_screenwidth()
        height = self.root.winfo_screenheight()
        self.root.geometry("%dx%d+0+0" % (width, height))
        self._is_maximized = True

    def _on_text_changed(self, _event=None):
        if self._format_job is not None:
            self.root.after_cancel(self._format_job)
        self._format_job = self.root.after(160, self._reformat)

    def _reformat(self):
        self._format_job = None
        content = self.input_text.get("1.0", "end-1c")
        self._last_input = content
        self._set_output(format_text(content, self.enabled))
        self._save_state()

    def _set_output(self, text):
        self.output_text.configure(state="normal")
        self.output_text.delete("1.0", "end")
        self.output_text.insert("1.0", text)
        self.output_text.configure(state="disabled")

    def _copy_output(self):
        content = self.output_text.get("1.0", "end-1c")
        if not content:
            self._flash_button_text(self.copy_button, "没有内容")
            return
        self.root.clipboard_clear()
        self.root.clipboard_append(content)
        self.root.update_idletasks()
        self._flash_button_text(self.copy_button, "已复制")

    def _clear_input(self):
        self.input_text.delete("1.0", "end")
        self._last_input = ""
        self._set_output("")
        self._save_state()
        self._flash_button_text(self.clear_button, "已清除")

    def _flash_button_text(self, button, temporary_text, delay=1200):
        original_text = button.cget("text")
        button.configure(text=temporary_text)
        self.root.after(delay, lambda: button.configure(text=original_text))

    def _save_state(self):
        save_settings(self.enabled, self._last_input)

    def _open_settings(self):
        self._settings_controller.open()

    def _toggle_rule(self, key, var):
        if var.get():
            self.enabled.add(key)
        else:
            self.enabled.discard(key)
        self._reformat()
        self._save_state()

    def _set_all_rules(self, on):
        for key, var in self._rule_vars.items():
            var.set(on)
            if on:
                self.enabled.add(key)
            else:
                self.enabled.discard(key)
        self._reformat()
        self._save_state()

    def _reset_defaults(self):
        self.enabled = set(get_enabled_defaults())
        for key, var in self._rule_vars.items():
            var.set(key in self.enabled)
        self._reformat()
        self._save_state()

    def _on_close(self):
        for job in (self._format_job,):
            if job is not None:
                try:
                    self.root.after_cancel(job)
                except Exception:
                    pass
        self.root.destroy()


def main():
    configure_windows_dpi()
    try:
        ctk = _import_gui()
    except Exception as exc:
        print("无法导入 GUI 库（tkinter / customtkinter）：%s" % exc)
        print("请在项目虚拟环境中安装依赖，例如：")
        print("  cd %r" % os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
        print("  .venv/bin/python -m pip install customtkinter")
        return 1
    app = FormatterApp(ctk)
    app.root.mainloop()
    return 0
