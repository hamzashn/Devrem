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

export default async function Download({ params }: { params: Promise<{ locale: string }> }) {
  const { locale } = await params;
  const t = getDictionary(locale).download;

  return (
    <div className="mx-auto max-w-3xl px-5 pt-20">
      <h1 className="text-4xl font-bold tracking-tight">{t.title}</h1>
      <p className="mt-3 font-mono text-sm text-secondary">
        {fill(t.subtitle, { version: VERSION })}
      </p>

      <div className="mt-10 flex flex-col gap-3 sm:flex-row">
        <a
          href={LINKS.installer}
          className="flex-1 rounded-xl bg-accent px-6 py-4 text-center font-semibold text-[var(--bg-base)] shadow-[0_0_30px_var(--accent-glow)] transition hover:brightness-110"
        >
          ⬇ {t.installer}
        </a>
        <a
          href={LINKS.msi}
          className="flex-1 rounded-xl border border-border bg-surface px-6 py-4 text-center font-medium transition hover:border-accent/50"
        >
          {t.msi}
        </a>
      </div>
      <a href={LINKS.releases} className="mt-3 inline-block text-sm text-accent hover:underline">
        {t.allReleases} →
      </a>

      <div className="mt-14 space-y-4">
        <section className="rounded-xl border border-border bg-surface p-6">
          <h2 className="font-semibold">{t.requirementsTitle}</h2>
          <ul className="mt-3 list-disc space-y-1.5 pl-5 text-sm text-secondary">
            {t.requirements.map((r) => (
              <li key={r}>{r}</li>
            ))}
          </ul>
        </section>
        <section className="rounded-xl border border-[#f59e0b]/30 bg-[#f59e0b]/5 p-6">
          <h2 className="font-semibold text-[#f59e0b]">{t.smartscreenTitle}</h2>
          <p className="mt-2 text-sm leading-relaxed text-secondary">{t.smartscreen}</p>
        </section>
        <section className="rounded-xl border border-border bg-surface p-6">
          <h2 className="font-semibold">{t.updatesTitle}</h2>
          <p className="mt-2 text-sm leading-relaxed text-secondary">{t.updates}</p>
        </section>
      </div>
    </div>
  );
}
