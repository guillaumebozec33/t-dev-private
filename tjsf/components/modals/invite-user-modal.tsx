"use client";

import { useState, useEffect } from "react";
import Button from "@/components/ui/button";
import Input from "@/components/ui/input";
import { useMutation } from "@tanstack/react-query";
import { generateCode } from "@/lib/api/endpoints/servers";
import { X, Check, Copy } from "lucide-react";
import { InvitationOutput } from "@/types";
import { useTranslation } from "@/lib/i18n/language-context";
import {useSelectedServer} from "@/hooks";

interface InviteUserModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export default function InviteUserModal({
  isOpen,
  onClose,
}: InviteUserModalProps) {
  const {selectedServer} = useSelectedServer()
  const [inviteCode, setInviteCode] = useState("");
  const [maxUses, setMaxUses] = useState("1");
  const [copyState, setCopyState] = useState<"copy" | "copied" | "error">("copy");
  const [isChecked, setIsChecked] = useState(false);
  const [codeGenerated, setCodeGenerated] = useState(false);
  const { t } = useTranslation();

  useEffect(() => {
    if (!isOpen) {
      const timer = setTimeout(() => {
        setInviteCode("");
        setMaxUses("1");
        setCodeGenerated(false);
        setCopyState("copy");
        setIsChecked(false);
      }, 300);
      return () => clearTimeout(timer);
    }
  }, [isOpen]);

  useEffect(() => {
    setInviteCode("");
    setMaxUses("1");
    setCodeGenerated(false);
    setCopyState("copy");
    setIsChecked(false);
  }, [selectedServer?.id]);

  const mutation = useMutation({
    mutationFn: generateCode,
    onSuccess: (data: InvitationOutput) => {
      setInviteCode(data.code);
      setCodeGenerated(true);
    },
  });

  const generateInviteCode = () => {
      if(parseInt(maxUses) < 1 || parseInt(maxUses) > 100){
          setMaxUses("1");
      } else if (maxUses == "" || maxUses == "-"){
          setMaxUses("1");
      }
    if (selectedServer?.id) {
      mutation.mutate({ server_id: selectedServer?.id, max_uses: parseInt(maxUses) });
    }
  };

  const copyToClipboard = async () => {
    try {
      if (navigator.clipboard && window.isSecureContext) {
        await navigator.clipboard.writeText(inviteCode);
      } else {
        const textArea = document.createElement("textarea");
        textArea.value = inviteCode;
        textArea.style.position = "fixed";
        textArea.style.left = "-999999px";
        document.body.appendChild(textArea);
        textArea.select();
        document.execCommand("copy");
        document.body.removeChild(textArea);
      }
      setCopyState("copied");
      setIsChecked(true);
      setTimeout(() => {
        setCopyState("copy");
        setIsChecked(false);
      }, 2000);
    } catch (error) {
      console.error("Erreur lors de la copie:", error);
      setCopyState("error");
      setTimeout(() => {
        setCopyState("copy");
      }, 2000);
    }
  };

  if (!isOpen) {
    return null;
  }

  function changeIntValue(value:string){
      let number = value;
      number = number.toLowerCase();
      const numberList = number.split("e")
      console.log(numberList[0])
      if(parseInt(numberList[0]) >100){
          setMaxUses("100");
      }
      else if(parseInt(numberList[0]) < 1){
          setMaxUses("1");
      }
      else if (value.slice(0) == "-"){
          setMaxUses("1");
      }
      else {
          setMaxUses(numberList[0])
      }
  }

  return (
    <div className="fixed inset-0 bg-black/30 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-white rounded-lg p-4 sm:p-6 w-full max-w-md shadow-xl border border-gray-200 max-h-[90vh] overflow-y-auto">
        <div className="flex justify-between items-center mb-4">
          <h2 className="text-lg sm:text-xl font-semibold text-gray-900">
            {t("inviteModal.title")}
          </h2>
          <button
            onClick={onClose}
            className="text-gray-600 hover:text-bordeaux-hover hover:cursor-pointer"
          >
            <X size={20} />
          </button>
        </div>

        <div className="space-y-4">
          {!codeGenerated ? (
            <>
              <Input
                label={t("inviteModal.maxUses")}
                type="number"
                value={maxUses}
                onChange={changeIntValue}
                min="1"
                max="100"
              />
              <p className="text-xs text-gray-400">{t("inviteModal.validity")}</p>
              <div className="flex justify-end space-x-3 pt-4">
                  <button
                      onClick={onClose}
                      className="flex-1 w-full px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 rounded-lg transition-colors hover:cursor-pointer hover:text-bordeaux-hover"
                  >
                      {t("inviteModal.cancel")}
                  </button>
                <Button onClick={generateInviteCode} className="flex-1 disabled:opacity-50 disabled:cursor-not-allowed hover:cursor-pointer">{t("inviteModal.generate")}</Button>
              </div>
            </>
          ) : (
            <>
              <div>
                <label className="block text-xs font-semibold text-gray-300 uppercase tracking-wide mb-2">
                  {t("inviteModal.codeLabel")}
                </label>
                <div className="flex">
                  <input
                    type="text"
                    value={inviteCode}
                    readOnly
                    className="w-full bg-white border border-gray-300 rounded px-3 py-2.5 text-gray-900 placeholder-gray-400 focus:outline-none focus:ring-2 focus:ring-steel-blue focus:border-transparent"
                  />
                  <button
                    onClick={() => copyToClipboard()}
                    className="bg-bordeaux hover:bg-bordeaux-hover text-white px-4 py-2.5 rounded-r transition-colors flex items-center gap-2"
                  >
                    {isChecked ? <Check size={16} /> : <Copy size={16} />}
                    {t(`inviteModal.${copyState}`)}
                  </button>
                </div>
                <p className="text-xs text-gray-400 mt-2">
                  {t("inviteModal.validityInfo", { uses: maxUses })}
                </p>
              </div>
              <div className="flex justify-end pt-4">
                <button
                  onClick={onClose}
                  className="px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 rounded-lg transition-colors hover:text-bordeaux-hover"
                >
                  {t("inviteModal.close")}
                </button>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
}
