"use client";

import { X } from "lucide-react";
import { useState } from "react";
import { useTranslation } from "@/lib/i18n/language-context";

interface BanDurationModalProps {
  memberName: string;
  onConfirm: (durationHours?: number) => void;
  onClose: () => void;
}

const BAN_DURATION_OPTIONS = [
  { key: "1hour" as const, hours: 1 },
  { key: "1day" as const, hours: 24 },
  { key: "2days" as const, hours: 48 },
  { key: "1week" as const, hours: 168 },
  { key: "1month" as const, hours: 720 },
  { key: "permanent" as const, hours: undefined },
];

export default function BanDurationModal({
  memberName,
  onConfirm,
  onClose,
}: BanDurationModalProps) {
  const [selectedDuration, setSelectedDuration] = useState<number | undefined>(24);
  const { t } = useTranslation();

  const handleConfirm = () => {
    onConfirm(selectedDuration);
    onClose();
  };

  return (
    <>
      <div className="fixed inset-0 bg-black/30 backdrop-blur-sm z-40" onClick={onClose} />
      <div className="fixed inset-0 flex items-center justify-center z-50 p-4">
        <div className="bg-white rounded-lg shadow-xl w-full max-w-md">
          {/* Header */}
          <div className="flex items-center justify-between p-4 border-b border-gray-200">
            <h2 className="text-lg font-semibold text-gray-900">
              {t("banDuration.title", { name: memberName })}
            </h2>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-bordeaux-hover transition-colors hover:cursor-pointer"
            >
              <X size={20} />
            </button>
          </div>

          {/* Content */}
          <div className="p-4">
            <p className="text-sm text-gray-600 mb-4">
              {t("banDuration.selectDuration")}
            </p>

            <div className="space-y-2">
              {BAN_DURATION_OPTIONS.map((duration) => (
                <button
                  key={duration.key}
                  onClick={() => setSelectedDuration(duration.hours)}
                  className={`w-full text-left px-4 py-3 rounded-lg border-2 transition-all ${
                    selectedDuration === duration.hours
                      ? "border-bordeaux bg-bordeaux/10 text-bordeaux font-medium"
                      : "border-gray-200 hover:border-gray-300 text-gray-700"
                  }`}
                >
                  {t(`banDuration.${duration.key}`)}
                  {duration.hours === undefined && (
                    <span className="ml-2 text-xs text-gray-500">
                      {t("banDuration.definitiveLabel")}
                    </span>
                  )}
                </button>
              ))}
            </div>
          </div>

          {/* Footer */}
          <div className="flex items-center justify-end gap-3 p-4 border-t border-gray-200">
            <button
              onClick={onClose}
              className="flex-1 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 rounded-lg transition-colors hover:text-bordeaux-hover hover:cursor-pointer"
            >
              {t("banDuration.cancel")}
            </button>
            <button
              onClick={handleConfirm}
              className="flex-1 px-4 py-2 text-sm font-medium text-white bg-bordeaux hover:bg-bordeaux-hover rounded-lg transition-colors"
            >
              {t("banDuration.confirm")}
            </button>
          </div>
        </div>
      </div>
    </>
  );
}
