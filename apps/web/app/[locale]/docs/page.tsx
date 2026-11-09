import type { Metadata } from "next";
import { getDictionary } from "@/lib/i18n";

export async function generateMetadata({
  params,
}: {
  params: Promise<{ locale: string }>;
}): Promise<Metadata> {
  const { locale } = await params;
  return { title: getDictionary(locale).docs.title };
}

export default async function Docs({ params }: { params: Promise<{ locale: string }> }) {
  const { locale } = await params;
  const t = getDictionary(locale).docs;

  return (
    <div className="mx-auto max-w-3xl px-5 pt-20">
      <h1 className="text-4xl font-bold tracking-tight">{t.title}</h1>
      <p className="mt-3 text-secondary">{t.subtitle}</p>

      <div className="mt-12 space-y-10">
        {t.sections.map((section) => (
          <section key={section.title}>
            <h2 className="text-xl font-semibold">{section.title}</h2>
            <div className="mt-3 space-y-3 leading-relaxed text-secondary">
              {section.body.map((p) => (
                <p key={p}>{p}</p>
              ))}
            </div>
          </section>
        ))}

        <section>
          <h2 className="text-xl font-semibold">{t.shortcutsTitle}</h2>
          <table className="mt-4 w-full overflow-hidden rounded-xl border border-border text-sm">
            <tbody>
              {t.shortcuts.map(([key, action]) => (
                <tr key={key} className="border-b border-border last:border-0">
                  <td className="w-1/2 bg-surface px-4 py-2.5">
                    <kbd className="rounded border border-border bg-base px-1.5 py-0.5 font-mono text-xs">
                      {key}
                    </kbd>
                  </td>
                  <td className="px-4 py-2.5 text-secondary">{action}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </section>
      </div>
    </div>
  );
}
