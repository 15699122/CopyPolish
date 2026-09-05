declare module "lucide-react" {
  import type { ComponentType, SVGProps } from "react";

  type LucideIcon = ComponentType<SVGProps<SVGSVGElement>>;

  export const Check: LucideIcon;
  export const Copy: LucideIcon;
  export const Eraser: LucideIcon;
  export const Maximize2: LucideIcon;
  export const Minus: LucideIcon;
  export const Settings: LucideIcon;
  export const X: LucideIcon;
}