import { Icon, type IconName } from "./Icon";
import { glyph, type PadKind } from "../input/glyphs";
import type { Action } from "../input/bus";

/** Dicas de botões só com símbolos: o botão do controle + um ícone do que ele faz. */
export function Hints({ items, pad }: { items: [Action, IconName, string][]; pad: PadKind | null }) {
  return (
    <div className="hints">
      {items.map(([a, icon, label]) => (
        <span key={a} title={label} aria-label={label}>
          <kbd>{glyph(a, pad)}</kbd>
          <Icon name={icon} size={16} />
        </span>
      ))}
    </div>
  );
}
