import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";

interface HelpDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

/** 静态使用说明；明确规则风险、结构保护和浏览器演示边界。 */
export function HelpDialog({ open, onOpenChange }: HelpDialogProps) {
  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogTrigger asChild>
        <Button variant="ghost" size="sm" data-testid="open-help" aria-label="打开帮助">
          <span className="flex h-4 w-4 items-center justify-center rounded-full border border-current text-[10px] font-semibold" aria-hidden="true">
            ?
          </span>
          帮助
        </Button>
      </DialogTrigger>
      <DialogContent
        data-testid="help-dialog"
        className="max-h-[calc(100vh-2rem)] w-[min(600px,calc(100vw-2rem))] max-w-[calc(100vw-2rem)] overflow-y-auto"
      >
        <DialogHeader>
          <DialogTitle>使用说明</DialogTitle>
          <DialogDescription>
            文案净排适合在复制、发布或提交前做一次可复核的文本整理。
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-5 text-sm leading-6" data-testid="help-content">
          <section className="space-y-1.5">
            <h3 className="font-semibold">先检查规则风险</h3>
            <p className="text-muted-foreground">
              低风险规则通常只调整间距或常见格式。标记为“需复核”或“高风险”的清洗规则可能删除、合并或改变原文信息，建议逐条启用，并在发布前对照原文检查。
            </p>
          </section>

          <section className="space-y-1.5">
            <h3 className="font-semibold">结构内容会受到保护</h3>
            <p className="text-muted-foreground">
              规则引擎会尽量保护 Markdown 链接、代码片段、URL、LaTeX 和其他已识别结构；保护不是内容审核或格式保证，复杂文档仍应查看完整输出。
            </p>
          </section>

          <section className="space-y-1.5">
            <h3 className="font-semibold">选择输出和复制动作</h3>
            <p className="text-muted-foreground">
              实时输出会在输入或设置变化后刷新，手动输出需要点击“立即排版”。“复制结果”会保留当前内容；“复制并清空”只有复制成功后才清空输入和输出，复制失败不会清除内容。
            </p>
          </section>

          <section className="space-y-1.5">
            <h3 className="font-semibold">浏览器演示模式的边界</h3>
            <p className="text-muted-foreground">
              浏览器预览只提供最小化 fallback，结果不代表桌面版 Rust 引擎的完整行为，也不代表真实桌面端 IPC、文件设置或系统剪贴板环境。重要内容请使用桌面版并人工复核。
            </p>
          </section>
        </div>

        <div className="flex justify-end">
          <DialogClose asChild>
            <Button data-testid="help-done">知道了</Button>
          </DialogClose>
        </div>
      </DialogContent>
    </Dialog>
  );
}