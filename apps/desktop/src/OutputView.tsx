import { useEffect, useRef } from "react";
import katex from "katex";
import vegaEmbed from "vega-embed";
import type { Output } from "./types";
import { sanitizeHtml } from "./sanitizeHtml";

export default function OutputView({ output }: { output: Output }) {
  if (output.type === "text") return <pre className="output text-output">{output.text}</pre>;
  if (output.type === "error") {
    return <pre className="output error-output"><strong>{output.name}</strong>{"\n"}{output.message}</pre>;
  }
  if (output.type === "html") {
    return <div className="output" dangerouslySetInnerHTML={{ __html: sanitizeHtml(output.html) }} />;
  }
  if (output.type === "latex") {
    const html = katex.renderToString(output.latex, { displayMode: true, throwOnError: false });
    return <div className="output math-output" data-latex={output.latex} dangerouslySetInnerHTML={{ __html: html }} />;
  }
  if (output.type === "chart") return <ChartOutput spec={output.spec} />;
  return <pre className="output">{JSON.stringify(output, null, 2)}</pre>;
}

function ChartOutput({ spec }: { spec: Record<string, unknown> }) {
  const containerRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    let finalized = false;
    let view: { finalize: () => void } | undefined;
    if (containerRef.current) {
      void vegaEmbed(containerRef.current, spec, { actions: false, renderer: "canvas" })
        .then((result) => {
          if (finalized) result.view.finalize();
          else view = result.view;
        })
        .catch(() => undefined);
    }
    return () => {
      finalized = true;
      view?.finalize();
    };
  }, [spec]);
  return <div className="output chart-output" ref={containerRef} />;
}
