import { ReactNode } from "react";

export interface ContextMenuItem {
  label: string;
  icon?: ReactNode;
  onClick: () => void;
  disabled?: boolean;
  variant?: "default" | "warning" | "danger";
  separator?: boolean;
}

interface ContextMenuProps {
  x: number;
  y: number;
  title?: string;
  subtitle?: string;
  avatarUrl?: string;
  items: ContextMenuItem[];
  onClose: () => void;
}

export default function ContextMenu({
  x,
  y,
  title,
  subtitle,
  avatarUrl,
  items,
  onClose,
}: ContextMenuProps) {
  const adjustedX = Math.min(x, window.innerWidth - 220);
  const adjustedY = Math.min(y, window.innerHeight - (items.length * 44 + 100));

  return (
    <>
      <div className="fixed inset-0 z-40" onClick={onClose} />
      <div
        className="fixed z-50 bg-white border border-gray-200 rounded-xl shadow-2xl overflow-hidden w-56"
        style={{ left: adjustedX, top: adjustedY }}
      >
        {(title || subtitle) && (
          <div className="px-4 py-3 bg-gray-50 border-b border-gray-100 flex items-center gap-3">
            <div className="w-9 h-9 rounded-full bg-steel-blue flex items-center justify-center flex-shrink-0 overflow-hidden">
              {avatarUrl
                ? <img src={avatarUrl} alt={title} className="w-full h-full object-cover" />
                : <span className="text-white text-sm font-semibold">{title?.charAt(0).toUpperCase()}</span>
              }
            </div>
            <div className="min-w-0">
              <p className="text-gray-900 font-semibold text-sm truncate">{title}</p>
              {subtitle && <p className="text-gray-400 text-xs">{subtitle}</p>}
            </div>
          </div>
        )}

        <div className="py-1.5">
          {items.map((item, index) => (
            <div key={index}>
              {item.separator && <div className="my-1.5 border-t border-gray-100" />}
              <button
                onClick={() => { if (!item.disabled) { item.onClick(); onClose(); } }}
                disabled={item.disabled}
                className={`w-full text-left px-4 py-2.5 text-sm flex items-center gap-3 transition-colors
                  ${ item.disabled
                    ? "text-gray-300 cursor-not-allowed"
                    : item.variant === "danger"
                    ? "text-red-500 hover:bg-red-50 hover:text-red-600"
                    : item.variant === "warning"
                    ? "text-orange-500 hover:bg-orange-50"
                    : "text-gray-700 hover:bg-gray-50 hover:text-gray-900"
                  }`}
              >
                {item.icon && (
                  <span className={`flex-shrink-0 ${
                    item.disabled ? "text-gray-300"
                    : item.variant === "danger" ? "text-red-400"
                    : item.variant === "warning" ? "text-orange-400"
                    : "text-gray-400"
                  }`}>{item.icon}</span>
                )}
                {item.label}
              </button>
            </div>
          ))}
        </div>
      </div>
    </>
  );
}
