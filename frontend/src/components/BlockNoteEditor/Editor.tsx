"use client";

import { useEffect, useRef } from "react";
import type { PartialBlock, Block } from "@blocknote/core";
import { useCreateBlockNote } from "@blocknote/react";
import { BlockNoteView } from "@blocknote/shadcn";
import { useBlockNoteBidiDirection } from "@/hooks/useBlockNoteBidiDirection";
import "@blocknote/shadcn/style.css";
import "@blocknote/core/fonts/inter.css";

interface EditorProps {
  initialContent?: Block[];
  onChange?: (blocks: Block[]) => void;
  editable?: boolean;
}

export default function Editor({ initialContent, onChange, editable = true }: EditorProps) {
  const editor = useCreateBlockNote({
    initialContent: initialContent as PartialBlock[] | undefined,
  });
  const onChangeRef = useRef(onChange);
  const editorViewRef = useRef<HTMLDivElement>(null);
  onChangeRef.current = onChange;
  useBlockNoteBidiDirection(editorViewRef);

  // Handle content changes
  useEffect(() => {
    const handleChange = () => {
      onChangeRef.current?.(editor.document);
    };

    const unsubscribe = editor.onChange(handleChange);

    return () => {
      if (typeof unsubscribe === 'function') {
        unsubscribe();
      }
    };
  }, [editor]);

  return (
    <div dir="auto" className="bidi-editor">
      <BlockNoteView
        ref={editorViewRef}
        editor={editor}
        editable={editable}
        theme="light"
      />
    </div>
  );
}
