import type { Metadata } from "next";
import { fill, getDictionary } from "@/lib/i18n";
import { LINKS, VERSION } from "@/lib/site";

export async function generateMetadata({
  params,
}: {
  params: Promise<{ locale: string }>;
}): Promise<Metadata> {
  const { locale } = await params;
  return { title: getDictionary(locale).download.title };
}

function Code({ children }: { children: string }) {
  return (
    <pre className="mt-3 overflow-x-auto rounded-lg border border-border bg-base px-4 py-3 font-mono text-sm text-primary">
      <code>{children}</code>
    </pre>
  );
}

export default async function Install({ params }: { params: Promise<{ locale: string }> }) {
  const { locale } = await params;
  const t = getDictionary(locale).download;
  const guide = locale === "en" ? LINKS.guideEn : LINKS.guideTr;

  return (
    <div className="mx-auto max-w-3xl px-5 pt-20">
      <h1 className="text-4xl font-bold tracking-tight">{t.title}</h1>
      <p className="mt-3 font-mono text-sm text-secondary">
        {fill(t.subtitle, { version: VERSION })}
      </p>

      <p className="mt-8 rounded-xl border border-accent/30 bg-accent/5 p-5 text-sm leading-relaxed text-secondary">
        {t.notice}
      </p>

      <div className="mt-10 space-y-10">
        <section>
          <h2 className="text-xl font-semibold">{t.toolsTitle}</h2>
          <ol className="mt-4 space-y-3">
            {t.tools.map(([name, label, href, note]) => (
              <li key={name} className="rounded-xl border border-border bg-surface p-4">
                <div className="flex flex-wrap items-baseline gap-x-3 gap-y-1">
                  <span className="font-semibold">{name}</span>
                  {href ? (
                    <a href={href} className="font-mono text-xs text-accent hover:underline">
                      {label}
                    </a>
                  ) : (
                    <code className="font-mono text-xs text-accent">{label}</code>
                  )}
                </div>
                <p className="mt-1 text-sm text-secondary">{note}</p>
              </li>
            ))}
          </ol>
        </section>

        <section>
          <h2 className="text-xl font-semibold">{t.cloneTitle}</h2>
          <Code>{`git clone ${LINKS.clone}\ncd Devrem\npnpm install`}</Code>
        </section>

        <section>
          <h2 className="text-xl font-semibold">{t.runTitle}</h2>
          <Code>{"pnpm dev\npnpm build"}</Code>
          <p className="mt-3 text-sm leading-relaxed text-secondary">
            {fill(t.runText, { version: VERSION })}
          </p>
        </section>

        <div className="flex flex-col gap-3 sm:flex-row">
          <a
            href={guide}
            className="flex-1 rounded-xl bg-accent px-6 py-4 text-center font-semibold text-[var(--bg-base)] shadow-[0_0_30px_var(--accent-glow)] transition hover:brightness-110"
          >
            {t.guide} →
          </a>
          <a
            href={LINKS.repo}
            className="flex-1 rounded-xl border border-border bg-surface px-6 py-4 text-center font-medium transition hover:border-accent/50"
          >
            {t.source}
          </a>
        </div>

        <section className="rounded-xl border border-border bg-surface p-6">
          <h2 className="font-semibold">{t.requirementsTitle}</h2>
          <ul className="mt-3 list-disc space-y-1.5 pl-5 text-sm text-secondary">
            {t.requirements.map((r) => (
              <li key={r}>{r}</li>
            ))}
          </ul>
        </section>
      </div>
    </div>
  );
}
