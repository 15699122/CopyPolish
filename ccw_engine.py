# -*- coding: utf-8 -*-
"""中文文案排版助手 · 规则引擎（纯 Python，不依赖 GUI）

规则来源：chinese-copywriting-guidelines（简体中文分支）
https://raw.githubusercontent.com/sparanoid/chinese-copywriting-guidelines/master/README.zh-Hans.md

本模块实现 13 条排版规则的文本转换，可独立于 GUI 运行与测试：
  空格        中英文之间需要增加空格 / 中文与数字之间需要增加空格
              数字与单位之间需要增加空格 / 全角标点与其他字符之间不加空格
              用 text-spacing 来挽救？（说明）
  标点符号    不重复使用标点符号
  全角和半角  使用全角中文标点 / 数字使用半角字符
              遇到完整的英文整句、特殊名词，其内容使用半角标点
  名词        专有名词使用正确的大小写 / 不要使用不地道的缩写
  争议        链接之间增加空格 / 简体中文使用直角引号
"""

import importlib
import os
import re

# ---------------------------------------------------------------------------
# 标准库轻量 YAML 子集读写（零第三方依赖，供 rules.yaml 使用）。
# 优先使用 PyYAML（若可用），否则回退内置实现。支持注释、嵌套 dict/list、
# 以及 str/int/bool/float/None 标量。解析足够支撑本项目 rules.yaml。
# 说明：这里用 importlib.import_module 做动态导入而非静态 import yaml，
# 以在不安装 PyYAML 的环境下避免 Pylance 报“无法从源解析导入”告警；
# 运行时行为与静态导入完全一致（可用则用，不可用则回退内置实现）。
# ---------------------------------------------------------------------------
try:
    _pyyaml = importlib.import_module("yaml")
    _HAS_YAML = True
except Exception:  # noqa: BLE001
    _pyyaml, _HAS_YAML = None, False

# ---------------------------------------------------------------------------
# 常量与字符集
# ---------------------------------------------------------------------------
_CJK = r"\u3400-\u4dbf\u4e00-\u9fff\uf900-\ufaff"
_CJK_RE = re.compile(r"[%s]" % _CJK)

# 半角 -> 全角（中文语境）
_FW_MAP = {
    ",": "，", ";": "；", ":": "：", "!": "！", "?": "？",
    "(": "（", ")": "）", ".": "。",
}
# 全角 -> 半角（英文整句 / 特殊名词语境）
_HW_MAP = {
    "，": ",", "。": ".", "：": ":", "；": ";", "！": "!",
    "？": "?", "＆": "&", "（": "(", "）": ")",
}

# 需要保护的内容：Markdown / LaTeX / 代码 / URL / 邮箱。
# 保护顺序非常重要：外层结构先于内层结构，避免链接里的 URL、代码块里的
# 反引号、公式里的 $ 被拆开处理。
_MD_FENCE_RE = re.compile(r"(^|\n)([ \t]*)(`{3,}|~{3,})[^\n]*\n.*?\n\2\3[ \t]*(?=\n|$)", re.S)
_LATEX_ENV_RE = re.compile(
    r"\\begin\{(equation\*?|align\*?|gather\*?|multline\*?|matrix|pmatrix|bmatrix|cases)\}.*?\\end\{\1\}",
    re.S,
)
_LATEX_DISPLAY_BRACKET_RE = re.compile(r"\\\[.*?\\\]", re.S)
_LATEX_INLINE_PAREN_RE = re.compile(r"\\\(.*?\\\)", re.S)
_LATEX_DISPLAY_DOLLAR_RE = re.compile(r"(?<!\\)\$\$(?!\$).*?(?<!\\)\$\$", re.S)
_LATEX_INLINE_DOLLAR_RE = re.compile(r"(?<!\\)\$(?!\s|\$)(?:\\.|[^$\n\\]){1,300}?(?<!\\)\$(?!\$)")
_LATEX_COMMAND_RE = re.compile(r"\\[A-Za-z]+\*?(?:\[[^\]\n]*\])?(?:\{[^{}\n]*(?:\{[^{}\n]*\}[^{}\n]*)*\})+")
_MD_IMAGE_RE = re.compile(r"!\[[^\]\n]*\]\([^\n)]*\)")
_MD_LINK_RE = re.compile(r"\[[^\]\n]+\]\([^\n)]*\)")
_MD_AUTO_LINK_RE = re.compile(r"<(?:(?:https?://[^>\s]+)|(?:[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}))>", re.I)
_CODE_RE = re.compile(r"`[^`\n]*`")
_URL_RE = re.compile(r"https?://[^\s，。；：！？、（）《》【】「」“”‘’…—<>'\"]+", re.I)
_EMAIL_RE = re.compile(r"[A-Za-z0-9._%+\-]+@[A-Za-z0-9.\-]+\.[A-Za-z]{2,}")

_PROTECT_PATTERNS = (
    _MD_FENCE_RE,
    _LATEX_ENV_RE,
    _LATEX_DISPLAY_BRACKET_RE,
    _LATEX_INLINE_PAREN_RE,
    _LATEX_DISPLAY_DOLLAR_RE,
    _LATEX_INLINE_DOLLAR_RE,
    _MD_IMAGE_RE,
    _MD_LINK_RE,
    _MD_AUTO_LINK_RE,
    _CODE_RE,
    _LATEX_COMMAND_RE,
    _URL_RE,
    _EMAIL_RE,
)
_PLACEHOLDER_RE = re.compile(r"\uE000CCWPROTECTED\d+\uE001")


def _yaml_parse_scalar(text):
    t = str(text).strip()
    if t in ("null", "~", "Null", "NULL", ""):
        return None
    if t == "{}":
        return {}
    if t == "[]":
        return []
    if t in ("true", "True", "TRUE"):
        return True
    if t in ("false", "False", "FALSE"):
        return False
    if t.startswith('"') and t.endswith('"'):
        body = t[1:-1]
        return (body.replace('\\"', '"')
                    .replace("\\n", "\n")
                    .replace("\\r", "\r")
                    .replace("\\t", "\t")
                    .replace("\\\\", "\\"))
    if t.startswith("'") and t.endswith("'"):
        return t[1:-1].replace("''", "'")
    if re.fullmatch(r"-?[0-9]+", t):
        return int(t)
    if re.fullmatch(r"-?[0-9]+\.[0-9]+", t):
        return float(t)
    return t


def _yaml_quote(value):
    s = str(value)
    if re.search(r"[:#\n\"'{\}\[\],&*!|>%@`-]|^\s|\s$", s) or s == "":
        return '"' + (s.replace("\\", "\\\\")
                       .replace("\n", "\\n")
                       .replace("\r", "\\r")
                       .replace("\t", "\\t")
                       .replace('"', '\\"')) + '"'
    return s


def _yaml_dump_lines(node, indent):
    pad = " " * indent
    lines = []
    if isinstance(node, dict):
        for k, v in node.items():
            key = _yaml_quote(k)
            if isinstance(v, (dict, list)):
                lines.append(pad + key + ":")
                lines.extend(_yaml_dump_lines(v, indent + 2))
            else:
                lines.append(pad + key + ": " + _yaml_quote(v))
    elif isinstance(node, list):
        for item in node:
            if isinstance(item, dict) and 1 <= len(item) <= 2:
                first = next(iter(item.items()))
                lines.append(pad + "- " + _yaml_quote(first[0]) + ": "
                             + _yaml_quote(first[1]))
                for k, v in list(item.items())[1:]:
                    lines.append(pad + "  " + _yaml_quote(k) + ": " + _yaml_quote(v))
            elif isinstance(item, (dict, list)):
                lines.append(pad + "-")
                lines.extend(_yaml_dump_lines(item, indent + 2))
            else:
                lines.append(pad + "- " + _yaml_quote(item))
    return lines


def _yaml_dump(root):
    return "\n".join(_yaml_dump_lines(root, 0)) + "\n"


def _yaml_fill(container, key, scalar):
    """把标量按已解析类型塞入容器（浅层无嵌套 list 场景）。"""
    container[key] = _yaml_parse_scalar(scalar)


def _yaml_load(text):
    """解析 YAML 子集 -> dict（根映射）。支持 key: scalar / key: 空开启多层 / - 列表。"""
    root = {}
    stack: list[tuple[int, dict | list]] = [(-1, root)]  # (indent, container)
    for raw in text.splitlines():
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        indent = len(raw) - len(raw.lstrip(" "))
        content = raw.strip()
        if content.startswith("- "):
            while len(stack) > 1 and stack[-1][0] >= indent:
                stack.pop()
            cont = stack[-1][1]
            if not isinstance(cont, list):
                if isinstance(cont, dict):
                    cont.setdefault("_items", [])
                    cont = cont["_items"]
                    stack[-1] = (stack[-1][0], cont)
                else:
                    continue
            cont.append(_yaml_parse_scalar(content[2:].strip()))
            continue
        if ":" not in content:
            continue
        key, _, val = content.partition(":")
        key = key.strip()
        val = val.strip()
        while len(stack) > 1 and stack[-1][0] >= indent:
            stack.pop()
        parent = stack[-1][1]
        if isinstance(parent, list):
            if parent and isinstance(parent[-1], dict):
                parent = parent[-1]
                stack[-1] = (stack[-1][0], parent)
            else:
                parent = root
            if not isinstance(parent, dict):
                continue
        if val == "":
            child = {}
            parent[key] = child
            stack.append((indent, child))
        elif val.startswith("- ") and "," not in val:
            child = [_yaml_parse_scalar(x) for x in (val[2:] + ",").split(",") if x]
            parent[key] = child
        else:
            parent[key] = _yaml_parse_scalar(val)
    return root


def load_yaml(path):
    """读取 YAML 文件；缺失/出错返回 {}。有 PyYAML 时优先 safe_load。"""
    try:
        with open(path, encoding="utf-8") as fh:
            text = fh.read()
    except Exception:
        return {}
    if _pyyaml is not None:
        try:
            return _pyyaml.safe_load(text) or {}
        except Exception:
            pass
    try:
        return _yaml_load(text)
    except Exception:
        return {}


def dump_yaml(path, data):
    """写入 YAML。"""
    try:
        os.makedirs(os.path.dirname(os.path.abspath(path)) or ".", exist_ok=True)
        text = (_pyyaml.safe_dump(data, allow_unicode=True, sort_keys=False)
                if _pyyaml is not None else _yaml_dump(data))
        with open(path, "w", encoding="utf-8") as fh:
            fh.write(text)
        return True
    except Exception:
        return False


def _slug(name):
    """将规则标题转成稳定键值（远程解析与内置列表共用同一算法）。"""
    s = name.replace("`", "").replace("？", "").replace("！", "")
    s = re.sub(r"[^\w\u4e00-\u9fff]+", "_", s).strip("_")
    return s or "rule"


def _protect(text, patterns=_PROTECT_PATTERNS, placeholders=None):
    """把 Markdown / LaTeX / URL / 邮箱 / 代码片段替换为私有区占位符。"""
    if placeholders is None:
        placeholders = {}
    counter = len(placeholders)

    def _rep(m):
        nonlocal counter
        ph = "\uE000CCWPROTECTED%d\uE001" % counter
        counter += 1
        placeholders[ph] = m.group(0)
        return ph

    for pat in patterns:
        text = pat.sub(_rep, text)
    return text, placeholders


def _restore(text, placeholders):
    """按创建顺序的逆序还原占位符，保证嵌套内容（如链接中的 URL）正确还原。"""
    for ph, val in reversed(list(placeholders.items())):
        text = text.replace(ph, val)
    return text


def _detect_newline(text):
    """返回输入中优先使用的换行符；混用时以首次出现者为准。"""
    if "\r\n" in text:
        first_crlf = text.find("\r\n")
        first_lf = text.find("\n")
        first_cr = text.find("\r")
        candidates = [(first_crlf, "\r\n")]
        if first_lf >= 0 and first_lf != first_crlf + 1:
            candidates.append((first_lf, "\n"))
        if first_cr >= 0 and first_cr != first_crlf:
            candidates.append((first_cr, "\r"))
        return min((c for c in candidates if c[0] >= 0), key=lambda x: x[0])[1]
    if "\n" in text:
        return "\n"
    if "\r" in text:
        return "\r"
    return "\n"


def _normalize_newlines(text):
    newline = _detect_newline(text)
    return text.replace("\r\n", "\n").replace("\r", "\n"), newline


def _restore_newlines(text, newline):
    return text if newline == "\n" else text.replace("\n", newline)


def _is_placeholder_line(line):
    return bool(_PLACEHOLDER_RE.fullmatch(line.strip()))


def _protect_markdown_lines(text, placeholders):
    """保护缩进代码行；保留普通 Markdown 标记所在行继续参与文字排版。"""
    lines = text.split("\n")
    counter = len(placeholders)
    for i, line in enumerate(lines):
        if line.startswith("    ") or line.startswith("\t"):
            ph = "\uE000CCWPROTECTED%d\uE001" % counter
            counter += 1
            placeholders[ph] = line
            lines[i] = ph
    return "\n".join(lines), placeholders


def _format_regular_text(text, enabled_all, enabled):
    """仅对普通行执行排版，避免规则跨越空行、换行和保护块。"""
    result = []
    for line in text.split("\n"):
        if not line.strip():
            result.append("")
            continue
        if _is_placeholder_line(line):
            result.append(line)
            continue
        current = line
        for rule in RULES:
            if not enabled_all and rule["key"] not in enabled:
                continue
            try:
                current = rule["fn"](current)
            except Exception:
                continue
        for fn in (_r_cn_en_space, _r_cn_digit_space, _r_digit_unit_space, _r_fw_punct_no_space):
            try:
                current = fn(current)
            except Exception:
                continue
        result.append(current)
    return "\n".join(result)


def _space_around_inline_placeholders(text, placeholders):
    """为行内保护片段补边界空格；整行保护块保持原样。"""
    lines = []
    inline_ph = {ph for ph, val in placeholders.items() if "\n" not in val}
    for line in text.split("\n"):
        if _is_placeholder_line(line):
            lines.append(line)
            continue
        for ph in inline_ph:
            esc = re.escape(ph)
            line = re.sub(r"(\S)(%s)" % esc, r"\1 \2", line)
            line = re.sub(r"(%s)([^\s，。；：！？、）】》」』])" % esc, r"\1 \2", line)
        lines.append(line)
    return "\n".join(lines)


# ---------------------------------------------------------------------------
# 13 条规则实现（每个函数：str -> str）
# ---------------------------------------------------------------------------
def _r_cn_en_space(text):
    """1. 中英文之间需要增加空格"""
    text = re.sub(r"(?<=[%s])(?=[A-Za-z])" % _CJK, " ", text)
    text = re.sub(r"(?<=[A-Za-z])(?=[%s])" % _CJK, " ", text)
    return text


def _r_cn_digit_space(text):
    """2. 中文与数字之间需要增加空格"""
    text = re.sub(r"(?<=[%s])(?=\d)" % _CJK, " ", text)
    text = re.sub(r"(?<=\d)(?=[%s])" % _CJK, " ", text)
    return text


def _r_digit_unit_space(text):
    """3. 数字与单位之间需要增加空格；度数/百分比与数字之间不加空格"""
    text = re.sub(r"(?<=\d)\s*(?=[°%％])", "", text)  # 90 ° -> 90°，15 % -> 15%
    text = re.sub(r"(?<=\d)(?=[A-Za-z]{1,4}(?![A-Za-z0-9]))", " ", text)  # 10Gbps -> 10 Gbps
    return text


def _r_fw_punct_no_space(text):
    """4. 全角标点与其他字符之间不加空格"""
    text = re.sub(r"\s+([，。；：！？、（）《》【】「」…—])", r"\1", text)
    text = re.sub(r"([，。；：！？、（）《》【】「」…—])\s+", r"\1", text)
    return text


def _r_css_text_spacing(text):
    """5. 用 text-spacing 来挽救？（CSS 自动排版说明，无需文本转换）"""
    return text


def _r_no_repeat_punct(text):
    """6. 不重复使用标点符号"""
    text = re.sub(r"([！？!?~～])\1+", r"\1", text)
    text = re.sub(r"([。，；：、])\1+", r"\1", text)
    text = re.sub(r"[！？!?][！？!?]+", "？！", text)
    return text


def _r_fullwidth_chinese_punct(text):
    """7. 使用全角中文标点（仅对含中文的行生效）"""
    def _conv_line(line):
        if not _CJK_RE.search(line):
            return line
        for hw, fw in _FW_MAP.items():
            if hw in ("!", "?"):
                # 中文句子中句尾的 ！？ 即使紧跟在英文缩写后也应转全角（如 JFGI！）
                pat = re.escape(hw) + r"(?![A-Za-z0-9])"
            elif hw == "(":
                # 全角括号：前侧非字母数字即可（如 成像 (NMRI) -> 成像（NMRI））
                pat = r"(?<![A-Za-z0-9])" + re.escape(hw)
            elif hw == ")":
                pat = re.escape(hw) + r"(?![A-Za-z0-9])"
            else:
                pat = r"(?<![A-Za-z0-9])" + re.escape(hw) + r"(?![A-Za-z0-9])"
            line = re.sub(pat, fw, line)
        # 英文双引号/单引号括住中文内容 -> 直角引号
        line = re.sub(r'"([^"\n]*?[%s][^"\n]*?)"' % _CJK, r"「\1」", line)
        line = re.sub(r"'([^'\n]*?[%s][^'\n]*?)'" % _CJK, r"『\1』", line)
        return line
    return "\n".join(_conv_line(l) for l in text.split("\n"))


def _r_fullwidth_digits(text):
    """8. 数字使用半角字符"""
    return re.sub(r"[０-９]", lambda m: chr(ord(m.group(0)) - 0xFEE0), text)


def _r_halfwidth_in_english(text):
    """9. 英文整句 / 特殊名词内使用半角标点（按英文书写惯例补空格）"""
    _ENG_MAP = {
        "，": ", ", "：": ": ", "；": "; ", "＆": " & ",
        "。": ".", "！": "!", "？": "?", "（": "(", "）": ")",
    }
    for fw, hw in _ENG_MAP.items():
        if hw in (", ", ": ", "; ", " & "):
            # 逗号/冒号/分号/& 后接空格，且前一个字符是英文字母（避免误伤 1，000 / 5：30）
            pat1 = r"(?<=[A-Za-z])" + re.escape(fw) + r"(?=[A-Za-z0-9])"
        else:
            pat1 = r"(?<=[A-Za-z0-9])" + re.escape(fw) + r"(?=[A-Za-z0-9])"
        text = re.sub(pat1, hw, text)
        # 句末标点紧跟英文并在关闭引号前（如 stay foolish。」 -> stay foolish.」）
        text = re.sub(r"(?<=[A-Za-z0-9])" + re.escape(fw) + r"(?=[」』》’”）])", hw, text)
    return text


_PROPER_NOUNS = {
    "github": "GitHub", "foursquare": "Foursquare", "microsoft": "Microsoft",
    "google": "Google", "facebook": "Facebook", "twitter": "Twitter",
    "youtube": "YouTube", "linkedin": "LinkedIn", "instagram": "Instagram",
    "wikipedia": "Wikipedia", "wechat": "WeChat", "javascript": "JavaScript",
    "typescript": "TypeScript", "html5": "HTML5", "css3": "CSS3",
    "json": "JSON", "http": "HTTP", "https": "HTTPS", "api": "API",
    "sql": "SQL", "php": "PHP", "ios": "iOS", "ipados": "iPadOS",
    "android": "Android", "iphone": "iPhone", "ipad": "iPad",
    "imac": "iMac", "mac": "Mac", "macos": "macOS", "windows": "Windows",
    "linux": "Linux", "bluetooth": "Bluetooth", "wifi": "Wi-Fi",
    "wi-fi": "Wi-Fi", "nextjs": "Next.js", "npm": "npm",
    "react": "React", "vue": "Vue", "mongodb": "MongoDB",
}
_CORPORATE_WORDS = {"corporation": "Corporation", "inc": "Inc"}


def _r_proper_nouns(text):
    """10. 专有名词使用正确的大小写"""
    for wrong, right in _PROPER_NOUNS.items():
        text = re.sub(r"(?<![A-Za-z0-9])" + re.escape(wrong) + r"(?![A-Za-z0-9])",
                      right, text, flags=re.IGNORECASE)
    for wrong, right in _CORPORATE_WORDS.items():
        text = re.sub(r"(?<![A-Za-z0-9])" + re.escape(wrong) + r"(?![A-Za-z0-9])",
                      right, text, flags=re.IGNORECASE)
    return text


_ABBR_MAP = {"ts": "TypeScript", "h5": "HTML5", "rjs": "React",
             "nextjs": "Next.js", "fed": "前端开发者"}


def _r_no_abbr(text):
    """11. 不要使用不地道的缩写"""
    for wrong, right in _ABBR_MAP.items():
        text = re.sub(r"(?<![A-Za-z0-9])" + re.escape(wrong) + r"(?![A-Za-z0-9])",
                      right, text, flags=re.IGNORECASE)
        if re.search(r"[\u4e00-\u9fff]", right):
            # 缩写展开为中文时，移除其前导空格（如 的 FED -> 的前端开发者）
            text = re.sub(r"\s+(?=" + re.escape(right) + r")", "", text)
    return text


def _r_space_around_links(text):
    """12. 链接之间增加空格（争议规则，默认关闭）"""
    text, ph = _protect(text, (_MD_IMAGE_RE, _MD_LINK_RE, _MD_AUTO_LINK_RE, _URL_RE, _EMAIL_RE))
    text = re.sub(r"(?<=\S)(?=\uE000CCWPROTECTED\d+\uE001)", " ", text)
    text = re.sub(r"(?<=\uE000CCWPROTECTED\d+\uE001)(?=\S)", " ", text)
    return _restore(text, ph)


def _r_corner_quotes(text):
    """13. 简体中文使用直角引号（争议规则，默认关闭）"""
    text = re.sub(r'"([^"\n]*?[%s][^"\n]*?)"' % _CJK, r"「\1」", text)
    text = re.sub(r"“([^”\n]*?[%s][^”\n]*?)”" % _CJK, r"「\1」", text)
    text = re.sub(r"'([^'\n]*?[%s][^'\n]*?)'" % _CJK, r"『\1』", text)
    text = re.sub(r"‘([^’\n]*?[%s][^’\n]*?)’" % _CJK, r"『\1』", text)
    return text


# ---------------------------------------------------------------------------
# 规则注册表（13 条，章节与远程 README 一一对应）
# ---------------------------------------------------------------------------
def _rule(key, section, name, disputed, fn, default=None):
    if default is None:
        default = not disputed
    return {"key": key, "section": section, "name": name,
            "disputed": disputed, "default": default, "fn": fn}


# 内嵌规则表：元数据（section/disputed/default）可由 rules.yaml 覆盖，fn 不可序列化，保留于此。
_EMBEDDED_RULES = [
    _rule(_slug("中英文之间需要增加空格"), "空格", "中英文之间需要增加空格", False, _r_cn_en_space),
    _rule(_slug("中文与数字之间需要增加空格"), "空格", "中文与数字之间需要增加空格", False, _r_cn_digit_space),
    _rule(_slug("数字与单位之间需要增加空格"), "空格", "数字与单位之间需要增加空格", False, _r_digit_unit_space),
    _rule(_slug("全角标点与其他字符之间不加空格"), "空格", "全角标点与其他字符之间不加空格", False, _r_fw_punct_no_space),
    _rule(_slug("用 `text-spacing` 来挽救？"), "空格", "用 text-spacing 来挽救？（说明：CSS 自动排版）", False, _r_css_text_spacing),
    _rule(_slug("不重复使用标点符号"), "标点符号", "不重复使用标点符号", False, _r_no_repeat_punct),
    _rule(_slug("使用全角中文标点"), "全角和半角", "使用全角中文标点", False, _r_fullwidth_chinese_punct),
    _rule(_slug("数字使用半角字符"), "全角和半角", "数字使用半角字符", False, _r_fullwidth_digits),
    _rule(_slug("遇到完整的英文整句、特殊名词，其内容使用半角标点"), "全角和半角", "遇到完整的英文整句、特殊名词，其内容使用半角标点", False, _r_halfwidth_in_english),
    _rule(_slug("专有名词使用正确的大小写"), "名词", "专有名词使用正确的大小写", False, _r_proper_nouns),
    _rule(_slug("不要使用不地道的缩写"), "名词", "不要使用不地道的缩写", False, _r_no_abbr),
    _rule(_slug("链接之间增加空格"), "争议", "链接之间增加空格", True, _r_space_around_links),
    _rule(_slug("简体中文使用直角引号"), "争议", "简体中文使用直角引号", True, _r_corner_quotes),
]

RULES = list(_EMBEDDED_RULES)

# key -> fn 实现映射（供 load_rules 用 yaml 元数据重建 RULES 时补回 fn）
_IMPL_BY_KEY = {r["key"]: r["fn"] for r in _EMBEDDED_RULES}
# key -> name 显示名映射（yaml 中仅存 key）
_NAME_BY_KEY = {r["key"]: r["name"] for r in _EMBEDDED_RULES}


def get_rule_by_key(key):
    for rule in RULES:
        if rule["key"] == key:
            return rule
    return None


def get_enabled_defaults():
    """返回按各规则 default 字段计算的默认启用集合。"""
    return {r["key"] for r in RULES if r.get("default", not r["disputed"])}


def format_text(text, enabled=None):
    """按启用的规则规范化文本；enabled=None 表示全部启用。

    名词/缩写规则可能引入新的英文，因此在全部规则之后再做一次
    空格与全角标点空白的收尾整理（规则幂等，可安全重复执行）。
    """
    if not text:
        return text
    enabled_all = enabled is None
    normalized, newline = _normalize_newlines(text)
    protected, ph = _protect(normalized)
    protected, ph = _protect_markdown_lines(protected, ph)
    formatted = _format_regular_text(protected, enabled_all, enabled or set())
    formatted = _space_around_inline_placeholders(formatted, ph)
    restored = _restore(formatted, ph)
    return _restore_newlines(restored, newline)


# ---------------------------------------------------------------------------
# 规则装载与设置持久化（rules.yaml）
# ---------------------------------------------------------------------------
_RULES_PATH = os.path.join(os.path.dirname(os.path.abspath(__file__)), "rules.yaml")


def load_rules(path=None):
    """从 rules.yaml 读取规则元数据，用内置实现补回 fn，重建并返回 RULES。

    yaml 中缺失的 key 保持内嵌默认；yaml 中多余但无内置实现的 key 会被忽略。
    失败时（无文件 / 解析出错）返回内嵌 RULES。
    """
    global RULES
    path = path or _RULES_PATH
    data = load_yaml(path)
    rules_map = data.get("rules") if isinstance(data, dict) else None
    if not isinstance(rules_map, dict) or not rules_map:
        RULES = list(_EMBEDDED_RULES)
        return RULES
    rebuilt = []
    for key, meta in rules_map.items():
        if not isinstance(meta, dict):
            continue
        fn = _IMPL_BY_KEY.get(key)
        name = _NAME_BY_KEY.get(key, key)
        if fn is None:
            continue  # 无内置实现的规则丢弃
        rebuilt.append(_rule(
            key,
            str(meta.get("section", "")),
            name,
            bool(meta.get("disputed", False)),
            fn,
            bool(meta.get("default", not bool(meta.get("disputed", False)))),
        ))
    # 若 yaml 未包含全部内置规则，补回缺失项，避免规则丢失
    seen = {r["key"] for r in rebuilt}
    for r in _EMBEDDED_RULES:
        if r["key"] not in seen:
            rebuilt.append(r)
    RULES = rebuilt
    return RULES


def load_settings(path=None):
    """读取 rules.yaml 的 settings 段；返回 (enabled_set, last_input)。"""
    path = path or _RULES_PATH
    data = load_yaml(path)
    st = data.get("settings") if isinstance(data, dict) else None
    st = st if isinstance(st, dict) else {}
    enabled = set()
    raw_en = st.get("enabled")
    if isinstance(raw_en, dict):
        enabled = {str(k) for k, v in raw_en.items() if v}
    elif isinstance(raw_en, list):
        enabled = {str(x) for x in raw_en}
    last_input = str(st.get("last_input", "") or "")
    return enabled, last_input


def save_settings(enabled, last_input="", path=None):
    """把用户设置写回 rules.yaml（保留 rules 元数据段）。"""
    path = path or _RULES_PATH
    data = load_yaml(path)
    if not isinstance(data, dict):
        data = {"rules": {}}
    if not isinstance(data.get("rules"), dict):
        data["rules"] = {}
    enabled_map = {str(k): True for k in enabled}
    data["settings"] = {
        "enabled": enabled_map,
        "last_input": last_input,
    }
    return dump_yaml(path, data)


def ensure_defaults(path=None):
    """若 rules.yaml 不存在或缺少 settings，用内置默认补齐并写回。"""
    path = path or _RULES_PATH
    data = load_yaml(path)
    if not isinstance(data, dict) or "rules" not in data:
        data = {"rules": {r["key"]: {
            "section": r["section"], "disputed": r["disputed"], "default": r["default"]}
            for r in _EMBEDDED_RULES}}
    st = data.get("settings")
    if not isinstance(st, dict):
        data["settings"] = {"enabled": {}, "last_input": ""}
    dump_yaml(path, data)


def initialize(config_path=None):
    """显式初始化默认配置（必要时写盘），由宿主（GUI / Rust / PyO3）按需调用。

    注意：模块导入阶段不再自动写 rules.yaml，避免在只读资源目录或
    嵌入 CPython 场景下产生隐式副作用。config_path=None 表示使用默认路径。
    """
    ensure_defaults(config_path)



