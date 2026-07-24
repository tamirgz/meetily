"use client";

import { RefObject, useEffect } from "react";

const BLOCK_CONTENT_SELECTOR = ".bn-block-content[data-content-type]";

function applyAutomaticDirection(root: ParentNode) {
  root
    .querySelectorAll<HTMLElement>(BLOCK_CONTENT_SELECTOR)
    .forEach((block) => {
      if (block.getAttribute("dir") !== "auto") {
        block.setAttribute("dir", "auto");
      }
    });
}

/**
 * Applies native HTML bidirectional direction detection to each BlockNote block.
 *
 * Direction attributes are presentation-only and are added directly to the DOM,
 * so they do not change the ProseMirror document or trigger React editor updates.
 */
export function useBlockNoteBidiDirection(
  containerRef: RefObject<HTMLElement>,
  enabled = true,
) {
  useEffect(() => {
    const container = containerRef.current;
    if (!enabled || !container) return;

    applyAutomaticDirection(container);

    const observer = new MutationObserver((mutations) => {
      for (const mutation of mutations) {
        for (const addedNode of mutation.addedNodes) {
          if (!(addedNode instanceof Element)) continue;

          if (addedNode.matches(BLOCK_CONTENT_SELECTOR)) {
            const block = addedNode as HTMLElement;
            if (block.getAttribute("dir") !== "auto") {
              block.setAttribute("dir", "auto");
            }
          }

          applyAutomaticDirection(addedNode);
        }
      }
    });

    observer.observe(container, {
      childList: true,
      subtree: true,
    });

    return () => observer.disconnect();
  }, [containerRef, enabled]);
}
