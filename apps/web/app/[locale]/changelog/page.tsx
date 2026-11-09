import fs from "node:fs";
import path from "node:path";
import type { Metadata } from "next";
import { getDictionary } from "@/lib/i18n";

export async function generateMetadata({
  params,
}: {
  params: Promise<{ locale: string }>;
}): Promise<Metadata> {
  const { locale } = await params;
  return { title: getDictionary(locale).changelog.title };
}

/** Yalnızca CHANGELOG.md'de kullanılan Markdown alt kümesi. */
function renderMarkdown(source: string): string {
  const escape = (s: string) =>
    s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
  const inline = (s: string) =>
    escape(s)
      .replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>")
      .replace(/\[(.+?)\]\((https?:\/\/[^)]+)\)/g, '<a href="$2">$1</a>');
  const html: string[] = [];
  let inList = false;
  let paragraph: string[] = [];
  const flush = () => {
    if (paragraph.length) html.push(`<p>${inline(paragraph.join(" "))}</p>`);
    paragraph = [];
  };
  const lines = source.split(/\r?\n/);
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i]!;
    if (/^- /.test(line)) {
      flush();
      if (!inList) html.push("<ul>");
      inList = true;
      // Girintili devam satırlarını birleştir
      let item = line.slice(2);
      while (i + 1 < lines.length && /^ {2}\S/.test(lines[i + 1]!))
        item += ` ${lines[++i]!.trim()}`;
      html.push(`<li>${inline(item)}</li>`);
      continue;
    }
    if (inList) {
      html.push("</ul>");
      inList = false;
    }
    if (line.startsWith("# ")) continue; // sayfa başlığı ayrıca gösteriliyor
    if (line.startsWith("## ")) {
      flush();
      html.push(`<h2>${inline(line.slice(3))}</h2>`);
    } else if (line.startsWith("### ")) {
      flush();
      html.push(`<h3>${inline(line.slice(4))}</h3>`);
    } else if (line.trim() === "") {
      flush();
    } else {
      paragraph.push(line.trim());
    }
  }
  flush();
  if (inList) html.push("</ul>");
  return html.join("\n");
}

export default async function Changelog({ params }: { params: Promise<{ locale: string }> }) {
  const { locale } = await params;
  const t = getDictionary(locale).changelog;
  const source = fs.readFileSync(path.join(process.cwd(), "..", "..", "CHANGELOG.md"), "utf8");

  return (
    <div className="mx-auto max-w-3xl px-5 pt-20">
      <h1 className="text-4xl font-bold tracking-tight">{t.title}</h1>
      <div
        className="prose-devrem mt-6"
        // Kaynak depodaki CHANGELOG.md; içerik kaçışlanarak üretilir.
        dangerouslySetInnerHTML={{ __html: renderMarkdown(source) }}
      />
    </div>
  );
}
