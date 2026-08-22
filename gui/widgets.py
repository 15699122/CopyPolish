# -*- coding: utf-8 -*-
"""GUI 可复用控件和视觉常量。"""

APP_NAME = "中文文案排版助手"
FONT_FAMILY = "Microsoft YaHei UI"
TITLE_FONT = (FONT_FAMILY, 24, "bold")
SECTION_FONT = (FONT_FAMILY, 16, "bold")
BODY_FONT = (FONT_FAMILY, 14)
SMALL_FONT = (FONT_FAMILY, 12)


def make_card(ctk, parent, title):
    """创建带标题的响应式卡片区域。"""
    frame = ctk.CTkFrame(parent, corner_radius=14)
    frame.grid_columnconfigure(0, weight=1)
    frame.grid_rowconfigure(1, weight=1)
    label = ctk.CTkLabel(frame, text=title, font=SECTION_FONT, anchor="w")
    label.grid(row=0, column=0, sticky="ew", padx=14, pady=(12, 6))
    return frame
