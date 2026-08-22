# -*- coding: utf-8 -*-
"""规则设置窗口。"""

from .widgets import BODY_FONT, SECTION_FONT, SMALL_FONT


class SettingsWindow:
    """基于 customtkinter 的规则设置窗口。"""

    def __init__(self, app):
        self.app = app
        self.ctk = app.ctk
        self.window = None

    def open(self):
        if self.window is not None and self.window.winfo_exists():
            self.window.lift()
            self.window.focus_force()
            return self.window

        ctk = self.ctk
        win = ctk.CTkToplevel(self.app.root)
        win.title("设置 — 排版规则")
        win.geometry("560x660")
        win.minsize(480, 520)
        # 保留系统标题栏；作为主窗口的临时子窗口显示，但不设置 topmost，避免遮挡其他应用。
        win.transient(self.app.root)
        win.attributes("-topmost", False)
        win.protocol("WM_DELETE_WINDOW", self._close)
        win.grid_columnconfigure(0, weight=1)
        win.grid_rowconfigure(1, weight=1)
        self.window = win
        self.app._settings_win = win
        # 设置窗口打开期间主窗口不能抢焦点或覆盖它，但设置窗口仍可移动、调整大小。
        win.grab_set()
        win.focus_force()

        header = ctk.CTkFrame(win, fg_color="transparent")
        header.grid(row=0, column=0, sticky="ew", padx=16, pady=(16, 8))
        header.grid_columnconfigure(0, weight=1)
        ctk.CTkLabel(header, text="排版规则", font=SECTION_FONT, anchor="w").grid(row=0, column=0, sticky="ew")
        ctk.CTkButton(header, text="恢复默认", width=104, height=34,
                      command=self.app._reset_defaults).grid(row=0, column=1, padx=(10, 0))

        scroll = ctk.CTkScrollableFrame(win, corner_radius=14)
        scroll.grid(row=1, column=0, sticky="nsew", padx=16, pady=(0, 10))
        scroll.grid_columnconfigure(0, weight=1)

        self.app._rule_vars = {}
        row = 0
        current_section = None
        for rule in self.app.rules:
            if rule["section"] != current_section:
                current_section = rule["section"]
                ctk.CTkLabel(scroll, text=current_section, font=SECTION_FONT, anchor="w").grid(
                    row=row, column=0, sticky="ew", padx=8, pady=(14 if row else 8, 6)
                )
                row += 1
            name = rule["name"].replace("`", "").strip()
            if rule["disputed"]:
                name += "（争议，默认关闭）"
            var = ctk.BooleanVar(value=rule["key"] in self.app.enabled)
            cb = ctk.CTkCheckBox(
                scroll,
                text=name,
                variable=var,
                font=BODY_FONT,
                height=30,
                command=lambda k=rule["key"], v=var: self.app._toggle_rule(k, v),
            )
            cb.grid(row=row, column=0, sticky="w", padx=12, pady=4)
            self.app._rule_vars[rule["key"]] = var
            row += 1

        bottom = ctk.CTkFrame(win, fg_color="transparent")
        bottom.grid(row=2, column=0, sticky="ew", padx=16, pady=(0, 16))
        bottom.grid_columnconfigure(2, weight=1)
        ctk.CTkButton(bottom, text="全选", width=88, height=36,
                      command=lambda: self.app._set_all_rules(True)).grid(row=0, column=0, padx=(0, 8))
        ctk.CTkButton(bottom, text="全不选", width=88, height=36,
                      command=lambda: self.app._set_all_rules(False)).grid(row=0, column=1)
        ctk.CTkLabel(bottom, text="Light 模式", font=SMALL_FONT, text_color="#666666").grid(
            row=0, column=2, sticky="e", padx=10
        )
        ctk.CTkButton(bottom, text="完成", width=88, height=36, command=self._close).grid(row=0, column=3)
        return win

    def _close(self):
        if self.window is not None and self.window.winfo_exists():
            try:
                self.window.grab_release()
            except Exception:
                pass
            self.window.destroy()
        self.window = None
        self.app._settings_win = None
